// src/database.rs

use crate::models::{ApplyOption, Job, Resume};
use chrono::Utc;
use sqlx::{MySql, Pool};

#[derive(Clone)]
pub struct Database {
    pool: Pool<MySql>,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = Pool::connect(database_url).await?;
        Ok(Database { pool })
    }

    // Tambahan di impl Database
pub async fn count_jobs(&self, query: Option<&str>) -> Result<i64, sqlx::Error> {
    if let Some(q) = query {
        let like = format!("%{}%", q);
        sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*) FROM jobs
               WHERE job_title LIKE ?
                  OR employer_name LIKE ?
                  OR job_location LIKE ?"#,
        )
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .fetch_one(&self.pool)
        .await
    } else {
        sqlx::query_scalar::<_, i64>(r#"SELECT COUNT(*) FROM jobs"#)
            .fetch_one(&self.pool)
            .await
    }
}

pub async fn list_jobs_paged(
    &self,
    query: Option<&str>,
    page: i64,
    per_page: i64,
) -> Result<Vec<Job>, sqlx::Error> {
    let page = page.max(1);
    let offset = (page - 1) * per_page;

    if let Some(q) = query {
        let like = format!("%{}%", q);
        sqlx::query_as::<_, Job>(
            r#"
            SELECT * FROM jobs
            WHERE job_title LIKE ?
               OR employer_name LIKE ?
               OR job_location LIKE ?
            ORDER BY updated_at DESC, job_posted_at_timestamp DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
    } else {
        sqlx::query_as::<_, Job>(
            r#"
            SELECT * FROM jobs
            ORDER BY updated_at DESC, job_posted_at_timestamp DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
    }
}


    pub async fn get_all_jobs(&self) -> Result<Vec<Job>, sqlx::Error> {
        // MySQL: gunakan ? (bukan $1)
        sqlx::query_as::<_, Job>(
            r#"
            SELECT * FROM jobs
            ORDER BY job_posted_at_timestamp DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn search_jobs(&self, q: &str) -> Result<Vec<Job>, sqlx::Error> {
        let like = format!("%{}%", q);
        sqlx::query_as::<_, Job>(
            r#"
            SELECT * FROM jobs
             WHERE job_title     LIKE ?
                OR employer_name LIKE ?
                OR job_location  LIKE ?
            ORDER BY job_posted_at_timestamp DESC
            "#,
        )
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .fetch_all(&self.pool)
        .await
    }

    /// Insert or update a job row (upsert).
    pub async fn upsert_job(&self, job: &Job) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO jobs (
                job_id, request_id, search_query, employer_name, employer_logo,
                employer_website, employer_company_type, employer_linkedin,
                job_publisher, job_employment_type, job_employment_type_text,
                job_employment_types_json, job_title, job_apply_link,
                job_apply_is_direct, job_apply_quality_score, job_description,
                job_is_remote, job_posted_human_readable, job_posted_at_timestamp,
                job_posted_at_datetime_utc, job_location, job_city, job_state,
                job_country, job_latitude, job_longitude, job_benefits_json,
                job_google_link, job_offer_expiration_datetime_utc,
                job_offer_expiration_timestamp, no_experience_required,
                required_experience_in_months, experience_mentioned,
                experience_preferred, job_salary_json, job_min_salary,
                job_max_salary, job_salary_currency, job_salary_period,
                job_highlights_json, job_job_title, job_posting_language,
                job_onet_soc, job_onet_job_zone, raw_json,
                created_at, updated_at, matching_analysis, cover_letter, isdelete
            ) VALUES (
                ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?,
                ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?,
                ?, ?, ?, ?, ?, ?, ?, ?, 0
            )
            ON DUPLICATE KEY UPDATE
                request_id = VALUES(request_id),
                search_query = VALUES(search_query),
                employer_name = VALUES(employer_name),
                employer_logo = VALUES(employer_logo),
                employer_website = VALUES(employer_website),
                employer_company_type = VALUES(employer_company_type),
                employer_linkedin = VALUES(employer_linkedin),
                job_publisher = VALUES(job_publisher),
                job_employment_type = VALUES(job_employment_type),
                job_employment_type_text = VALUES(job_employment_type_text),
                job_employment_types_json = VALUES(job_employment_types_json),
                job_title = VALUES(job_title),
                job_apply_link = VALUES(job_apply_link),
                job_apply_is_direct = VALUES(job_apply_is_direct),
                job_apply_quality_score = VALUES(job_apply_quality_score),
                job_description = VALUES(job_description),
                job_is_remote = VALUES(job_is_remote),
                job_posted_human_readable = VALUES(job_posted_human_readable),
                job_posted_at_timestamp = VALUES(job_posted_at_timestamp),
                job_posted_at_datetime_utc = VALUES(job_posted_at_datetime_utc),
                job_location = VALUES(job_location),
                job_city = VALUES(job_city),
                job_state = VALUES(job_state),
                job_country = VALUES(job_country),
                job_latitude = VALUES(job_latitude),
                job_longitude = VALUES(job_longitude),
                job_benefits_json = VALUES(job_benefits_json),
                job_google_link = VALUES(job_google_link),
                job_offer_expiration_datetime_utc = VALUES(job_offer_expiration_datetime_utc),
                job_offer_expiration_timestamp = VALUES(job_offer_expiration_timestamp),
                no_experience_required = VALUES(no_experience_required),
                required_experience_in_months = VALUES(required_experience_in_months),
                experience_mentioned = VALUES(experience_mentioned),
                experience_preferred = VALUES(experience_preferred),
                job_salary_json = VALUES(job_salary_json),
                job_min_salary = VALUES(job_min_salary),
                job_max_salary = VALUES(job_max_salary),
                job_salary_currency = VALUES(job_salary_currency),
                job_salary_period = VALUES(job_salary_period),
                job_highlights_json = VALUES(job_highlights_json),
                job_job_title = VALUES(job_job_title),
                job_posting_language = VALUES(job_posting_language),
                job_onet_soc = VALUES(job_onet_soc),
                job_onet_job_zone = VALUES(job_onet_job_zone),
                raw_json = VALUES(raw_json),
                matching_analysis = VALUES(matching_analysis),
                cover_letter = VALUES(cover_letter),
                updated_at = VALUES(updated_at)
            "#,
            job.job_id,
            job.request_id,
            job.search_query,
            job.employer_name,
            job.employer_logo,
            job.employer_website,
            job.employer_company_type,
            job.employer_linkedin,
            job.job_publisher,
            job.job_employment_type,
            job.job_employment_type_text,
            job.job_employment_types_json,
            job.job_title,
            job.job_apply_link,
            job.job_apply_is_direct,
            job.job_apply_quality_score,
            job.job_description,
            job.job_is_remote,
            job.job_posted_human_readable,
            job.job_posted_at_timestamp,
            job.job_posted_at_datetime_utc,
            job.job_location,
            job.job_city,
            job.job_state,
            job.job_country,
            job.job_latitude,
            job.job_longitude,
            job.job_benefits_json,
            job.job_google_link,
            job.job_offer_expiration_datetime_utc,
            job.job_offer_expiration_timestamp,
            job.no_experience_required,
            job.required_experience_in_months,
            job.experience_mentioned,
            job.experience_preferred,
            job.job_salary_json,
            job.job_min_salary,
            job.job_max_salary,
            job.job_salary_currency,
            job.job_salary_period,
            job.job_highlights_json,
            job.job_job_title,
            job.job_posting_language,
            job.job_onet_soc,
            job.job_onet_job_zone,
            job.raw_json,
            Utc::now(),
            Utc::now(),
            job.matching_analysis,
            job.cover_letter,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn upsert_apply_option(&self, option: &ApplyOption) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT IGNORE INTO job_apply_options (job_id, publisher, apply_link, is_direct)
            VALUES (?, ?, ?, ?)
            "#,
            option.job_id,
            option.publisher,
            option.apply_link,
            option.is_direct
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn find_job(&self, job_id: &str) -> Result<Option<Job>, sqlx::Error> {
        let job = sqlx::query_as::<_, Job>(
            r#"SELECT * FROM jobs WHERE job_id = ? LIMIT 1"#,
        )
        .bind(job_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(job)
    }

    pub async fn get_apply_options(&self, job_id: &str) -> Result<Vec<ApplyOption>, sqlx::Error> {
        let options = sqlx::query_as::<_, ApplyOption>(
            r#"
            SELECT id, job_id, publisher, apply_link, is_direct, created_at
              FROM job_apply_options
             WHERE job_id = ?
             ORDER BY id ASC
            "#,
        )
        .bind(job_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(options)
    }

    pub async fn get_resume(&self, id: i32) -> Result<Option<Resume>, sqlx::Error> {
        let resume = sqlx::query_as::<_, Resume>(
            r#"SELECT * FROM myresume WHERE id = ? LIMIT 1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(resume)
    }

    pub async fn upsert_resume(&self, id: i32, description: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO myresume (id, description)
            VALUES (?, ?)
            ON DUPLICATE KEY UPDATE description = VALUES(description)
            "#,
            id,
            description
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update_matching_analysis(
        &self,
        job_id: &str,
        analysis: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"UPDATE jobs SET matching_analysis = ?, updated_at = ? WHERE job_id = ?"#,
            analysis,
            Utc::now(),
            job_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update_cover_letter(
        &self,
        job_id: &str,
        cover_letter: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"UPDATE jobs SET cover_letter = ?, updated_at = ? WHERE job_id = ?"#,
            cover_letter,
            Utc::now(),
            job_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
