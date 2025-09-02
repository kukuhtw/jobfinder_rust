use std::time::Duration;
use anyhow::{bail, Context, Result};
use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::Value;

#[derive(Clone)]
pub struct LinkedInApiClient {
    api_key: String,
    base_url: String,
    host: String,
    client: reqwest::Client,
}

impl LinkedInApiClient {
    pub fn new(api_key: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(20))
            .user_agent("job-finder/1.0 (+https://example.invalid)")
            .build()
            .expect("failed to build reqwest client");

        Self {
            api_key,
            base_url: "https://jobs-api14.p.rapidapi.com/v2/linkedin".to_string(),
            host: "jobs-api14.p.rapidapi.com".to_string(),
            client,
        }
    }

    fn auth_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("x-rapidapi-host", HeaderValue::from_static("jobs-api14.p.rapidapi.com"));
        headers.insert("x-rapidapi-key", HeaderValue::from_str(&self.api_key).expect("invalid rapidapi key"));
        headers
    }

    fn push_if_some(q: &mut Vec<(&'static str, String)>, key: &'static str, val: Option<&str>) {
        if let Some(s) = val {
            let t = s.trim();
            if !t.is_empty() { q.push((key, t.to_string())); }
        }
    }

    async fn send_json_with_detail(&self, req: reqwest::RequestBuilder) -> Result<Value> {
        let resp = req.send().await.context("request failed")?;
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();

        if !status.is_success() {
            // <- di sinilah body error ikut dinaikkan ke caller
            bail!("HTTP {}: {}", status, text);
        }

        let json: Value = serde_json::from_str(&text)
            .with_context(|| format!("invalid JSON: {}", text))?;
        Ok(json)
    }

    pub async fn search(
        &self,
        query: &str,
        experience_levels: Option<&str>,
        workplace_types: Option<&str>,
        location: Option<&str>,
        date_posted: Option<&str>,
        employment_types: Option<&str>,
        next_token: Option<&str>,
    ) -> Result<Value> {
        let url = format!("{}/search", self.base_url);

        let mut q: Vec<(&str, String)> = Vec::with_capacity(8);
        q.push(("query", query.trim().to_string()));
        Self::push_if_some(&mut q, "experienceLevels", experience_levels);
        Self::push_if_some(&mut q, "workplaceTypes", workplace_types);
        Self::push_if_some(&mut q, "location",        location);
        Self::push_if_some(&mut q, "datePosted",      date_posted);
        Self::push_if_some(&mut q, "employmentTypes", employment_types);
        Self::push_if_some(&mut q, "nextToken",       next_token);

        let req = self.client.get(&url).headers(self.auth_headers()).query(&q);
        self.send_json_with_detail(req).await
    }

    pub async fn get_job(&self, id: &str) -> Result<Value> {
        let url = format!("{}/get", self.base_url);
        let req = self.client.get(&url)
            .headers(self.auth_headers())
            .query(&[("id", id.trim().to_string())]);
        self.send_json_with_detail(req).await
    }
}
