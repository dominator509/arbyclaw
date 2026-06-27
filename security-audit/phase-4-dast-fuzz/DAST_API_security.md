# Dynamic Application Security Testing (DAST) & API Security

## Scope
*Target:* Outbound HTTP/RPC clients and future embedded dashboard servers.
*Status:* **MOCKED / BYPASS** (Application does not currently expose listening ports per Phase 13/14 boundaries).

## API Security (REST / GraphQL / gRPC) Mock Tests
If the application exposed endpoints, the following dynamic tests would be executed via tools like OWASP ZAP or Burp Suite Professional:

1. **Mass Assignment Testing:**
   - Injecting unauthorized fields (e.g., `{"is_admin": true}`) into JSON payloads.
2. **SSRF (Server-Side Request Forgery):**
   - Attempting to force the CEX/DEX connectors to query internal metadata services (e.g., AWS IMDSv2 `169.254.169.254`).
3. **GraphQL Introspection & Depth Limitation:**
   - Fuzzing for deeply nested queries that could cause CPU exhaustion / DoS.
4. **Rate Limit Evasion:**
   - Testing `X-Forwarded-For` and `X-Real-IP` spoofing to bypass local rate limits.

## Coverage-Guided Fuzzing Strategy
- Integrating `cargo-fuzz` into the CI/CD pipeline.
- Continuous mutation of market data quotes (Order Book updates) to ensure deserializers do not panic on unexpected token symbols, negative prices, or integer overflows.
