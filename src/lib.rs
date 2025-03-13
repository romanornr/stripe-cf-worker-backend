use worker::*;
use serde::{Serialize, Deserialize};
use serde_json::json;

// Define a generic response struct to wrap any successful data
// The generic type T allows to wrap any type that elements Serialize
#[derive(Serialize)]
pub struct ApiResponse<T> {
    data: Option<T>,
    error: Option<String>,
}

// Define an error response struct to wrap error messages
#[derive(Serialize)]
pub struct ErrorResponse {
    data: Option<String>,
    error: Option<String>
}

// A helper function to create a successful JSON response
// It takes any serialize data, wraps it in our ApiResponse struct and covers it into a JSON response
pub fn success_response<T: Serialize>(data: T) -> Result<Response> {
    let resp = ApiResponse {
        data: Some(data),
        error: None,
    };
    Response::from_json(&resp)
}

pub fn error_response(message: &str, status: u16) -> Result<Response>{
    let error_obj = ErrorResponse {
        data: None,
        error: Some(message.to_string())
    };

    // Serialize the error object into a JSON string
    let json_string = serde_json::to_string(&error_obj)
        .map_err(|e| worker::Error::RustError(e.to_string()))?;

    Response::error(json_string, status)
}

// -------- Stripe Client implementation --------

#[derive(Debug, Clone)]
pub struct StripeClient {
    api_key: String,
    base_url: String,
}

impl StripeClient {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            base_url: "https://api.stripe.com/v1".to_string(),
        }
    }
}

async fn test_stripe_client() -> Result<Response> {
    let client = StripeClient::new("sk_test_4eC39HqLyjWDarjtT1zdp7dc"); // not a real api key

    // for display purpose,mask the api key by showing the first 7 digits and last 4 characters
    let key_str = client.api_key;
    let masked_key = format!("{}...{}", &key_str[..7], &key_str[key_str.len()-4..]);

    success_response(format!(
        "StripeClient created with the base URL: {} and API key: {}",
        client.base_url, masked_key
    ))
}

// A simple test endpoint to verify response helpers
// The #[event(fetch)] attribute marks this as the function that hat handles HTTP requests
#[event(fetch)]
pub async fn main(req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    let path = req.path();

    match path.as_str() {
        "/test" => success_response("Response helpers are working!"),
        "/test_stripe" => test_stripe_client().await,
        _ => error_response("Not Found", 404)
    }
}

