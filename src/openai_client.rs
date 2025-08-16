// src/openai_client.rs
use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Debug, Deserialize, Clone)]
struct ChatCompletionChoice {
    message: ChatMessage,
}

#[derive(Debug, Deserialize, Clone)]
struct ChatCompletionResponse {
    choices: Vec<ChatCompletionChoice>,
}

#[derive(Clone)]
pub struct OpenAIClient {
    client: reqwest::Client,
    api_key: String,
}

impl OpenAIClient {
    pub fn new(api_key: String) -> Self {
        let client = reqwest::Client::new();
        Self { client, api_key }
    }

    pub fn available(&self) -> bool {
        !self.api_key.is_empty()
    }

    pub async fn analyze_match(
        &self,
        resume: &str,
        job_desc: &str
    ) -> Result<String, reqwest::Error> {
        let system = "You are an expert technical recruiter. Compare a candidate resume against a job description. Output a concise analysis";
        let user = format!(
            "RESUME:\n{}\n\nJOB DESCRIPTION:\n{}\n\nTASK:\n\
             - Provide a match score (0-100%).\n\
             - Summarize fit in 3-6 sentences.\n\
             - List 3-6 strengths (bullets).\n\
             - List 3-6 gaps/risks (bullets) with quick upskilling tips.\n\
             - Suggest a short tailored headline to use at the top of the resume.\n\
             Keep it under 2500 characters. Use Markdown.",
            resume, job_desc
        );

        let request = ChatCompletionRequest {
            model: "gpt-5".to_string(),
            messages: vec![
                ChatMessage { role: "system".into(), content: system.into() },
                ChatMessage { role: "user".into(), content: user },
            ],
        };

        let resp = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await?
            .error_for_status()?;

        let api_response: ChatCompletionResponse = resp.json().await?;
        let text = api_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_else(|| "OpenAI returned empty response or unexpected format.".to_string());

        Ok(text)
    }

    pub async fn generate_cover_letter(
        &self,
        resume: &str,
        job: &serde_json::Value
    ) -> Result<String, reqwest::Error> {
        let job_title = job.get("job_title").and_then(|v| v.as_str()).unwrap_or("");
        let employer  = job.get("employer_name").and_then(|v| v.as_str()).unwrap_or("");
        let location  = job.get("job_location").and_then(|v| v.as_str()).unwrap_or("");
        let desc      = job.get("job_description").and_then(|v| v.as_str()).unwrap_or("");
        let lang      = job.get("job_posting_language").and_then(|v| v.as_str()).unwrap_or("en");
        let target_lang = if lang == "en" || lang == "id" { lang } else { "en" };

        let system = "You are an expert career coach and recruiter. Write a concise, tailored, professional cover letter to Employer. Greeting first to name of Employer / company";
        let user = format!(
            "RESUME:\n{}\n\nJOB TITLE: {}\nEMPLOYER: {}\nLOCATION: {}\n\
             JOB DESCRIPTION:\n{}\n\nTASK:\n\
             - Write a one-page cover letter (200–300 words) in language: {}.\n\
             - Be specific to the job; highlight 3–4 matching strengths from the resume.\n\
             - Use a confident but humble tone, avoid clichés, no formatting, plain text.\n\
             - Start with greetings, a strong opening hook. End with a short call-to-action.\n\
             - If the candidate name appears in the resume, use it; otherwise omit the name in the signature.",
            resume, job_title, employer, location, desc, target_lang
        );

        let request = ChatCompletionRequest {
            model: "gpt-5".to_string(),
            messages: vec![
                ChatMessage { role: "system".into(), content: system.into() },
                ChatMessage { role: "user".into(),   content: user },
            ],
        };

        let resp = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await?
            .error_for_status()?;

        let api_response: ChatCompletionResponse = resp.json().await?;
        let text = api_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_else(|| "OpenAI returned empty response or unexpected format.".to_string());

        Ok(text)
    }
}
