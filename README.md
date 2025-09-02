# JobFinder (Rust) — Land Remote Full-Time Faster: Match + Cover Letter

A fast job search aggregator built with **Warp (HTTP)**, **Tokio (async)**, **SQLx (MySQL)**, **Askama (templates)**, and **Reqwest**.
Fetch jobs from **RapidAPI JSearch**, persist them to **MySQL**, render **mobile-first UI (Bootstrap)**, and (optionally) add **AI analysis & cover-letter generation**.

UPDATED: September 2, 2025 — Added LinkedIn Job Source
📺 Watch here: https://www.youtube.com/watch?v=E6HY621XsSA



## Benefits (Manfaat)

### For Job Seekers

* **Find remote full-time faster** — filter dan paginasi langsung ke sumber (RapidAPI JSearch) untuk menghemat waktu riset.
* **AI Match & Gap Insights (opsional)** — ringkasan deskripsi kerja, highlight skill yang cocok & kekurangan yang perlu ditutup.
* **Instant cover letters** — surat lamaran otomatis yang disesuaikan dengan role, perusahaan, dan tone pilihan.
* **Consistent, mobile-first UI** — daftar & detail lowongan nyaman dibaca di smartphone.
* **Dedup & persistence** — lowongan disimpan di MySQL (berbasis `api_job_id`) sehingga mudah dilacak kembali tanpa data ganda.
* **Resume-ready workflow** — simpan/update resume lalu generate cover letter berbasis resume tersebut.
* **Time-to-apply turun** — dari “lihat lowongan” ke “kirim surat lamaran” dalam beberapa klik.

### For Developers / Teams

* **Production-friendly stack** — Warp + Tokio (async), SQLx (MySQL), Askama (SSR), Reqwest (rustls) = cepat, aman, tanpa OpenSSL.
* **Clear separation of concerns** — `api_client`, `database`, `handlers`, `models`, `openai_client` memudahkan perawatan & scaling.
* **Config via `.env`** — pasang `DATABASE_URL`, `RAPIDAPI_KEY`, dan (opsional) `OPENAI_API_KEY` untuk AI.
* **Scalable pagination** — kontrol `page` dan `num_pages` saat pengambilan dari API untuk batching terukur dan hemat kuota.
* **Indexing-ready** — mudah menambah indeks (judul/perusahaan/lokasi) agar query listing & filter tetap kencang.
* **Extensible** — gampang menambah filter (gaji, senioritas), saved search, notifikasi, atau export DOCX/PDF.

