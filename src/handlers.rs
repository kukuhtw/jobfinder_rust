// src/handlers.rs
use askama::Template;
use warp::{http::StatusCode, Rejection, Reply}; // <- penting
use warp::reply::Response;
use warp::hyper::Body;
use warp::http;

use crate::{api_client, database, openai_client};

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
    pub preview: String, // ringkasan 100 kata
    pub has_analysis: bool, // 👈 add this
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
    pub rows: Vec<JobRow>,           // ⬅️ ganti dari Vec<Job>
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

// ==================== Handlers ====================

pub async fn index_handler() -> Result<Response, Rejection> {
    // Ambil SEMUA negara dari const COUNTRIES, tambahkan flag selected utk "ID"
    let countries: Vec<(&str, &str, bool)> =
        COUNTRIES.iter().map(|(c, n)| (*c, *n, *c == "ID")).collect();

    // Bahasa + flag default "en"
    let languages_src: &[&str] = &["en", "id", "de", "fr", "nl", "ja", "ko"];
    let languages: Vec<(&str, bool)> =
        languages_src.iter().map(|l| (*l, *l == "en")).collect();

    // Opsi tanggal + flag default "all"
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


    // ⬇️ ambil dari form; default 1
    let page: i32 = params.get("page").and_then(|s| s.parse().ok()).unwrap_or(1).max(1);
    let num_pages: i32 = params.get("num_pages").and_then(|s| s.parse().ok()).unwrap_or(1).max(1);

    // (&str, i32, i32, &str, &str, &str)
    let results = match api.search(&query, 1, 1, &date_posted, &country, &language).await {
        Ok(r) => r,
        Err(e) => {
            return Ok(
                warp::reply::with_status(format!("API error: {e}"), StatusCode::BAD_GATEWAY)
                    .into_response()
            );
        }
    };

    // ⬇️ iterasi array di dalam `data`
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

        // bikin preview 100 kata
     
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

    // bikin list PageLink agar template tidak perlu bandingkan &usize vs usize
    let start = current_page.saturating_sub(3).max(1);
    let end = (current_page + 3).min(total_pages.max(1));
    let pages: Vec<PageLink> = (start..=end)
        .map(|n| PageLink { n, is_current: n == current_page })
        .collect();

    let page_ctx = JobsTemplate {
    query: query.unwrap_or_default(),
    rows,              // ⬅️ bukan "jobs"
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
) -> Result<Response, Rejection> {
    let job = db.find_job(&job_id).await;
    let opts = db.get_apply_options(&job_id).await;

    match (job, opts) {
        (Ok(Some(job)), Ok(apply_options)) => {
            let page = JobTemplate { job, apply_options };
            let html = page.render().unwrap_or_else(|e| format!("Template error: {e}"));
            Ok(warp::reply::html(html).into_response())
        }
        (Ok(None), _) => Ok(warp::reply::with_status("Job not found", StatusCode::NOT_FOUND).into_response()),
        (Err(e), _) | (_, Err(e)) => Ok(
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
