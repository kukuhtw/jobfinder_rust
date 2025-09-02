// src/handlers.rs
use askama::Template;
use warp::{http::StatusCode, Rejection, Reply};
use warp::reply::Response;
use warp::hyper::Body;
use warp::http;
use crate::linkedin_client;


use crate::{api_client, database, openai_client};

// ==================== Allowlists untuk filter LinkedIn ====================
const ALLOWED_EXPERIENCE: &[&str] = &[
    "intern","entry","associate","midSenior","director","executive","notApplicable"
];
const ALLOWED_WORKPLACE: &[&str] = &["remote","hybrid","onSite"];
const ALLOWED_EMPLOYMENT: &[&str] = &["contractor","fulltime","parttime","intern","temporary"];
const ALLOWED_DATE_POSTED: &[&str] = &["any","day","week","month"];

// ==================== Konstanta Negara ====================
const COUNTRIES: &[(&str, &str)] = &[
    ("ID","Indonesia"),
    ("US","United States"),
    ("CA","Canada"),
    ("GB","United Kingdom"),
    ("DE","Germany"),
    ("FR","France"),
    ("NL","Netherlands"),
    ("CH","Switzerland"),
    ("SE","Sweden"),
    ("NO","Norway"),
    ("DK","Denmark"),
    ("FI","Finland"),
    ("AT","Austria"),
    ("BE","Belgium"),
    ("IE","Ireland"),
    ("LU","Luxembourg"),
    ("IS","Iceland"),
    ("AU","Australia"),
    ("NZ","New Zealand"),
    ("SG","Singapore"),
    ("JP","Japan"),
    ("KR","South Korea"),
    ("TW","Taiwan"),
    ("HK","Hong Kong"),
    ("AE","United Arab Emirates"),
    ("QA","Qatar"),
    ("KW","Kuwait"),
    ("BH","Bahrain"),
    ("SA","Saudi Arabia"),
    ("OM","Oman"),
    ("IL","Israel"),
];

// ==== struct kecil untuk paging ====
#[derive(Debug, Clone)]
pub struct PageLink {
    pub n: usize,
    pub is_current: bool,
}

// ==== baris data di tabel (job + preview 100 kata) ====
#[derive(Debug, Clone)]
pub struct JobRow {
    pub job: crate::models::Job,
    pub preview: String,      // ringkasan 100 kata
    pub has_analysis: bool,   // ada/tidak analisis
}

// ==================== Templates ====================
#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    pub title: &'a str,
    pub subtitle: &'a str,
    pub countries: Vec<(&'a str, &'a str, bool)>, // (code, name, selected)
    pub languages: Vec<(&'a str, bool)>,          // (lang, selected)
    pub date_posted_opts: Vec<(&'a str, bool)>,   // (dp, selected)
}

#[derive(Template)]
#[template(path = "jobs.html")]
pub struct JobsTemplate {
    pub query: String,
    pub rows: Vec<JobRow>,
    pub current_page: usize,
    pub per_page: usize,
    pub total_jobs: usize,
    pub total_pages: usize,
    pub pages: Vec<PageLink>,
}

#[derive(Template)]
#[template(path = "job.html")]
pub struct JobTemplate {
    pub job: crate::models::Job,
    pub apply_options: Vec<crate::models::ApplyOption>,
}

#[derive(Template)]
#[template(path = "resume.html")]
pub struct ResumeTemplate {
    pub id: i32,
    pub resume: Option<String>,
}

// ==================== Helpers ====================

fn none_if_empty(s: Option<String>) -> Option<String> {
    s.and_then(|x| {
        let t = x.trim().to_string();
        if t.is_empty() { None } else { Some(t) }
    })
}

/// Bersihkan string list dipisah ';' (hapus spasi, kosong, duplikat).
fn clean_sc_list(input: Option<String>) -> Option<String> {
    let s = input?.trim().to_string();
    if s.is_empty() { return None; }
    let mut out: Vec<String> = s
        .split(';')
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_string())
        .collect();
    out.dedup();
    if out.is_empty() { None } else { Some(out.join(";")) }
}

