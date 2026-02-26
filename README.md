# Luzmo HTTP Plugin -- Rust & Node.js Implementation

Minimal Luzmo plugin implementation with improved error handling,
structured query execution, and modular architecture.

This repository contains two implementations:

-   Original HTTP plugin (Node.js)
-   Refactored & production-ready version (Rust -- Actix Web)

The Rust implementation is the main and actively maintained version.

------------------------------------------------------------------------

## Project Overview

This project implements a custom HTTP-based Luzmo plugin that:

-   Authenticates incoming requests
-   Parses Luzmo query payloads
-   Builds a structured QueryPlan
-   Executes filtering and aggregation
-   Returns Luzmo-compatible JSON responses

The Rust version introduces a modular architecture for maintainability
and scalability.

------------------------------------------------------------------------

## Architecture (Rust Version)

    src/
    │
    ├── engine/     → Query processing & execution logic
    ├── server/     → HTTP layer (Actix endpoints)
    ├── utils/      → Shared helpers (auth, ids, sanitizing)
    ├── errors/     → Centralized error handling
    ├── luzmo/      → Request/response types
    └── main.rs     → Application entrypoint

### Execution Flow

1.  HTTP request received
2.  X-Secret authentication validated
3.  JSON body parsed
4.  QueryPlan constructed
5.  Filtering & aggregation executed
6.  Luzmo-compatible JSON response returned

------------------------------------------------------------------------

## Features (Rust Version)

-   X-Secret header authentication
-   QueryPlan-based execution pipeline
-   Filtering support:
    -   equals
    -   not equals
    -   greater than / less than
    -   contains
    -   in
    -   is not null
-   Aggregations:
    -   sum
    -   avg
    -   min
    -   max
    -   count (including `count(*)`)
-   Month bucketing for date fields
-   Deterministic grouping output
-   JSON sanitizing
-   Health endpoint with timestamp
-   Manual API testing via localhost or ngrok

------------------------------------------------------------------------

## Environment Configuration

Create a `.env` file:

    cp .env.example .env

Or define environment variables manually:

  -----------------------------------------------------------------------
  Variable               Required             Description
  ---------------------- -------------------- ---------------------------
  LUZMO_PLUGIN_SECRET    Yes (production)     Secret for X-Secret
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

    {
      "Count": 10,
      "value": [...]
    }

------------------------------------------------------------------------

### GET /health

    curl http://localhost:3000/health

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

4.  Use the HTTPS URL in Luzmo as plugin endpoint

------------------------------------------------------------------------

## Roadmap

Planned improvements for the Rust implementation:

-   Unit tests for engine layer
-   Integration tests for query endpoint
-   Structured logging
-   Configurable dataset sources
-   Sorting support
-   QueryPlan validation improvements
-   Pagination support
-   Performance benchmarks

------------------------------------------------------------------------

## Status
Functional HTTP plugin\
Modular Rust architecture\
Luzmo-compatible responses\
Ongoing improvements
