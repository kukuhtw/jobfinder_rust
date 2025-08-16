// src/models.rs
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::types::BigDecimal;
use chrono::{DateTime, Utc};

// default untuk field yang tidak dikirim API
fn default_string() -> String { String::new() }
fn default_zero_i8() -> i8 { 0 }
fn default_now() -> DateTime<Utc> { Utc::now() }


#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Job {
    // --- teks / varchar ---
    pub job_id: String,
    pub request_id: Option<String>,
    pub search_query: Option<String>,
    pub employer_name: Option<String>,
    pub employer_logo: Option<String>,
    pub employer_website: Option<String>,
    pub employer_company_type: Option<String>,
    pub employer_linkedin: Option<String>,
    pub job_publisher: Option<String>,
    pub job_employment_type: Option<String>,
    pub job_employment_type_text: Option<String>,
    pub job_title: Option<String>,
    pub job_apply_link: Option<String>,
    pub job_posted_human_readable: Option<String>,
    pub job_location: Option<String>,
    pub job_city: Option<String>,
    pub job_state: Option<String>,
    pub job_country: Option<String>,
    pub job_google_link: Option<String>,
    pub job_salary_currency: Option<String>,
    pub job_salary_period: Option<String>,
    pub job_job_title: Option<String>,
    pub job_posting_language: Option<String>,
    pub job_onet_soc: Option<String>,
    pub job_onet_job_zone: Option<String>,

    // --- JSON ---
    pub job_employment_types_json: Option<serde_json::Value>,
    pub job_benefits_json: Option<serde_json::Value>,
    pub job_salary_json: Option<serde_json::Value>,
    pub job_highlights_json: Option<serde_json::Value>,
    pub raw_json: Option<serde_json::Value>,

    // --- angka & boolean ---
    // ⬇⬇ ubah i8 -> bool supaya bisa decode true/false dari API
    pub job_apply_is_direct: Option<bool>,
    pub job_apply_quality_score: Option<BigDecimal>,
    pub job_is_remote: Option<bool>,
    pub job_posted_at_timestamp: Option<i64>,
    pub job_latitude: Option<BigDecimal>,
    pub job_longitude: Option<BigDecimal>,
    pub no_experience_required: Option<bool>,
    pub required_experience_in_months: Option<i32>,
    pub experience_mentioned: Option<bool>,
    pub experience_preferred: Option<bool>,
    pub job_min_salary: Option<BigDecimal>,
    pub job_max_salary: Option<BigDecimal>,

    // --- teks panjang ---
    #[serde(default = "default_string")]
    pub matching_analysis: String,
    pub job_description: Option<String>,
    pub cover_letter: Option<String>,

    // --- waktu ---
    pub job_offer_expiration_timestamp: Option<i64>,
    pub job_posted_at_datetime_utc: Option<DateTime<Utc>>,
    pub job_offer_expiration_datetime_utc: Option<DateTime<Utc>>,


    // Field khusus DB (API tidak mengirim ini) -> kasih default
    #[serde(default = "default_now")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "default_now")]
    pub updated_at: DateTime<Utc>,
    #[serde(default = "default_zero_i8")]
    pub isdelete: i8,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ApplyOption {
    pub id: u64,
    pub job_id: String,
    pub publisher: Option<String>,
    pub apply_link: Option<String>,
    // ⬇⬇ ubah i8 -> bool untuk decode API
    pub is_direct: Option<bool>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Resume {
    pub id: i32,
    pub description: String,
}