/// Bersihkan + validasi terhadap allowlist (biar gak ada nilai ilegal).
fn clean_sc_list_with_allowlist(input: Option<String>, allow: &[&str]) -> Option<String> {
    use std::collections::HashSet;
    let s = input?.trim().to_string();
    if s.is_empty() { return None; }
    let set: HashSet<&str> = allow.iter().copied().collect();
    let mut out: Vec<String> = s
        .split(';')
        .map(|t| t.trim())
        .filter(|t| !t.is_empty() && set.contains(*t))
        .map(|t| t.to_string())
        .collect();
    out.dedup();
    if out.is_empty() { None } else { Some(out.join(";")) }
}

// ==================== Handlers ====================

pub async fn index_handler() -> Result<Response, Rejection> {
    let countries: Vec<(&str, &str, bool)> =
        COUNTRIES.iter().map(|(c, n)| (*c, *n, *c == "ID")).collect();

    let languages_src: &[&str] = &["en", "id", "de", "fr", "nl", "ja", "ko"];
    let languages: Vec<(&str, bool)> =
        languages_src.iter().map(|l| (*l, *l == "en")).collect();

    let dp_src: &[&str] = &["all", "today", "3days", "week", "month"];
    let date_posted_opts: Vec<(&str, bool)> =
        dp_src.iter().map(|dp| (*dp, *dp == "all")).collect();

    let page = IndexTemplate {
        title: "Job Finder",
        subtitle: "Cari lowongan, analisa kecocokan, dan buat cover letter",
        countries,
        languages,
        date_posted_opts,
    };

    let html = page.render().unwrap_or_else(|e| format!("Template error: {e}"));
    Ok(warp::reply::html(html).into_response())
}

pub async fn fetch_handler(
    params: std::collections::HashMap<String, String>,
    db: database::Database,
    api: api_client::JobApiClient,
) -> Result<Response, Rejection> {
    let query       = params.get("query").cloned().unwrap_or_default();
    let country     = params.get("country").cloned().unwrap_or_else(|| "ID".into());
    let language    = params.get("language").cloned().unwrap_or_else(|| "en".into());
    let date_posted = params.get("date_posted").cloned().unwrap_or_else(|| "all".into());

    let page: i32 = params.get("page").and_then(|s| s.parse().ok()).unwrap_or(1).max(1);
    let num_pages: i32 = params.get("num_pages").and_then(|s| s.parse().ok()).unwrap_or(1).max(1);

    let results = match api.search(&query, page, num_pages, &date_posted, &country, &language).await {
        Ok(r) => r,
        Err(e) => {
            return Ok(
                warp::reply::with_status(format!("API error: {e}"), StatusCode::BAD_GATEWAY)
                    .into_response()
            );
        }
    };

    for job in &results.data {
        if let Err(e) = db.upsert_job(job).await {
            return Ok(
                warp::reply::with_status(
                    format!("DB error saat menyimpan hasil: {e}"),
                    StatusCode::INTERNAL_SERVER_ERROR
                ).into_response()
            );
        }
    }

    let uri: http::Uri = format!("/list?q={}", urlencoding::encode(&query))
        .parse()
        .unwrap();

    let resp = warp::http::Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header("Location", uri.to_string())
        .body(Body::empty())
        .unwrap();
    Ok(resp)
}

pub async fn list_handler(
    query: Option<String>,
    page: usize,
    db: crate::database::Database,
) -> Result<Response, Rejection> {
    const PER_PAGE: usize = 50;

    let total_jobs_i64 = db.count_jobs(query.as_deref()).await.map_err(|_| warp::reject())?;
    let total_jobs = total_jobs_i64 as usize;
    let total_pages = (total_jobs + PER_PAGE - 1).max(1) / PER_PAGE;

    let current_page = page.clamp(1, total_pages.max(1));

    let jobs = db
        .list_jobs_paged(query.as_deref(), current_page as i64, PER_PAGE as i64)
        .await
        .map_err(|_| warp::reject())?;

    let rows: Vec<JobRow> = jobs
        .into_iter()
        .map(|job| {
            let text = job.matching_analysis.clone();
            let mut iter = text.split_whitespace();
            let first_100: Vec<&str> = iter.by_ref().take(100).collect();
            let preview = if iter.next().is_some() {
                format!("{} ...", first_100.join(" "))
            } else {
                first_100.join(" ")
            };

            let has_analysis = !job.matching_analysis.is_empty();

            JobRow {
                job,
                preview,
                has_analysis,
            }
        })
        .collect();

    let start = current_page.saturating_sub(3).max(1);
    let end = (current_page + 3).min(total_pages.max(1));
    let pages: Vec<PageLink> = (start..=end)
        .map(|n| PageLink { n, is_current: n == current_page })
        .collect();

    let page_ctx = JobsTemplate {
        query: query.unwrap_or_default(),
        rows,
        current_page,
        per_page: PER_PAGE,
        total_jobs,
        total_pages,
        pages,
    };

    let html = page_ctx.render().unwrap_or_else(|e| format!("Template error: {e}"));
    Ok(warp::reply::html(html).into_response())
}

