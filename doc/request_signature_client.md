# Request Signature Client Implementation

## Overview

The frontend now supports signing requests using Web Crypto API for WASM compatibility. The implementation uses PBKDF2-HMAC-SHA256 with 600,000 iterations, matching the server-side validation.

## Components

### UserService (`rook_lw_admin_fe/src/services/user_service.rs`)
Manages user authentication state and credentials:
- Stores credentials in SessionStorage
- **Derives and caches signing key during login** (avoiding expensive PBKDF2 on every request)
- Provides login/logout functionality
- Retrieves cached signing key for request signing

### RequestSigner (`rook_lw_admin_fe/src/services/request_signer.rs`)
Handles cryptographic operations using Web Crypto API:
- PBKDF2 key derivation (600,000 iterations) - **done once during login**
- HMAC-SHA256 signing using cached key
- Deterministic salt generation from user_id

### Service Integration
DaemonService and ImageInfoService now accept an optional UserService:
- Automatically sign requests to `/api/*` paths
- Use **cached signing key** from UserService (no expensive derivation on each request)
- Add `X-Signature` header with base64url-encoded signature

## Usage Example

```rust
use leptos::prelude::*;
use crate::services::{UserService, DaemonService};

#[component]
pub fn MyComponent() -> impl IntoView {
    // Create user service
    let user_service = UserService::new();
    
    // Login (derives signing key once and caches it)
    spawn_local(async move {
        match user_service.login("admin".to_string(), "password".to_string()).await {
            Ok(_) => println!("Logged in! Signing key cached."),
            Err(e) => println!("Login failed: {}", e),
        }
    });
    
    // Create daemon service with user authentication
    let daemon_service = DaemonService::new("http://localhost:8080")
        .with_user_service(user_service.clone());
    
    // Make authenticated requests - uses cached signing key (fast!)
    spawn_local(async move {
        match daemon_service.status().await {
            Ok(Some(info)) => println!("Daemon running: PID {}", info.pid),
            Ok(None) => println!("Daemon not running"),
            Err(e) => println!("Error: {}", e),
        }
    });
    
    view! { <div>"My Component"</div> }
}
```

## Security Features

1. **PBKDF2 Key Derivation**: 600,000 iterations makes brute-force attacks computationally expensive
2. **Key Caching**: Signing key derived once during login and cached in SessionStorage (base64-encoded)
3. **Deterministic Salt**: Uses `SHA256("rook_lw_user_salt_" + user_id)` for consistent salting
4. **Request Integrity**: Signs `METHOD|URL|TIMESTAMP|SALT|BODY` format
5. **SessionStorage**: Credentials and signing key persist across page reloads but not across browser sessions
6. **HMAC-SHA256**: Strong message authentication prevents tampering

## Request Signature Format

Each signed request includes:
- **X-Signature header**: `{timestamp}.{salt}.{signature_base64url}`
- **timestamp**: Unix timestamp in seconds
- **salt**: Base64url-encoded random salt
- **signature**: HMAC-SHA256 of `METHOD|URL|TIMESTAMP|SALT|BODY`

Example:
```
X-Signature: 1704123456.abc123def456.xyz789signature
```

## Web Crypto API Details

The implementation uses browser's SubtleCrypto API:
- `importKey()` - Import password and HMAC key
- `deriveBits()` - PBKDF2 key derivation
- `sign()` - HMAC-SHA256 signing

All operations are:
- Asynchronous (returns Promises converted to Futures)
- Hardware-accelerated when available
- Secure (keys never exposed to JavaScript)

## Migration Notes

Services without user authentication will continue to work unchanged. To add authentication:

```rust
// Before
let service = DaemonService::new("http://localhost:8080");

// After
let user_service = UserService::new();
// ... login ...
let service = DaemonService::new("http://localhost:8080")
    .with_user_service(user_service);
```

## Testing

To test the implementation:

1. Start the backend with signature validation enabled
2. Login with valid credentials
3. Make API requests - should succeed with valid signatures
4. Try without login - requests should fail with 401 Unauthorized
5. Check browser DevTools Network tab for X-Signature headers