[Demo Video](https://youtu.be/NKZABTIH44s) • *Tagline:* **Remote Full-Time. Smart Match. Instant Cover Letters.**


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

CHECK file database.sql

### 4) Tables (example schema)

> Adjust as needed. Add/rename columns to match your `models` and handlers.

```sql
-- jobs: one row per job posting
--
-- Database: `job_finder`
--

-- --------------------------------------------------------

--
-- Table structure for table `jobs`
--

CREATE TABLE `jobs` (
  `job_id` varchar(64) NOT NULL,
  `request_id` varchar(64) DEFAULT NULL,
  `search_query` varchar(255) DEFAULT NULL,
  `employer_name` varchar(255) DEFAULT NULL,
  `employer_logo` varchar(1024) DEFAULT NULL,
  `employer_website` varchar(1024) DEFAULT NULL,
  `employer_company_type` varchar(255) DEFAULT NULL,
  `employer_linkedin` varchar(1024) DEFAULT NULL,
  `job_publisher` varchar(255) DEFAULT NULL,
  `job_employment_type` varchar(64) DEFAULT NULL,
  `job_employment_type_text` varchar(64) DEFAULT NULL,
  `job_employment_types_json` longtext CHARACTER SET utf8mb4 COLLATE utf8mb4_bin DEFAULT NULL CHECK (json_valid(`job_employment_types_json`)),
  `job_title` varchar(512) DEFAULT NULL,
  `job_apply_link` varchar(1024) DEFAULT NULL,
  `job_apply_is_direct` tinyint(1) DEFAULT NULL,
  `job_apply_quality_score` decimal(6,2) DEFAULT NULL,
  `job_description` longtext DEFAULT NULL,
  `job_is_remote` tinyint(1) DEFAULT NULL,
  `job_posted_human_readable` varchar(64) DEFAULT NULL,
  `job_posted_at_timestamp` bigint(20) DEFAULT NULL,
  `job_posted_at_datetime_utc` datetime DEFAULT NULL,
  `job_location` varchar(255) DEFAULT NULL,
  `job_city` varchar(128) DEFAULT NULL,
  `job_state` varchar(128) DEFAULT NULL,
  `job_country` varchar(16) DEFAULT NULL,
  `job_latitude` decimal(10,7) DEFAULT NULL,
  `job_longitude` decimal(10,7) DEFAULT NULL,
  `job_benefits_json` longtext CHARACTER SET utf8mb4 COLLATE utf8mb4_bin DEFAULT NULL CHECK (json_valid(`job_benefits_json`)),
  `job_google_link` varchar(1024) DEFAULT NULL,
  `job_offer_expiration_datetime_utc` datetime DEFAULT NULL,
  `job_offer_expiration_timestamp` bigint(20) DEFAULT NULL,
  `no_experience_required` tinyint(1) DEFAULT NULL,
  `required_experience_in_months` int(11) DEFAULT NULL,
  `experience_mentioned` tinyint(1) DEFAULT NULL,
  `experience_preferred` tinyint(1) DEFAULT NULL,
  `job_salary_json` longtext CHARACTER SET utf8mb4 COLLATE utf8mb4_bin DEFAULT NULL CHECK (json_valid(`job_salary_json`)),
  `job_min_salary` decimal(18,2) DEFAULT NULL,
  `job_max_salary` decimal(18,2) DEFAULT NULL,
  `job_salary_currency` varchar(8) DEFAULT NULL,
  `job_salary_period` varchar(32) DEFAULT NULL,
  `job_highlights_json` longtext CHARACTER SET utf8mb4 COLLATE utf8mb4_bin DEFAULT NULL CHECK (json_valid(`job_highlights_json`)),
  `job_job_title` varchar(255) DEFAULT NULL,
  `job_posting_language` varchar(16) DEFAULT NULL,
  `job_onet_soc` varchar(32) DEFAULT NULL,
  `job_onet_job_zone` varchar(32) DEFAULT NULL,
  `raw_json` longtext CHARACTER SET utf8mb4 COLLATE utf8mb4_bin DEFAULT NULL CHECK (json_valid(`raw_json`)),
  `created_at` timestamp NOT NULL DEFAULT current_timestamp(),
  `updated_at` timestamp NOT NULL DEFAULT current_timestamp() ON UPDATE current_timestamp(),
  `matching_analysis` text NOT NULL,
  `cover_letter` text DEFAULT NULL,
  `isdelete` tinyint(1) NOT NULL DEFAULT 0
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- --------------------------------------------------------

--
-- Table structure for table `job_apply_options`
--

CREATE TABLE `job_apply_options` (
  `id` bigint(20) UNSIGNED NOT NULL,
  `job_id` varchar(64) NOT NULL,
  `publisher` varchar(255) DEFAULT NULL,
  `apply_link` varchar(1024) DEFAULT NULL,
  `is_direct` tinyint(1) DEFAULT NULL,
  `created_at` timestamp NOT NULL DEFAULT current_timestamp()
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- --------------------------------------------------------

--
-- Table structure for table `myresume`
--

CREATE TABLE `myresume` (
  `id` int(11) NOT NULL,
  `description` text NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

--
-- Indexes for dumped tables
--

--
-- Indexes for table `jobs`
--
ALTER TABLE `jobs`
  ADD PRIMARY KEY (`job_id`),
  ADD KEY `idx_title` (`job_title`(191)),
  ADD KEY `idx_employer` (`employer_name`(191)),
  ADD KEY `idx_city` (`job_city`),
  ADD KEY `idx_country` (`job_country`);

--
-- Indexes for table `job_apply_options`
--
ALTER TABLE `job_apply_options`
  ADD PRIMARY KEY (`id`),
  ADD UNIQUE KEY `uniq_job_link` (`job_id`,`apply_link`(191));

--
-- Indexes for table `myresume`
--
ALTER TABLE `myresume`
  ADD PRIMARY KEY (`id`);

--
-- AUTO_INCREMENT for dumped tables
--

--
-- AUTO_INCREMENT for table `job_apply_options`
--
ALTER TABLE `job_apply_options`
  MODIFY `id` bigint(20) UNSIGNED NOT NULL AUTO_INCREMENT;

--
-- AUTO_INCREMENT for table `myresume`
--
ALTER TABLE `myresume`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT;

--
-- Constraints for dumped tables
--

--
-- Constraints for table `job_apply_options`
--
ALTER TABLE `job_apply_options`
  ADD CONSTRAINT `fk_apply_job` FOREIGN KEY (`job_id`) REFERENCES `jobs` (`job_id`) ON DELETE CASCADE;
COMMIT;

/*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;
/*!40101 SET CHARACTER_SET_RESULTS=@OLD_CHARACTER_SET_RESULTS */;
/*!40101 SET COLLATION_CONNECTION=@OLD_COLLATION_CONNECTION */;

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
📧 [kukuh.tw@gmail.com](mailto:kukuh.tw@gmail.com)
📱 [https://wa.me/628129893706](https://wa.me/628129893706)
X/Twitter: [@kukuhtw](https://x.com/kukuhtw) · IG: [@kukuhtw](https://instagram.com/kukuhtw)
FB: [facebook.com/kukuhtw](https://www.facebook.com/kukuhtw) · LinkedIn: [id.linkedin.com/in/kukuhtw](https://id.linkedin.com/in/kukuhtw)

---

## License


---

> **Tagline**: *Remote Full-Time. Smart Match. Instant Cover Letters.*
> *Land Remote Full-Time Faster: Match + Cover Letter.*
