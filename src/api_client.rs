// src/api_clients.rs

use reqwest::{
    header::{HeaderMap, HeaderValue, ACCEPT},
    Client,
};
use serde::{de::DeserializeOwned, Deserialize};

fn extract_first_json_slice<'a>(raw: &'a str) -> Option<&'a str> {
    let s = raw.trim_start_matches('\u{FEFF}').trim_start();
    let start = s.find(|c: char| c == '{' || c == '[')?;
    let bytes = s.as_bytes();
    let (mut i, mut depth, mut in_str, mut escape) = (start, 0usize, false, false);

    while i < bytes.len() {
        let b = bytes[i];
        if in_str {
            if escape { escape = false; }
            else if b == b'\\' { escape = true; }
            else if b == b'"' { in_str = false; }
        } else {
            match b {
                b'"' => in_str = true,
                b'{' | b'[' => depth += 1,
                b'}' | b']' => {
                    if depth == 0 { return None; }
                    depth -= 1;
                    if depth == 0 {
                        return Some(&s[start..=i]); // JSON pertama selesai
                    }
                }
                _ => {}
            }
        }
        i += 1;
    }
    None
}

fn parse_lenient<T: DeserializeOwned>(raw: &str) -> Result<T, serde_json::Error> {
    let trimmed = raw.trim_start_matches('\u{FEFF}').trim();
    let mut de = serde_json::Deserializer::from_str(trimmed);
    match T::deserialize(&mut de) {
        Ok(v) => Ok(v),
        Err(e1) => {
            if let Some(pos) = trimmed.rfind(['}', ']']) {
                let slice = &trimmed[..=pos];
                serde_json::from_str(slice).map_err(|_| e1)
            } else {
                Err(e1)
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ApiResponse {
    #[serde(default)]
    pub data: Vec<crate::models::Job>,
}

#[derive(Clone)]
pub struct JobApiClient {
    client: Client,
    api_key: String,
    host: String,
}

impl JobApiClient {
    pub fn new(api_key: String, host: String) -> Self {
        Self {
            client: Client::builder()
                .user_agent("job-finder/0.1")
                .build()
                .expect("build reqwest client"),
            api_key,
            host,
        }
    }

    /// (&str, i32, i32, &str, &str, &str)
    pub async fn search(
        &self,
        query: &str,
        page: i32,
        num_pages: i32,
        date_posted: &str,
        country: &str,
        language: &str,
    ) -> Result<ApiResponse, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("https://{}/search", self.host);

        let mut headers = HeaderMap::new();
        headers.insert("X-RapidAPI-Key", HeaderValue::from_str(&self.api_key)?);
        headers.insert("X-RapidAPI-Host", HeaderValue::from_str(&self.host)?);
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        let resp = self
            .client
            .get(&url)
            .headers(headers)
            .query(&[
                ("query", query),
                ("page", &page.to_string()),
                ("num_pages", &num_pages.to_string()),
                ("date_posted", date_posted),
                ("country", country),
                ("language", language),
            ])
            .send()
            .await?
            .error_for_status()?; // 4xx/5xx -> error duluan

        // Ambil body mentah
        let body: String = resp.text().await?;

        // 1) Coba ambil JSON pertama (kalau ada trailing sampah)
        if let Some(first) = extract_first_json_slice(&body) {
            if let Ok(ok) = serde_json::from_str::<ApiResponse>(first) {
                return Ok(ok);
            }
        }

        // 2) Fallback parser lenient
        if let Ok(ok) = parse_lenient::<ApiResponse>(&body) {
            return Ok(ok);
        }

        // 3) Masih gagal -> kirim error + preview
        let mut preview = body.clone();
        if preview.len() > 1200 {
            preview.truncate(1200);
            preview.push('…');
        }
        Err(format!(
            "failed to decode RapidAPI response (possibly trailing data). Preview:\n{preview}"
        )
        .into())
    }
}
