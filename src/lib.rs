use worker::*;
use serde::Serialize;

#[derive(Serialize)]
struct ApiResponse<T> {
    data: Option<T>,
    error: Option<String>,
}

#[derive(Serialize)]
struct ErrorResponse {
    data: Option<String>,
    error: Option<String>,
}

fn success_response<T: Serialize>(data: T) -> Result<Response> {
    let resp = ApiResponse {
        data: Some(data),
        error: None,
    };
    Response::from_json(&resp)
}

fn error_response(message: &str, status: u16) -> Result<Response> {
    let error_obj = ErrorResponse {
        data: None,
        error: Some(message.to_string()),
    };

    let json_string = serde_json::to_string(&error_obj)
        .map_err(|e| worker::Error::RustError(e.to_string()))?;

    Response::error(json_string, status)
}

struct StripeClient {
    api_key: String,
    base_url: String,
}

impl StripeClient {
    // creates a new StripeClient with the given api_key
    fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            base_url: "https://api.stripe.com/v1".to_string(),
        }
    }
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    if req.path() == "/test" {
        return test_endpoint(env).await;
    }

    Response::error("Not Found", 404)
}

async fn test_endpoint(env: Env) -> Result<Response> {
    match env.secret("STRIPE_SECRET_KEY") {
        Ok(key) => {
            let key_str = key.to_string();
            // We mask the key: Show first 7 and 4 characters of the key
            let safe_key = format!("{}...{}", &key_str[..7], &key_str[key_str.len()-4..]);
            //Response::ok(&format!("Env loaded! Key: {}", safe_key))
            success_response(format!("Env loaded! Key: {}", safe_key))
        },
        Err(e) => Response::error(&format!("Failed to load Stripe secret key: {}", e ), 500)
    }
}
