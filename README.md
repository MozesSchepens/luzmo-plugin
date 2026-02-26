# Luzmo HTTP Plugin -- Rust & Node.js Implementation

A modular HTTP-based Luzmo plugin implemented in Rust (Actix Web).
This project provides a fully functional custom plugin capable of:

-   Authenticating reaquests from Luzmo
-   Parsing Luzmo query payloads
-   Executing filtering and aggregation
-   Returning fully Luzmo-compatible JSON responses

The Rust implementation is the primary and actively maintained version.

------------------------------------------------------------------------

## Project Overview

This plugin implements the Luzmo HTTP connector contract and acts as a custom data source.

The execution pipeline:
1.  HTTP request received
2.  X-Secret authentication validated
3.  JSON payload parsed into structured types
4.  QueryPlan constructed
5.  Filters applied
6.  Aggregation executed (if required)
7.  Luzmo-compatible array-of-arrays response returned

The architecture is designed to be modular, maintainable and extensible.

------------------------------------------------------------------------

## Architecture (Rust Version)

src/
│
├── engine/
│   ├── execute.rs       → Orchestrates query execution
│   ├── aggregation.rs   → Grouping & aggregation logic
│   ├── filters.rs       → Filter engine
│   ├── plan.rs          → QueryPlan builder
│   └── dataset.rs       → Demo dataset & schema mapping
│
├── luzmo/
│   └── types.rs         → Luzmo request/response structs
│
├── utils/
│   ├── secret.rs        → X-Secret validation
│   ├── ids.rs           → Column ID normalization
│   └── sanitize.rs      → JSON normalization helpers
│
├── errors.rs            → Centralized error handling
├── lib.rs               → Library exports
└── main.rs              → Actix server entrypoint

------------------------------------------------------------------------

## Features

Authentication
-  X-Secret header validation

Filtering
-  equals / not equals
-  greater than / less than
-  in
-  contains
-  is null / is not null
-  Date comparisons (RFC3339 compatible)

Aggregations
-  sum
-  avg
-  min
-  max
-  count(including count (*))
-  Multi-measure support

Date Handling
-  Month bucketing support
-  RFC3339 output format

Execution
-  Raw mode (no aggregation)
-  Aggreagtion mode (group-by logic)
-  Deterministic group output
-  JSON sanitizing for Luzmo compatibility

Testing
-  Engine-level unit tests
-  Manual contract testing via PowerShell
-  End-to-end testing through Luzmo UI
-  Ngrok traffic inspection

------------------------------------------------------------------------

## Environment Configuration

Create a `.env` file:

    cp .env.example .env

Or define environment variables manually:

  -----------------------------------------------------------------------
  Variable               Required             Description
  ---------------------- -------------------- ---------------------------
  LUZMO_PLUGIN_SECRET    Yes                  Secret for X-Secret
                                              authentication

  PORT                   No                   Default: 3000

  NODE_ENV               No                   development / production
  -----------------------------------------------------------------------

------------------------------------------------------------------------

## Running the Project

### Node.js Version (Original)

Install dependencies:

    npm install

Start server:

    npm start

------------------------------------------------------------------------

### Rust Version (Main Implementation)

Make sure Rust is installed:

    rustc --version
    cargo --version

Run the server:

    cargo run

Server runs on:

    http://localhost:3000

------------------------------------------------------------------------

## API Endpoints

### POST /datasets

    curl -X POST http://localhost:3000/datasets   -H "X-Secret: dev_secret"

------------------------------------------------------------------------

### POST /query

    curl -X POST http://localhost:3000/query   -H "X-Secret: dev_secret"   -H "Content-Type: application/json"   -d '{
            "dataset_id": "demo",
            "columns": [
              {"id": "category"},
              {"id": "value", "aggregation": "sum"}
            ],
            "limit": 10
          }'

#### Response format

    [
      ["A", 123.45],
      ["B", 456.78]
    ]
No wrapper object is returned.

------------------------------------------------------------------------

### GET /health

    curl http://localhost:3000/health

Returns a simple status response.

------------------------------------------------------------------------

## Testing with ngrok

1.  Install ngrok
2.  Add your authtoken:

```{=html}
<!-- -->
```
    ngrok config add-authtoken YOUR_AUTHTOKEN

3.  Expose your local server:

```{=html}
<!-- -->
```
    ngrok http 3000

4.  Use the generated HTTPS URL as the Luzmo plugin endpoint.

------------------------------------------------------------------------
## Current Status
-  Fully functional HTTP plugin
-  Contractually compatible with Luzmo
-  Stable under normal query load
-  Modular Rust architecture
-  Extensively tested (filters, aggregations, edge cases)

------------------------------------------------------------------------
## Roadmap

Planned improvements:

-   Sorting support
-   Pagination support
-   Structured logging
-   Configurable data sources (beyond demo dataset)
-   Performance optimizations
-   Extended integration test coverage

