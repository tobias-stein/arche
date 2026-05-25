---
title: CORS — service rejects cross-origin requests from admin UI
status: completed
---

## Summary

The arche-service does not emit CORS headers, so browsers block API requests from the admin UI when served from a different origin (e.g., Nginx on `localhost:3000` → service on `localhost:8080`).

The `tower-http` dependency already includes the `cors` feature, but `CorsLayer` is never added to the router.

## Acceptance criteria

- [ ] `OPTIONS` preflight requests return `200` with `Access-Control-Allow-Origin: *` (or the specific UI origin)
- [ ] Actual API responses include `Access-Control-Allow-Origin` headers
- [ ] Admin UI can fetch from `http://localhost:8080` when served from `http://localhost:3000`
- [ ] Native dev mode (Vite on `localhost:5173`) also works

## Notes

The fix was applied in `crates/arche-service/src/main.rs` by adding `.layer(CorsLayer::permissive())` as the outermost layer. For production, this should be restricted to known origins.
