use std::collections::HashMap;
use std::fmt::format;
use worker::*;
use serde::{Serialize, Deserialize};
use http::StatusCode;
use serde_json::json;
// make sure to import the http crate


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

// -------- Stripe data structures --------

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePaymentIntentRequest {
    pub amount: i64,
    pub currency: String,
    // These fields are optional, so we only include them if provided
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_method_types: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capture_method: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentIntent {
    pub id: String,
    pub object: String,
    pub amount: i64,
    pub currency: String,
    pub status: String,
    pub client_secret: String,
    pub capture_method: String,
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

    // Send a post request to the specified Stripe API path with form-encoded data
    pub async fn post<T: Serialize, U: for<'de> Deserialize<'de>>(&self, path: &str, data: &T, ) -> worker::Result<U> {
    // Build the full URL by concatenating the base URL with the provided path
    let url = format!("{}{}", self.base_url, path);

    // create ans setup the headers. Stripe requires an Authorization header and the content type to be "application/x-www-form-urlencoded"
    let mut headers = Headers::new();
    headers.set("Authorization", &format!("Bearer {}", self.api_key))?; // set the Authorization header
    headers.set("Content-Type", "application/x-www-form-urlencoded")?; // set the content type header

    let mut init = RequestInit::new();
    init.with_method(Method::Post)
        .with_headers(headers);

    // Serialize the data into the URL-encoded form parameters
    let form_data = serde_urlencoded::to_string(data)
        .map_err(|e| worker::Error::RustError(e.to_string()))?;

    console_log!("Sending POST request to {} with data: {}", url, form_data);

    // Attach the serialized data as the body of the request
    init.with_body(Some(form_data.into()));

    // create teh request and set it
    let req = Request::new_with_init(&url, &init)?;
    let mut resp = Fetch::Request(req).send().await?;

    // Retrieve the HTTP status code and the response body as text
    let status = resp.status_code();
    let json_text = resp.text().await?;

    console_log!("Response status code: {}, body:{}", status, json_text);

    // if the response status indicates success (200 - 299)
    // try to describe the response JSON into the expected type U
    if (200..300).contains(&status) {
        serde_json::from_str::<U>(&json_text).map_err(|e| worker::Error::RustError(format!("JSON parse error: {}", e)))
    } else {
        Err(worker::Error::RustError(format!("Stripe API error (status {}): {}", status, json_text)))
    }
  }

    pub async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str, query_params: Option<HashMap<String, String>>) -> worker::Result<T> {
        // Parse the base URL from the client's base_url field and join it with the provided path
        let mut url = Url::parse(&self.base_url)
            .and_then(|base| base.join(path))
            .map_err(|e|worker::Error::RustError(e.to_string()))?;

        // Append query parameters, if any, using the URL's query_pairs mut method
        if let Some(params) = query_params {
            let mut pairs = url.query_pairs_mut();
            for (key, value) in params {
                pairs.append_pair(&key, &value);
            }
        }
        console_log!("Get request to {}", url.as_str());

        // Set up the necessary headers for the request
        let mut headers = Headers::new();
        headers.set("Authorization", &format!("Bearer {}", self.api_key))?;

        // Initialize the request with the GET method and headers
        let mut init = RequestInit::new();
        init.with_method(Method::Get)
            .with_headers(headers);

        // Create a new request using the constructed URL and initialization options
        let req = Request::new_with_init(url.as_str(), &init)?;
        // Send the request asynchronously and await the response
        let mut resp = Fetch::Request(req).send().await?;

        // Retrieve the HTTP status code and response body
        let status = resp.status_code();
        let json_text = resp.text().await?;
        console_log!("Response status code {}, body: {}", status, json_text);

        // Convert raw status code into a StatusCode type for checking success
        let status_code = StatusCode::from_u16(status)
            .map_err(|e| worker::Error::RustError(format!("Invalid status code: {}", e)))?;

        // If the status is success, deserialize the JSON text into type T
        // Otherwise return an error with the status code and response text
        match status_code.is_success() {
            true => serde_json::from_str::<T>(&json_text)
                .map_err(|e| worker::Error::RustError(format!("JSON parse error {}", e))),
            false => Err(worker::Error::RustError(format!("Stripe API error (status {}): {}", status, json_text)))
        }
    }
}

async fn create_payment_intent(env: Env, mut req: Request) -> Result<Response> {
    // Load the Stripe secret API key from the environment secrets
    let stripe_key = match env.secret("STRIPE_SECRET_KEY") {
        Ok(key) => key.to_string(),
        Err(e) => return error_response(&format!("Failed to load Stripe secret key: {}", e), 500),
    };

    // Create an instance of StripeClient using the API key
    let stripe_client = StripeClient::new(&stripe_key);

    // Deserialize the incoming JSON into our CreatePaymentIntentRequest struct
    let request_data = match req.json::<CreatePaymentIntentRequest>().await {
        Ok(data) => data,
        Err(e) => return error_response(&format!("Invalid request data: {}", e), 400),
    };

    // Build the paramaters for the Stripe API request
    // We use a Hashmap to build the URL-encoded from parameters
    let mut params = std::collections::HashMap::new();
    params.insert("amount".to_string(), request_data.amount.to_string());
    params.insert("currency".to_string(), request_data.currency.clone());
    // Stripe expects "payment_method_types[]" as a key when sending payment
    params.insert("payment_method_types[]".to_string(), "card".to_string());
    // Set capture method to "automatic" adjust as needed
    params.insert("capture_method".to_string(), "automatic".to_string());

    console_log!("Payment intent request parameters: {:?}", params);

    // send a POST request to Stripe's /payment_intents endpoint
    match stripe_client.post::<_, PaymentIntent>("/payment_intents", &params).await {
        Ok(payment_intent) => {
            // on success, return the paymentIntent into a JSON success response
            success_response(payment_intent)
        },
        Err(e) => {
            // Log the error and return an error response
            console_error!("Error creating payment intent: {}", e);
            error_response(&format!("Failed to create a payment intent: {}", e), 500)
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
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let path = req.path();

    match path.as_str() {
        "/test" => success_response("Response helpers are working!"),
        "/test_stripe" => test_stripe_client().await,
        "/create-payment-intent" => create_payment_intent(env, req).await,
        _ => error_response("Not Found", 404)
    }
}

