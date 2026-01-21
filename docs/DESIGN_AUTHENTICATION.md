# Authentication Design Notes

**Status:** Design/notes only. This document describes options and does not
imply they are implemented in the codebase.

## Goals

- Capture viable Rust/Axum/Tower auth frameworks with user/group support.
- Provide a concise comparison and recommended paths for Repeater Atlas.

## Key Concepts

- **Authentication**: identifying a user (login/session).
- **Authorization**: enforcing access control (roles, groups, permissions).
- **Session vs Token**: cookie-backed sessions (server-side state) vs JWT/bearer
  tokens (stateless).
- **Stateless JWT cookie**: signed token stored in a cookie, verified on each
  request.

## Target Approach (current preference)

- **User table** with `call_sign`, `email`, `password_hash`.
- **JWT in a signed cookie** (no server-side session store).
- **JWT claims**: `sub = call_sign`, `iat`, `exp` (7 days).
- **Authorization**: roles (if/when added) are looked up on each request.
- **Logout**: expire cookie client-side; no server-side revocation list.

## Candidate Frameworks (Axum/Tower)

### 1) axum-login (user + group permissions)

- Provides authentication/authorization middleware for Axum.
- Supports user and group permissions via `AuthzBackend`.
- Built on top of `tower-sessions`.

### 2) tower-sessions (session layer)

- Session middleware for `tower`, includes an Axum `Session` extractor.
- Pluggable session stores (Redis, SQL, filesystem, in-memory, etc.).
- Sessions load lazily when used, keeping overhead low.

### 3) tower-sessions-cookie-store (cookie-backed sessions)

- Cookie-based persistence for `tower-sessions`, with signed or encrypted
  cookies.
- Offers a signed-cookie default and a warning against plaintext cookies.

### 4) axum-auth (basic/bearer extractors)

- Lightweight HTTP Basic/Bearer auth extractors.
- Not a full user/group framework; best for simple auth or internal APIs.

### 5) axum-gate (JWT + OAuth2 + roles/groups)

- JWT-based auth with cookie or bearer support.
- Optional OAuth2 Authorization Code + PKCE flow.
- Built-in roles, groups, and permission system.

### 6) axum-casbin (authorization middleware)

- Casbin policy enforcement as Axum middleware.
- Authorization only; authentication must be implemented separately.

## Summary for Repeater Atlas

- If we want **sessions with user/group permissions**, `axum-login` +
  `tower-sessions` is the most Axum-native path.
- If we want **JWT + OAuth2** out of the box, `axum-gate` is a candidate.
- If we only need **simple HTTP auth**, `axum-auth` is the minimal option.
- If we want **policy-based authorization**, `axum-casbin` can sit on top of
  another auth system.

## Recommendation and Rationale

- **Recommended:** implement JWT cookies directly (e.g., with `jsonwebtoken`)
  rather than adopting a full framework.
- **Why:**
  - The current requirement is **stateless** JWT cookies with short, fixed
    claims (`sub`, `iat`, `exp`).
  - We **do not** want a server-side session store, which rules out
    `tower-sessions` and `axum-login`.
  - Using a full framework would add complexity without reducing core work (we
    still need DB lookups per request for roles).
  - A small, explicit extractor for JWT cookie validation keeps behavior clear
    and easy to audit.
- **Future-proofing:**
  - If OAuth2 or external IdPs are needed later, `axum-gate` becomes a stronger
    candidate.
  - If fine-grained policies are required later, `axum-casbin` can be layered on
    for authorization.