pub async fn view_handler(
    job_id: String,
    db: database::Database,
    li: linkedin_client::LinkedInApiClient, // <-- baru
) -> Result<Response, Rejection> {
    use serde_json::json;

    // helper kecil
    fn is_blank(opt: Option<&String>) -> bool {
        opt.map(|s| s.trim().is_empty()).unwrap_or(true)
    }

    // 1) Ambil dari DB
    let mut job_opt = db.find_job(&job_id).await.map_err(|_| warp::reject())?;

    // 2) Jika job ada, prefix "li_" dan description kosong -> lazy enrich
    if let Some(job) = &job_opt {
        let is_linkedin = job.job_id.starts_with("li_");
        let desc_kosong = is_blank(job.job_description.as_ref());
        if is_linkedin && desc_kosong {
            // strip prefix: "li_4293585810" -> "4293585810"
            let li_id = job.job_id.strip_prefix("li_").unwrap_or(&job.job_id).to_string();

            match li.get_job(&li_id).await {
                Ok(v) => {
                    // API bisa return {"data": {...}} atau langsung {...}
                    let detail_obj = v.get("data").cloned().unwrap_or(v);

                    // upsert ke DB dengan mapper kamu
                    if let Err(e) = db.upsert_job_from_linkedin(&detail_obj).await {
                        eprintln!("upsert_job_from_linkedin error: {e}");
                    } else {
                        // optional: tambahkan apply option LinkedIn bila belum ada
                        let link = detail_obj
                            .get("linkedinUrl")
                            .and_then(|x| x.as_str())
                            .map(|s| s.to_string());
                        if let Err(e) = db.insert_apply_option_if_new(&job_id, "LinkedIn", link, None).await {
                            eprintln!("insert_apply_option_if_new error: {e}");
                        }
                    }

                    // refetch dari DB agar dapat description terbaru
                    job_opt = db.find_job(&job_id).await.map_err(|_| warp::reject())?;
                }
                Err(e) => {
                    eprintln!("LinkedIn get_job({li_id}) error: {e}");
                    // biarkan lanjut render tanpa description
                }
            }
        }
    }

    // 3) Render seperti biasa
    // 3) Render seperti biasa
    let opts = db.get_apply_options(&job_id).await;

    match (job_opt, opts) {
        (Some(job), Ok(apply_options)) => {
            let page = JobTemplate { job, apply_options };
            let html = page.render().unwrap_or_else(|e| format!("Template error: {e}"));
            Ok(warp::reply::html(html).into_response())
        }
        (None, _) => Ok(warp::reply::with_status("Job not found", StatusCode::NOT_FOUND).into_response()),
        (_, Err(e)) => Ok(
            warp::reply::with_status(format!("DB error: {e}"), StatusCode::INTERNAL_SERVER_ERROR)
                .into_response()
        ),
    }

}

pub async fn analyze_handler(
    params: std::collections::HashMap<String, String>,
    db: database::Database,
    openai: openai_client::OpenAIClient,
) -> Result<Response, Rejection> {
    let job_id = params.get("job_id").cloned().unwrap_or_default();
    let resume_id: i32 = params.get("resume_id").and_then(|s| s.parse().ok()).unwrap_or(1);

    let job = match db.find_job(&job_id).await {
        Ok(Some(j)) => j,
        Ok(None) => {
            return Ok(warp::reply::with_status("Job not found", StatusCode::NOT_FOUND).into_response())
        }
        Err(e) => {
            return Ok(
                warp::reply::with_status(format!("DB error: {e}"), StatusCode::INTERNAL_SERVER_ERROR)
                    .into_response()
            )
        }
    };

    let resume_text = db
        .get_resume(resume_id)
        .await
        .ok()
        .flatten()
        .map(|r| r.description)
        .unwrap_or_default();

    let analysis = if openai.available() {
        let desc = job.job_description.as_deref().unwrap_or_default();
        match openai.analyze_match(&resume_text, desc).await {
            Ok(s) => s,
            Err(e) => format!("OpenAI error: {e}"),
        }
    } else {
        "OpenAI API key not configured; skipping analysis.".to_string()
    };

    if let Err(e) = db.update_matching_analysis(&job_id, &analysis).await {
        return Ok(
            warp::reply::with_status(format!("Failed to save analysis: {e}"), StatusCode::INTERNAL_SERVER_ERROR)
                .into_response()
        );
    }

    let resp = warp::http::Response::builder()
        .status(StatusCode::FOUND)
        .header("Location", format!("/view/{job_id}"))
        .body(Body::empty())
        .unwrap();
    Ok(resp)
}

