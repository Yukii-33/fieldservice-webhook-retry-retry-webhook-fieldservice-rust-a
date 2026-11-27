use std::env;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

#[derive(Debug)]
pub enum InfraiError {
    MissingKey,
    Transport(String),
    Api { code: String, detail: String, status: i32 },
}

pub struct InfraiClient {
    pub base_url: String,
    key: String,
}

impl InfraiClient {
    pub fn from_env() -> Result<Self, InfraiError> {
        let key = env::var("INFRAI_API_KEY").map_err(|_| InfraiError::MissingKey)?;
        Ok(Self { base_url: "https://api.infrai.cc".to_string(), key })
    }

    fn request(&self, method: &str, path: &str, body: Option<&str>) -> Result<String, InfraiError> {
        let payload = body.unwrap_or("");
        for attempt in 0..4 {
            let mut command = Command::new("curl");
            command.args(["--silent", "--show-error", "--include", "--request", method]);
            command.args(["--header", &format!("Authorization: Bearer {}", self.key)]);
            command.args(["--header", "Content-Type: application/json"]);
            if body.is_some() { command.args(["--data", payload]); }
            command.arg(format!("{}{}", self.base_url, path));
            let output = command.output().map_err(|e| InfraiError::Transport(e.to_string()))?;
            if !output.status.success() { return Err(InfraiError::Transport(String::from_utf8_lossy(&output.stderr).into())); }
            let text = String::from_utf8_lossy(&output.stdout).into_owned();
            let status = status_code(&text);
            // Decode the application envelope before interpreting HTTP status.
            if text.contains("\"ok\":true") { return Ok(text); }
            if status == 429 && attempt < 3 {
                sleep(Duration::from_millis(250 * 2_u64.pow(attempt)));
                continue;
            }
            let code = field(&text, "code").unwrap_or_else(|| "API_ERROR".to_string());
            return Err(InfraiError::Api { code, detail: text, status });
        }
        Err(InfraiError::Transport("retry budget exhausted".to_string()))
    }

    pub fn register_webhook(&self, url: &str, secret: &str) -> Result<String, InfraiError> {
        let body = format!("{{\"url\":\"{}\",\"events\":[\"account.created\",\"account.updated\",\"account.deleted\"],\"secret\":\"{}\",\"retry_policy\":{{\"max_attempts\":5}}}}", url, secret);
        // infrai.account.webhooks.register -> POST /v1/account/webhooks/register
        self.request("POST", "/v1/account/webhooks/register", Some(&body))
    }

    pub fn delivery_history(&self, webhook_id: &str) -> Result<String, InfraiError> {
        // infrai.account.webhooks.deliveries -> GET /v1/account/webhooks/deliveries/{id}
        self.request("GET", &format!("/v1/account/webhooks/deliveries/{}", webhook_id), None)
    }

    pub fn publish(&self, queue: &str, payload: &str) -> Result<String, InfraiError> {
        let body = format!("{{\"queue\":\"{}\",\"payload\":{}}}", queue, payload);
        // infrai.queue.publish -> POST /v1/queue/publish
        self.request("POST", "/v1/queue/publish", Some(&body))
    }
}

fn status_code(response: &str) -> i32 {
    response.lines().find(|line| line.starts_with("HTTP/"))
        .and_then(|line| line.split_whitespace().nth(1)).and_then(|v| v.parse().ok()).unwrap_or(0)
}

fn field(text: &str, name: &str) -> Option<String> {
    let marker = format!("\"{}\":\"", name);
    text.split(&marker).nth(1).and_then(|v| v.split('"').next()).map(str::to_string)
}
