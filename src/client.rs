use std::time::{Duration, Instant};

use reqwest::{Client as HttpClient, Method, header};
use serde_json::Value;

use crate::{
    VERSION,
    config::ResolvedConfig,
    error::{CliError, Result},
};

#[derive(Clone)]
pub struct ApiClient {
    http: HttpClient,
    config: ResolvedConfig,
}

#[derive(Clone, Debug)]
pub struct ApiResponse {
    pub data: Value,
    pub status: u16,
}

impl ApiClient {
    pub fn new(config: ResolvedConfig) -> Result<Self> {
        if config.api_key.is_empty() {
            return Err(CliError::Api {
                code: "ERR_MISSING_AUTH".into(),
                message: "No API key configured".into(),
                status: 401,
                request_id: None,
                meta: None,
            });
        }

        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", config.api_key)).map_err(|_| {
                CliError::Config("The API key contains invalid header characters".into())
            })?,
        );
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json"),
        );

        let http = HttpClient::builder()
            .default_headers(headers)
            .user_agent(format!("llmpulse-cli/{VERSION}"))
            .timeout(Duration::from_secs(120))
            .build()?;
        Ok(Self { http, config })
    }

    pub fn project_id(&self) -> Result<u64> {
        self.config.project_id.ok_or_else(|| {
            CliError::Usage(
                "--project-id is required. Set a default with: llmpulse config set default_project_id <id>"
                    .into(),
            )
        })
    }

    pub async fn get(&self, path: &str, query: &[(String, String)]) -> Result<ApiResponse> {
        self.request(Method::GET, path, query, None).await
    }

    pub async fn post(&self, path: &str, body: Value) -> Result<ApiResponse> {
        self.request(Method::POST, path, &[], Some(body)).await
    }

    pub async fn patch(&self, path: &str, body: Value) -> Result<ApiResponse> {
        self.request(Method::PATCH, path, &[], Some(body)).await
    }

    pub async fn delete(&self, path: &str, query: &[(String, String)]) -> Result<ApiResponse> {
        self.request(Method::DELETE, path, query, None).await
    }

    pub async fn request(
        &self,
        method: Method,
        path: &str,
        query: &[(String, String)],
        body: Option<Value>,
    ) -> Result<ApiResponse> {
        let url = format!("{}{}", self.config.base_url.trim_end_matches('/'), path);
        let started = Instant::now();
        if self.config.verbose {
            eprintln!("{method} {url}");
            if let Some(body) = &body {
                eprintln!("{body}");
            }
        }

        let mut request = self.http.request(method.clone(), &url).query(query);
        if let Some(body) = body {
            request = request.json(&body);
        }
        let response = request.send().await?;
        let status = response.status();
        let status_text = status.canonical_reason().unwrap_or("Unknown status");
        let bytes = response.bytes().await?;
        let data = if bytes.is_empty() {
            Value::Object(Default::default())
        } else {
            serde_json::from_slice(&bytes).map_err(|error| CliError::Api {
                code: format!("HTTP_{}", status.as_u16()),
                message: format!("The API returned a non-JSON response: {error}"),
                status: status.as_u16(),
                request_id: None,
                meta: None,
            })?
        };

        if self.config.verbose {
            eprintln!(
                "{} {} ({}ms)",
                status.as_u16(),
                status_text,
                started.elapsed().as_millis()
            );
            if let Some(request_id) = data.get("request_id").and_then(Value::as_str) {
                eprintln!("Request ID: {request_id}");
            }
        }

        if !status.is_success() {
            let error = data.get("error");
            return Err(CliError::Api {
                code: error
                    .and_then(|value| value.get("code"))
                    .and_then(Value::as_str)
                    .map(str::to_owned)
                    .unwrap_or_else(|| format!("HTTP_{}", status.as_u16())),
                message: error
                    .and_then(|value| value.get("message"))
                    .and_then(Value::as_str)
                    .map(str::to_owned)
                    .unwrap_or_else(|| status_text.to_owned()),
                status: status.as_u16(),
                request_id: data
                    .get("request_id")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                meta: error.and_then(|value| value.get("meta")).cloned(),
            });
        }

        Ok(ApiResponse {
            data,
            status: status.as_u16(),
        })
    }
}