pub async fn resume_handler(
    id: Option<i32>,
    db: database::Database,
) -> Result<Response, Rejection> {
    let id = id.unwrap_or(1);
    match db.get_resume(id).await {
        Ok(Some(r)) => {
            let page = ResumeTemplate { id: r.id, resume: Some(r.description) };
            let html = page.render().unwrap_or_else(|e| format!("Template error: {e}"));
            Ok(warp::reply::html(html).into_response())
        }
        Ok(None) => {
            let page = ResumeTemplate { id, resume: None };
            let html = page.render().unwrap_or_else(|e| format!("Template error: {e}"));
            Ok(warp::reply::html(html).into_response())
        }
        Err(e) => Ok(
            warp::reply::with_status(format!("DB error: {e}"), StatusCode::INTERNAL_SERVER_ERROR)
                .into_response()
        ),
    }
}

pub async fn resume_save_handler(
    id: i32,
    description: String,
    db: database::Database,
) -> Result<Response, Rejection> {
    match db.upsert_resume(id, &description).await {
        Ok(()) => {
            let resp = warp::http::Response::builder()
                .status(StatusCode::FOUND)
                .header("Location", format!("/resume?id={id}"))
                .body(Body::empty())
                .unwrap();
            Ok(resp)
        }
        Err(e) => Ok(
            warp::reply::with_status(format!("DB error: {e}"), StatusCode::INTERNAL_SERVER_ERROR)
                .into_response()
        ),
    }
}

/// Fetch dari LinkedIn (jobs-api14/v2)
/// Versi ini melakukan enrichment langsung via /get.
/// (Jika ingin hemat kuota, simpan hasil search saja dan enrich saat /view)
pub async fn fetch_linkedin_handler(
    params: std::collections::HashMap<String, String>,
    db: database::Database,
    li: linkedin_client::LinkedInApiClient,
) -> Result<Response, Rejection> {
    // Ambil & bersihkan input
    let query            = params.get("query").cloned().unwrap_or_else(|| "kotlin".into());

    let experience_lvls  = clean_sc_list_with_allowlist(params.get("experience_levels").cloned(), ALLOWED_EXPERIENCE);
    let workplace_types  = clean_sc_list_with_allowlist(params.get("workplace_types").cloned(),  ALLOWED_WORKPLACE);
    let employment_types = clean_sc_list_with_allowlist(params.get("employment_types").cloned(), ALLOWED_EMPLOYMENT);

    let location         = none_if_empty(params.get("location").cloned()).or_else(|| Some("Worldwide".into()));

    let mut date_posted  = none_if_empty(params.get("date_posted").cloned()).or_else(|| Some("month".into()));
    if let Some(dp) = &date_posted {
        if !ALLOWED_DATE_POSTED.contains(&dp.as_str()) {
            date_posted = Some("month".into());
        }
    }

    // penting: None jika kosong, agar tidak mengirim nextToken=
    let next_token       = none_if_empty(params.get("next_token").cloned());

    // Panggil /v2/linkedin/search
    let search_json = match li.search(
        &query,
        experience_lvls.as_deref(),
        workplace_types.as_deref(),
        location.as_deref(),
        date_posted.as_deref(),
        employment_types.as_deref(),
        next_token.as_deref(),
    ).await {
        Ok(v) => v,
        Err(e) => {
            return Ok(
                warp::reply::with_status(format!("LinkedIn Search error: {e}"), StatusCode::BAD_GATEWAY)
                    .into_response()
            );
        }
    };

    // data: [ { id, title, companyName, location, datePosted, ... } ]
    let items = search_json.get("data").and_then(|d| d.as_array()).cloned().unwrap_or_default();
    let mut saved = 0usize;

    // Ambil nextToken untuk paging berikutnya (jika ada)
    let next_token_new = search_json
        .get("meta").and_then(|m| m.get("nextToken")).and_then(|v| v.as_str()).map(|s| s.to_string())
        .or_else(|| search_json.get("nextToken").and_then(|v| v.as_str()).map(|s| s.to_string()));

    // Enrichment per item: /v2/linkedin/get
    for item in items {
        let id_str = item.get("id").and_then(|v| v.as_str()).unwrap_or_default().to_string();
        if id_str.is_empty() { continue; }

        let detail = match li.get_job(&id_str).await {
            Ok(v) => v,
            Err(e) => {
                eprintln!("get_job({id_str}) error: {e}");
                serde_json::json!({ "data": item })  // ⬅️ pakai path penuh
            }
        };

        let detail_obj = detail.get("data").cloned().unwrap_or(detail.clone());

        if let Err(e) = db.upsert_job_from_linkedin(&detail_obj).await {
            eprintln!("upsert_job_from_linkedin error: {e}");
        } else {
            let job_id = format!("li_{}", id_str);
            let link   = detail_obj.get("linkedinUrl").and_then(|v| v.as_str()).map(|s| s.to_string());
            if let Err(e) = db.insert_apply_option_if_new(&job_id, "LinkedIn", link, None).await {
                eprintln!("insert_apply_option_if_new error: {e}");
            }
            saved += 1;
        }
    }

    // Redirect dengan notice (+ next_token jika ada)
    let mut notice = format!("LinkedIn fetched: {saved} items");
    let mut qs = format!("q={}", urlencoding::encode(&query));
    if let Some(nt) = next_token_new {
        notice.push_str(" (has next page)");
        qs.push_str(&format!("&next_token={}", urlencoding::encode(&nt)));
    }

    let uri: http::Uri = format!("/list?{}&notice={}", qs, urlencoding::encode(&notice))
        .parse()
        .unwrap();

    let resp = warp::http::Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header("Location", uri.to_string())
        .body(Body::empty())
        .unwrap();
    Ok(resp)
}

