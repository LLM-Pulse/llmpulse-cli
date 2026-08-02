use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("{message}")]
    Api {
        code: String,
        message: String,
        status: u16,
        request_id: Option<String>,
        meta: Option<serde_json::Value>,
    },
    #[error("{0}")]
    Config(String),
    #[error("{0}")]
    Usage(String),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Csv(#[from] csv::Error),
    #[error(transparent)]
    Dialoguer(#[from] dialoguer::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, CliError>;

pub fn format_error(error: &CliError) -> String {
    let CliError::Api {
        code,
        message,
        request_id,
        ..
    } = error
    else {
        return format!("Error: {error}");
    };

    let mut lines = vec![format!("Error: {message} [{code}]")];
    if let Some(request_id) = request_id {
        lines.push(format!("Request ID: {request_id}"));
    }

    let hint = match code.as_str() {
        "ERR_MISSING_AUTH" | "ERR_INVALID_API_KEY" => {
            Some("Run: llmpulse config set api_key <your-api-key>")
        }
        "ERR_REVOKED_API_KEY" => {
            Some("This API key has been revoked. Generate a new one in the LLM Pulse dashboard.")
        }
        "ERR_PROJECT_NOT_FOUND" => Some("Run: llmpulse dimensions projects list"),
        "ERR_RATE_LIMITED" => Some("Wait a moment, then try the request again."),
        _ => None,
    };

    if let Some(hint) = hint {
        lines.push(format!("Hint: {hint}"));
    }
    lines.join("\n")
}
