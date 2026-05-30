# Cryptography, Identity, and Access Control Test Suite

## 1. Authentication & MFA Testing
*Target:* Future Dashboard and API endpoints.
*Status:* Currently **MOCKED** (No HTTP/Web server deployed per Phase 13 boundaries).

**Test Cases:**
- `test_auth_rejects_weak_passwords`: Ensure password entropy limits are enforced.
- `test_auth_jwt_signature_validation`: Verify JWTs are rejected if signed with the `None` algorithm or an invalid key.
- `test_auth_mfa_totp_enforcement`: Ensure critical actions (e.g., config changes, manual trades) require a valid TOTP token.
- `test_auth_session_timeout`: Verify session invalidation after 15 minutes of inactivity.

## 2. Authorization & RBAC
*Target:* Policy Engine and CLI/Communications routing.

**Test Cases:**
- `test_rbac_read_only_agent`: Verify an agent tagged `role: observer` cannot execute `ExecutionAdapter::submit()`.
- `test_rbac_privilege_escalation`: Attempt to mutate the `role` parameter via API payload injection; expect explicit rejection by deserializer.
- `test_policy_deny_by_default`: Pass an empty or undefined policy context to the Policy Engine; expect `Decision::Deny`.

## 3. Cryptography & Key Management
*Target:* Wallet signers, TLS configuration, and local SQLite state encryption.

**Test Cases:**
- `test_tls_min_version`: Ensure outbound REST clients (e.g., reqwest) strictly require TLS 1.3.
- `test_tls_cipher_suites`: Reject connections offering weak ciphers (e.g., RC4, 3DES, CBC modes).
- `test_key_management_in_memory`: Verify that Web3 private keys, when loaded from secure storage (e.g., AWS KMS or Hashicorp Vault), are wrapped in `secrecy::Secret` or zeroized on drop.
- `test_sqlite_encryption`: Verify that local WAL states containing sensitive execution drafts are encrypted at rest using `sqlcipher`.

## Conclusion
The core repository strictly relies on Rust's type system and explicit boundaries. Future implementations of live Web servers and Signers must pass these tests before Production Readiness.
