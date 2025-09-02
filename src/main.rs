// src/main.rs
// cd /rust/jobfinder
// cargo clean
// cargo build
// cargo run

/*=============================================================================
  JobFinder (Rust) - Land Remote Full-Time Faster: Match + Cover Letter.
  Stack: Warp (HTTP) | Tokio (async) | SQLx (MySQL) | Askama (Template) | Reqwest

  Deskripsi
  Aplikasi pencari lowongan kerja yang memanggil RapidAPI JSearch, 
  menyimpan hasil ke MySQL, dan menampilkan daftar + detail dengan UI Bootstrap.
  Mendukung filter dan paginasi (page, num_pages).

  Fitur
  - Form pencarian: q, country, language, date_posted
  - Paginasi: page dan num_pages saat fetch data dari RapidAPI
  - Persistensi: tabel jobs dan job_apply_options (SQLx MySQL)
  - Templating: Askama (server-side render), UI mobile-first (Bootstrap)
  - Opsional: ringkasan/analisis AI (OpenAI) untuk deskripsi pekerjaan

  Endpoints (contoh)
  - GET  /                -> halaman pencarian (index)
  - POST /fetch           -> ambil data dari RapidAPI dan simpan ke DB
  - GET  /list[?q=...]    -> daftar job (filter judul/perusahaan/lokasi)
  - GET  /detail/{id}     -> detail job + opsi apply
  - GET  /resume          -> halaman resume (opsional)

  Environment Variables (.env)
  - DATABASE_URL   = mysql://user:pass@host:3306/job_finder
  - RAPIDAPI_KEY   = <your_rapidapi_key>
  - OPENAI_API_KEY = <opsional_jika_pakai_AI>

  Build & Run (contoh)
  - rustc/cargo versi stabil
  - set .env lalu: cargo run
  - dependencies utama: tokio, warp, sqlx(mysql, runtime-tokio-rustls, chrono, json), 
    reqwest (json, rustls-tls), serde(+derive), askama, askama_warp, dotenv, chrono, uuid

  Programmer Profile
  Name       : Kukuh Tripamungkas Wicaksono (Kukuh TW)
  Email      : kukuh.tw@gmail.com
  WhatsApp   : https://wa.me/628129893706
  Instagram  : @kukuhtw
  X/Twitter  : @kukuhtw
  Facebook   : https://www.facebook.com/kukuhtw
  LinkedIn   : https://id.linkedin.com/in/kukuhtw

  Catatan
  - Sesuaikan skema DB (migrasi SQLx) dengan struktur tabel yang dipakai.
  - Gunakan indexing pada kolom pencarian (judul, perusahaan, lokasi) untuk performa.
  - Tangani rate limit RapidAPI dan error network secara defensif (retry/backoff).
  - Pastikan sanitasi input query dan validasi parameter page/num_pages.

  Versi      : 0.1.0
 
=============================================================================*/


mod models;
mod database;
mod api_client;
mod openai_client;
mod handlers;
mod linkedin_client; // ⬅️ tambahkan


use std::env;
use dotenv::dotenv;
use warp::Filter; // ← tambahkan kembali


#[tokio::main]
async fn main() {
    dotenv().ok();
    
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let rapidapi_key = env::var("RAPIDAPI_KEY").expect("RAPIDAPI_KEY must be set");
    let openai_key = env::var("OPENAI_API_KEY").unwrap_or_default();
    
    let db = database::Database::new(&database_url).await.expect("Failed to connect to database");
    let api_client = api_client::JobApiClient::new(rapidapi_key.clone(), "jsearch.p.rapidapi.com".to_string());


    let openai_client = openai_client::OpenAIClient::new(openai_key);
    let li_client = linkedin_client::LinkedInApiClient::new(rapidapi_key.clone());

let fetch_li = warp::post()
    .and(warp::path("fetch_li"))
    .and(warp::body::form())
    .and(with_db(db.clone()))
    .and(with_li_client(li_client.clone()))
    .and_then(|params, db, li_client| async move {
        handlers::fetch_linkedin_handler(params, db, li_client).await
    });


    // Routes
    let index = warp::get()
        .and(warp::path::end())
        .and_then(|| async { handlers::index_handler().await });
    
    let fetch = warp::post()
        .and(warp::path("fetch"))
        .and(warp::body::form())
        .and(with_db(db.clone()))
        .and(with_api_client(api_client.clone()))
        .and_then(|params, db, api_client| async move {
            handlers::fetch_handler(params, db, api_client).await
        });
    
   let list = warp::get()
    .and(warp::path("list"))
    .and(warp::query::<std::collections::HashMap<String, String>>())
    .and(with_db(db.clone()))
    .and_then(|query_map: std::collections::HashMap<String, String>, db: database::Database| async move {
        let query = query_map.get("q").cloned();
        let page: usize = query_map.get("page").and_then(|s| s.parse().ok()).unwrap_or(1);
        handlers::list_handler(query, page, db).await
    });



    
   let view = warp::get()
    .and(warp::path("view"))
    .and(warp::path::param::<String>())
    .and(warp::path::end())
    .and(with_db(db.clone()))
    .and(with_li_client(li_client.clone())) // ⬅️ inject LinkedIn client
    .and_then(|job_id, db, li_client| async move {
        handlers::view_handler(job_id, db, li_client).await
    });
    
    let analyze = warp::post()
        .and(warp::path("analyze"))
        .and(warp::body::form())
        .and(with_db(db.clone()))
        .and(with_openai_client(openai_client.clone()))
        .and_then(|params, db, openai_client| async move {
            handlers::analyze_handler(params, db, openai_client).await
        });
    
   let resume = warp::get()
    .and(warp::path("resume"))
    .and(warp::query::<std::collections::HashMap<String, String>>())
    .and(with_db(db.clone()))
    .and_then(|query_map: std::collections::HashMap<String, String>, db: database::Database| async move {
        let id = query_map.get("id").and_then(|s| s.parse().ok());
        handlers::resume_handler(id, db).await
    });

    
    let resume_save = warp::post()
        .and(warp::path("resume_save"))
        .and(warp::body::form())
        .and(with_db(db.clone()))
        .and_then(|form: std::collections::HashMap<String, String>, db| async move {
            let id = form.get("id").and_then(|s| s.parse().ok()).unwrap_or(1);
            let description = form.get("description").cloned().unwrap_or_default();
            handlers::resume_save_handler(id, description, db).await
        });
    
    let cover_generate = warp::post()
        .and(warp::path("cover_generate"))
        .and(warp::body::form())
        .and(with_db(db.clone()))
        .and(with_openai_client(openai_client.clone()))
        .and_then(|params, db, openai_client| async move {
            handlers::cover_generate_handler(params, db, openai_client).await
        });
    
    let static_files = warp::get()
        .and(warp::fs::dir("static"));
    
    let routes = index
        .or(fetch)
        .or(list)
        .or(view)
        .or(analyze)
        .or(resume)
        .or(resume_save)
        .or(cover_generate)
        .or(static_files)
        .or(fetch_li);   // ⬅️ baru
    
    println!("Server started at http://localhost:3030");
    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

fn with_db(db: database::Database) -> impl Filter<Extract = (database::Database,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

fn with_api_client(client: api_client::JobApiClient) -> impl Filter<Extract = (api_client::JobApiClient,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || client.clone())
}

fn with_openai_client(client: openai_client::OpenAIClient) -> impl Filter<Extract = (openai_client::OpenAIClient,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || client.clone())
}


fn with_li_client(client: linkedin_client::LinkedInApiClient)
    -> impl Filter<Extract = (linkedin_client::LinkedInApiClient,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || client.clone())
}