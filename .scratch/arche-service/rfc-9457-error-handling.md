---
title: RFC 9457 error handling
status: ready-for-agent
---

## Parent

Arche API Service PRD (see `.scratch/arche-service/prd.md`)

## What to build

Implement consistent error handling across all endpoints using RFC 9457 Problem JSON.

Define a `ProblemResponse` struct:
```json
{
  "type": "/errors/validation-error",
  "title": "Validation Error",
  "status": 400,
  "detail": "attribute 'damage': range min (25.0) exceeds max (10.0)",
  "instance": null,
  "errors": [
    { "path": "attributes.damage", "message": "min must be <= max" }
  ]
}
```

Implement `axum::response::IntoResponse` for `ProblemResponse`. The `type` URI is relative to the API base (not a real document).

Standard error types per the API spec:
- 400: `/errors/validation-error`
- 401: `/errors/unauthorized`
- 403: `/errors/forbidden`
- 404: `/errors/not-found`, `/errors/no-matching-blueprints`
- 409: `/errors/delete-referenced-resource`, `/errors/import-conflict`
- 422: `/errors/unprocessable-entity`

Create a helper `AppResult<T>` type alias and `AppError` enum that implements `IntoResponse` for ergonomic handler signatures. The `errors` array is optional and provides per-field error details.

## Acceptance criteria

- [ ] `ProblemResponse` struct with all RFC 9457 fields
- [ ] `AppError` enum maps to correct status codes and type URIs
- [ ] `IntoResponse` produces correct JSON body and Content-Type header
- [ ] Validation errors include `errors` array with per-field details
- [ ] All standard error statuses from API spec are covered
- [ ] Unit tests for error serialization

## Blocked by

- arche-service/service-scaffolding.md