pub async fn cover_generate_handler(
    params: std::collections::HashMap<String, String>,
    db: database::Database,
    openai: openai_client::OpenAIClient,
) -> Result<Response, Rejection> {
    let job_id = params.get("job_id").cloned().unwrap_or_default();
    let resume_id: i32 = params.get("resume_id").and_then(|s| s.parse().ok()).unwrap_or(1);

    let job_val = match db.find_job(&job_id).await {
        Ok(Some(j)) => j,
        Ok(None) => {
            return Ok(warp::reply::with_status("Job not found", StatusCode::NOT_FOUND).into_response())
        }
        Err(e) => {
            return Ok(
                warp::reply::with_status(format!("DB error: {e}"), StatusCode::INTERNAL_SERVER_ERROR)
                    .into_response()
            )
        }
    };

    let resume_text = db
        .get_resume(resume_id)
        .await
        .ok()
        .flatten()
        .map(|r| r.description)
        .unwrap_or_default();

    let mut job_json = serde_json::Map::new();
    job_json.insert("job_title".into(), serde_json::Value::String(job_val.job_title.unwrap_or_default()));
    job_json.insert("employer_name".into(), serde_json::Value::String(job_val.employer_name.unwrap_or_default()));
    job_json.insert("job_location".into(), serde_json::Value::String(job_val.job_location.unwrap_or_default()));
    job_json.insert("job_description".into(), serde_json::Value::String(job_val.job_description.unwrap_or_default()));
    job_json.insert("job_posting_language".into(), serde_json::Value::String(job_val.job_posting_language.unwrap_or_else(|| "en".into())));

    let cover = if openai.available() {
        match openai.generate_cover_letter(&resume_text, &serde_json::Value::Object(job_json)).await {
            Ok(s) => s,
            Err(e) => format!("OpenAI error: {e}"),
        }
    } else {
        "OpenAI API key not configured; skipping cover letter.".to_string()
    };

    if let Err(e) = db.update_cover_letter(&job_id, &cover).await {
        return Ok(
            warp::reply::with_status(format!("Failed to save cover letter: {e}"), StatusCode::INTERNAL_SERVER_ERROR)
                .into_response()
        );
    }

    let resp = warp::http::Response::builder()
        .status(StatusCode::FOUND)
        .header("Location", format!("/view/{job_id}"))
        .body(Body::empty())
        .unwrap();
    Ok(resp)
}
