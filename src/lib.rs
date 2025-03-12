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

// A simple test endpoint to verify response helpers
// The #[event(fetch)] attribute marks this as the function that hat handles HTTP requests
#[event(fetch)]
pub async fn main(_req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    // we return a success response with a simple message
    success_response("Response helpers are working!")
}

