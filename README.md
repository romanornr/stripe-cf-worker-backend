# Stripe Terminal API for Cloudflare Workers

This project is a Rust-based Stripe Terminal API built for Cloudflare Workers using WebAssembly.

## Features

- Full Stripe Terminal integration for in-person payments
- API endpoints for creating payment intents
- Terminal connection token management
- Terminal reader management
- Support for retrieving payment history
- Built with Rust for performance and reliability
- Deployed as a Cloudflare Worker

## API Endpoints

- `/test` (GET): Verifies the Stripe key is properly configured
- `/create-payment-intent` (POST): Creates payment intents for Stripe Terminal
- `/get-recent-payment-intents` (GET): Lists the 10 most recent payment intents
- `/get-location-id` (GET): Gets the configured terminal location ID
- `/connection-token` (POST): Creates terminal connection tokens
- `/readers/id` (GET): Gets the reader ID for a location
- `/cancel-action` (POST): Cancels an in-progress reader action

## Deployment

### Prerequisites

1. [Wrangler CLI](https://developers.cloudflare.com/workers/wrangler/install-and-update/) installed
2. Rust and Cargo installed
3. wasm-pack installed (`cargo install wasm-pack`)
4. A Cloudflare account with Workers enabled

### Environment Variables

Set the following secrets in your Cloudflare Worker:

```bash
wrangler secret put STRIPE_SECRET_KEY
wrangler secret put LOCATION_ID
```

### Build and Deploy

```bash
# Build the project
wrangler build

# Deploy to Cloudflare
wrangler publish
```

## Development

### Local Development

```bash
# Run locally
wrangler dev
```

### Running Tests

```bash
cargo test
```

## Implementation Details

This project implements a custom Stripe API client designed specifically for the WebAssembly environment. Instead of relying on the async-stripe crate (which depends on tokio and isn't compatible with WebAssembly), it uses a custom implementation that interfaces directly with the Stripe REST API using the Cloudflare Workers Fetch API.

## License

MIT