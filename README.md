# JobFinder (Rust) — Land Remote Full-Time Faster: Match + Cover Letter

A fast job search aggregator built with **Warp (HTTP)**, **Tokio (async)**, **SQLx (MySQL)**, **Askama (templates)**, and **Reqwest**.
Fetch jobs from **RapidAPI JSearch**, persist them to **MySQL**, render **mobile-first UI (Bootstrap)**, and (optionally) add **AI analysis & cover-letter generation**.

https://youtu.be/NKZABTIH44s Demo Video
---

## Features

* **Search form**: `q`, `country`, `language`, `date_posted`
* **Pagination** when fetching from JSearch: `page`, `num_pages`
* **Persistence**: tables `jobs` and `job_apply_options` (SQLx MySQL)
* **SSR UI**: Askama templates + Bootstrap (mobile-first)
* **Optional AI**: summarize job description, match insights, and **auto cover letter**

---

## Stack

* **Rust** (stable), **Tokio**, **Warp**
* **SQLx** (MySQL + `runtime-tokio-rustls`, `chrono`, `json`)
* **Reqwest** (`json`, `rustls-tls`)
* **Askama** + `askama_warp`
* **dotenv**, **chrono**, \*\*uuid\`

> TLS note: using `reqwest` with `rustls-tls` means **no OpenSSL** needed.

---

## Getting Started

### 1) Prerequisites

* **Rust** (latest stable) & Cargo
  If you hit an “edition2024”/Cargo mismatch error, run:

  ```bash
  rustup update
  ```
* **MySQL** 5.7+ / 8.0+
* **RapidAPI key** for **JSearch**
* **OpenAI API key** (optional, only if using AI features)

### 2) Clone

```bash
git clone https://github.com/kukuhtw/jobfinder_rust.git
cd jobfinder_rust
```

### 3) Create Database

```sql
CREATE DATABASE job_finder CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
```

### 4) Tables (example schema)

> Adjust as needed. Add/rename columns to match your `models` and handlers.

```sql
-- jobs: one row per job posting
CREATE TABLE jobs (
  id              BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
  api_job_id      VARCHAR(191) NOT NULL,            -- unique from JSearch
  title           VARCHAR(255) NOT NULL,
  company         VARCHAR(255),
  location        VARCHAR(255),
  country         VARCHAR(32),
  language        VARCHAR(32),
  employment_type VARCHAR(64),                      -- full-time/part-time/remote flag etc
  description     MEDIUMTEXT,
  url             TEXT,
  date_posted     DATETIME,
  raw_json        JSON,
  created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE KEY uq_api_job_id (api_job_id),
  KEY idx_title (title),
  KEY idx_company (company),
  KEY idx_location (location),
  KEY idx_date_posted (date_posted)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- job_apply_options: multiple apply methods per job (email, ATS link, etc.)
CREATE TABLE job_apply_options (
  id         BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
  job_id     BIGINT UNSIGNED NOT NULL,
  label      VARCHAR(255) NOT NULL,
  apply_url  TEXT,
  extra_json JSON,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT fk_apply_job FOREIGN KEY (job_id) REFERENCES jobs(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- (Optional) store resume/profile/AI artifacts
CREATE TABLE resumes (
  id          BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
  owner_name  VARCHAR(191),
  description MEDIUMTEXT,
  updated_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
```

> For text search you can also add FULLTEXT indexes on `(title, company, location, description)` if your MySQL supports it.

### 5) Environment

Create `.env` in project root:

```env
DATABASE_URL=mysql://user:pass@127.0.0.1:3306/job_finder
RAPIDAPI_KEY=your_rapidapi_key_here
OPENAI_API_KEY=your_openai_api_key_here   # optional
```

### 6) Build & Run

```bash
# from the repo root
cargo clean
cargo build
cargo run
```

Server runs at **[http://localhost:3030](http://localhost:3030)**.

---

## Endpoints

### `GET /`

Index page: search form (q, country, language, date\_posted) and quick help.

### `POST /fetch`

Fetch jobs from RapidAPI JSearch and persist to MySQL.

**Form fields**:

* `q` (string) — query text (e.g., “Rust developer”)
* `country` (string, optional, e.g., `us`)
* `language` (string, optional, e.g., `en`)
* `date_posted` (string, optional; e.g., `today`, `3days`, `week`, `month`)
* `page` (int, optional; default `1`)
* `num_pages` (int, optional; how many pages to collect)

**cURL example:**

```bash
curl -X POST http://localhost:3030/fetch \
  -d "q=rust developer" \
  -d "country=us" \
  -d "language=en" \
  -d "date_posted=week" \
  -d "page=1" \
  -d "num_pages=2"
```

### `GET /list?q=...&page=...`

List jobs (server-side rendered).

* `q` filters **title/company/location**
* `page` (default `1`)

### `GET /view/{id}`

Job detail by local DB id + apply options.

### `POST /analyze` (optional AI)

Analyze/summarize job description or produce match notes using OpenAI.

**Form fields** (example):

* `job_id` (required)
* `mode` in `{summary|match}`

### `GET /resume?id=...`

Display stored resume data (optional page).

### `POST /resume_save`

Save/update resume text.

**Form fields**:

* `id` (int, default `1`)
* `description` (text)

### `POST /cover_generate` (optional AI)

Generate a tailored **cover letter** for a given job + resume.

**Form fields** (example):

* `job_id` (required)
* `resume_id` (optional, default `1`)
* `tone` (optional, e.g., `professional`, `concise`, `enthusiastic`)

### Static files

* `GET /static/*` serves assets (CSS/JS/images).

---

## RapidAPI JSearch

* Host: `jsearch.p.rapidapi.com`
* Auth: header `X-RapidAPI-Key: <RAPIDAPI_KEY>`
* The app uses `page` and `num_pages` to paginate upstream calls, then upserts into `jobs` (using `api_job_id` to prevent duplicates).

> Handle rate limits with backoff/retry. The code paths are prepared to surface errors cleanly.

---

## Project Layout (high-level)

```
src/
  api_client.rs      # RapidAPI JSearch client (Reqwest)
  database.rs        # SQLx pool + repository logic
  handlers.rs        # Warp route handlers
  models.rs          # Data models / DTOs
  openai_client.rs   # Optional OpenAI integration
  main.rs            # App wiring & routes
templates/
  index.html         # Askama templates (SSR)
  list.html
  view.html
static/
  css/, js/, img/
```

> `with_db`, `with_api_client`, and `with_openai_client` pass cloned handles into Warp filters. Ensure your database wrapper implements `Clone` (e.g., wraps `Arc<sqlx::Pool<MySql>>`).

---

## UI/UX Notes

* **Bootstrap** for mobile-first layout.
* On `/list`, show **first \~100 words** of description, with a **“View” modal** or detail page for the full content.
* Include **filters** (query input), **Reset**, and **pagination controls**.

---

## Security & Validation

* Sanitize/validate inputs for `q`, `country`, `language`, `date_posted`, `page`, `num_pages`.
* Enforce reasonable bounds (e.g., `1 ≤ page ≤ 50`, `1 ≤ num_pages ≤ 10`) to avoid abuse.
* Consider `rate limiting` at the app level to protect RapidAPI quota.

---

## Troubleshooting

* **TLS/OpenSSL**: we use `rustls`, so no OpenSSL setup is required.
* **Cargo/Edition errors**: `rustup update` to the latest stable toolchain.
* **RapidAPI quota**: implement exponential backoff; check HTTP 429/5xx handling.
* **SQLx**: verify `DATABASE_URL` and that schemas match your models.

---

## Roadmap

* ✅ Pagination for upstream fetch
* ✅ Basic filters + listing + details
* ✅ AI: summary/match + cover letter generation
* ☐ Saved searches & alerts
* ☐ Company/role deduplication & ranking
* ☐ Auth (optional) and per-user resumes/letters
* ☐ Export to PDF / DOCX (cover letters)

---

## Credits

**Programmer**
Kukuh Tripamungkas Wicaksono (Kukuh TW)
📧 **[kukuh.tw@gmail.com](mailto:kukuh.tw@gmail.com)**
📱 **[https://wa.me/628129893706](https://wa.me/628129893706)**
X/Twitter: **@kukuhtw** · IG: **@kukuhtw**
FB: **facebook.com/kukuhtw** · LinkedIn: **id.linkedin.com/in/kukuhtw**

---

## License


---

> **Tagline**: *Remote Full-Time. Smart Match. Instant Cover Letters.*
> *Land Remote Full-Time Faster: Match + Cover Letter.*
