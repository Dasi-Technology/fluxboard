use crate::error::{AppError, AppResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};

const GEMINI_API_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta/models";

#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
    generation_config: GenerationConfig,
}

#[derive(Debug, Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Debug, Serialize)]
struct Part {
    text: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GenerationConfig {
    temperature: f32,
    top_p: f32,
    top_k: i32,
    max_output_tokens: i32,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    content: ContentResponse,
}

#[derive(Debug, Deserialize)]
struct ContentResponse {
    parts: Vec<PartResponse>,
}

#[derive(Debug, Deserialize)]
struct PartResponse {
    text: String,
}

pub struct AiService {
    client: Client,
    api_key: String,
}

impl AiService {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    /// Generate a bullet point description from card title and existing description
    pub async fn generate_bullet_points(&self, title: &str, context: &str) -> AppResult<String> {
        let prompt = format!(
            "Based on the following card title and context, generate a concise bullet-point description (3-5 points) that outlines key aspects or tasks. Format using markdown bullet points (-).\n\nTitle: {}\nContext: {}\n\nGenerate only the bullet points, no additional text:",
            title,
            if context.is_empty() {
                "No additional context provided"
            } else {
                context
            }
        );

        self.generate_text(&prompt).await
    }

    /// Generate a long-form description from card title and existing description
    pub async fn generate_long_description(&self, title: &str, context: &str) -> AppResult<String> {
        let prompt = format!(
            "Based on the following card title and context, generate a detailed, well-structured description (2-3 paragraphs) that provides comprehensive information. Use markdown formatting for better readability.\n\nTitle: {}\nContext: {}\n\nGenerate only the description, no additional text:",
            title,
            if context.is_empty() {
                "No additional context provided"
            } else {
                context
            }
        );

        self.generate_text(&prompt).await
    }

    /// Internal method to call Gemini API
    async fn generate_text(&self, prompt: &str) -> AppResult<String> {
        let request = GeminiRequest {
            contents: vec![Content {
                parts: vec![Part {
                    text: prompt.to_string(),
                }],
            }],
            generation_config: GenerationConfig {
                temperature: 0.7,
                top_p: 0.95,
                top_k: 40,
                max_output_tokens: 1024,
            },
        };

        let url = format!(
            "{}/gemini-2.5-flash:generateContent?key={}",
            GEMINI_API_BASE_URL, self.api_key
        );

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                log::error!("Failed to call Gemini API: {}", e);
                AppError::InternalError("Failed to call AI service".to_string())
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            log::error!("Gemini API error {}: {}", status, error_text);
            return Err(AppError::InternalError(
                "AI service returned an error".to_string(),
            ));
        }

        let gemini_response: GeminiResponse = response.json().await.map_err(|e| {
            log::error!("Failed to parse Gemini response: {}", e);
            AppError::InternalError("Failed to parse AI response".to_string())
        })?;

        gemini_response
            .candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.trim().to_string())
            .ok_or_else(|| {
                log::error!("No content in Gemini response");
                AppError::InternalError("No content in AI response".to_string())
            })
    }
}
