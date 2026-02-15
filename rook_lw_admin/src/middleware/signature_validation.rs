use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse, Payload},
    Error, HttpMessage, HttpRequest,
    middleware::Next,
    web::Bytes,
};
use crate::app::AppState;
use crate::services::RequestSignatureService;
use rook_lw_models::user::{RequestSignature, User};
use actix_web::web::Data;
use tracing::{debug, warn, info};
use std::pin::Pin;
use futures_util::stream;
use futures_util::StreamExt;

/// Header name for request signatures
pub const SIGNATURE_HEADER: &str = "X-Signature";

/// Query parameter name for signatures (used for image URLs)
pub const SIGNATURE_QUERY_PARAM: &str = "sig";

/// Helper function to extract authenticated user from request extensions
/// 
/// # Example
/// ```ignore
/// use actix_web::{HttpRequest, HttpResponse};
/// use rook_lw_admin::middleware::get_authenticated_user;
/// 
/// async fn handler(req: HttpRequest) -> HttpResponse {
///     match get_authenticated_user(&req) {
///         Some(user) => HttpResponse::Ok().body(format!("Hello, {}!", user.name)),
///         None => HttpResponse::Unauthorized().body("Unauthorized"),
///     }
/// }
/// ```
pub fn get_authenticated_user(req: &HttpRequest) -> Option<User> {
    req.extensions().get::<User>().cloned()
}

/// Configuration for signature validation middleware
#[derive(Clone)]
pub struct SignatureValidationConfig {
    /// Path patterns that require signature validation (e.g., "/api")
    pub protected_paths: Vec<String>,
    /// Maximum body size in bytes (default: 10MB)
    pub max_body_size: usize,
}

impl SignatureValidationConfig {
    pub fn new(protected_paths: Vec<String>) -> Self {
        Self { 
            protected_paths,
            max_body_size: 10 * 1024 * 1024, // 10MB default
        }
    }
    
    pub fn with_max_body_size(mut self, max_body_size: usize) -> Self {
        self.max_body_size = max_body_size;
        self
    }
    
    /// Check if a path should be protected
    pub fn should_validate(&self, path: &str) -> bool {
        self.protected_paths.iter().any(|pattern| {
            if pattern.ends_with("/*") {
                let prefix = &pattern[..pattern.len() - 2];
                path.starts_with(prefix)
            } else {
                path == pattern
            }
        })
    }
}

/// Extract signature from request header or query parameter
fn extract_signature(req: &ServiceRequest) -> Result<RequestSignature, Error> {
    if let Some(sig_header) = req.headers().get(SIGNATURE_HEADER) {
        debug!("Found signature in header");
        let sig_str = sig_header.to_str().map_err(|_| {
            warn!("Invalid signature header encoding");
            actix_web::error::ErrorUnauthorized("Invalid authentication signature")
        })?;
        RequestSignature::from_base64url(sig_str)
            .map_err(|e| {
                warn!("Invalid signature format: {}", e);
                actix_web::error::ErrorUnauthorized("Invalid authentication signature")
            })
    } else if let Some(sig_query) = req.query_string().split('&')
        .find(|p| p.starts_with(&format!("{}=", SIGNATURE_QUERY_PARAM)))
        .and_then(|p| p.strip_prefix(&format!("{}=", SIGNATURE_QUERY_PARAM)))
    {
        debug!("Found signature in query parameter");
        let decoded = urlencoding::decode(sig_query)
            .map_err(|_| {
                warn!("Invalid URL encoding in signature parameter");
                actix_web::error::ErrorUnauthorized("Invalid authentication signature")
            })?;
        RequestSignature::from_base64url(&decoded)
            .map_err(|e| {
                warn!("Invalid signature format: {}", e);
                actix_web::error::ErrorUnauthorized("Invalid authentication signature")
            })
    } else {
        warn!("No signature found for protected path: {}", req.path());
        Err(actix_web::error::ErrorUnauthorized("Missing authentication signature"))
    }
}

/// Build URL for signature verification, excluding the signature parameter
fn build_verification_url(req: &ServiceRequest) -> String {
    if req.query_string().is_empty() {
        format!("{}", req.uri().path())
    } else {
        let clean_query = req.query_string()
            .split('&')
            .filter(|p| !p.starts_with(&format!("{}=", SIGNATURE_QUERY_PARAM)))
            .collect::<Vec<_>>()
            .join("&");
        if clean_query.is_empty() {
            format!("{}", req.uri().path())
        } else {
            format!("{}?{}", req.uri().path(), clean_query)
        }
    }
}

/// Buffer the request body by reading all chunks from the payload
async fn buffer_request_body(mut payload: Payload, max_size: usize) -> Result<Bytes, Error> {
    let mut body_bytes = actix_web::web::BytesMut::new();
    let mut total_size = 0;
    
    while let Some(chunk) = payload.next().await {
        let chunk = chunk.map_err(|e| {
            warn!("Failed to read request body chunk: {}", e);
            actix_web::error::ErrorBadRequest("Failed to read request body")
        })?;
        
        total_size += chunk.len();
        if total_size > max_size {
            warn!("Request body exceeds maximum size limit: {} > {}", total_size, max_size);
            return Err(actix_web::error::ErrorPayloadTooLarge("Request body too large"));
        }
        
        body_bytes.extend_from_slice(&chunk);
    }
    Ok(body_bytes.freeze())
}

/// Reconstruct a ServiceRequest with a buffered body
fn reconstruct_request(http_req: HttpRequest, body_bytes: Bytes) -> ServiceRequest {
    let body_stream = stream::once(async move { Ok::<_, actix_web::error::PayloadError>(body_bytes) });
    let boxed_stream: Pin<Box<dyn futures_util::Stream<Item = Result<Bytes, actix_web::error::PayloadError>> + 'static>> = 
        Box::pin(body_stream.map(|res| res.map(Bytes::from)));
    let payload = Payload::Stream {
        payload: boxed_stream,
    };
    ServiceRequest::from_parts(http_req, payload)
}

/// Creates signature validation middleware for the given configuration
/// 
/// This middleware validates request signatures using HMAC-SHA256. It can extract
/// signatures from the `X-Signature` header or the `sig` query parameter.
/// 
/// # Usage Example
/// 
/// ```ignore
/// use actix_web::{web, App, HttpServer};
/// use rook_lw_admin::middleware::{signature_validation, SignatureValidationConfig};
/// 
/// let config = SignatureValidationConfig::new(vec!["/api/*".to_string()]);
/// 
/// HttpServer::new(move || {
///     let cfg = config.clone();
///     App::new()
///         .app_data(web::Data::new(app_state.clone()))
///         // Apply signature validation to /api/* paths
///         .wrap(middleware::from_fn(move |req, next| {
///             signature_validation(cfg.clone(), req, next)
///         }))
///         .service(web::scope("/api")
///             .configure(api_routes)
///         )
/// })
/// # ;
/// ```
/// 
/// # Accessing User in Handlers
/// 
/// ```ignore
/// use actix_web::{HttpRequest, HttpResponse};
/// use rook_lw_admin::middleware::get_authenticated_user;
/// 
/// async fn protected_handler(req: HttpRequest) -> HttpResponse {
///     match get_authenticated_user(&req) {
///         Some(user) => HttpResponse::Ok().body(format!("Hello, {}!", user.name)),
///         None => HttpResponse::Unauthorized().body("Unauthorized"),
///     }
/// }
/// ```
pub async fn signature_validation(
    config: SignatureValidationConfig,
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    // Check if this path should be validated
    if !config.should_validate(req.path()) {
        info!("Path {} does not require signature validation", req.path());
        return next.call(req).await;
    }

    info!("Validating signature for path: {}", req.path());

    // Extract signature from header or query parameter
    let request_sig = extract_signature(&req)?;

    // Get app state and look up user
    let app_state = req.app_data::<Data<AppState>>()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("App state not found"))?;

    let user = app_state.user_repo.get_user(&request_sig.user_id)
        .map_err(|e| {
            warn!("Database error looking up user: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?
        .ok_or_else(|| {
            warn!("User not found: {}", request_sig.user_id);
            actix_web::error::ErrorUnauthorized("Invalid authentication signature")
        })?;

    // Build URL for verification
    let url = build_verification_url(&req);

    // Extract and buffer the request body
    let (http_req, payload) = req.into_parts();
    let body_bytes = buffer_request_body(payload, config.max_body_size).await?;
    
    debug!("Body size: {} bytes", body_bytes.len());

    // Verify signature
    let is_valid = RequestSignatureService::verify_signature(
        &user,
        &request_sig,
        http_req.method().as_str(),
        &url,
        &body_bytes,
    )
    .map_err(|e| {
        warn!("Signature verification error: {}", e);
        actix_web::error::ErrorInternalServerError("Signature verification error")
    })?;

    if !is_valid {
        warn!("Invalid signature for user: {}", user.name);
        return Err(actix_web::error::ErrorUnauthorized("Invalid authentication signature"));
    }

    debug!("Signature valid for user: {}", user.name);

    // Store user in request extensions and reconstruct request
    http_req.extensions_mut().insert(user);
    let req = reconstruct_request(http_req, body_bytes);

    next.call(req).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_validation_config_wildcard() {
        let config = SignatureValidationConfig::new(vec!["/api/*".to_string()]);
        
        assert!(config.should_validate("/api/images"));
        assert!(config.should_validate("/api/users"));
        assert!(config.should_validate("/api/"));
        assert!(!config.should_validate("/public/images"));
        assert!(!config.should_validate("/images"));
    }

    #[test]
    fn test_signature_validation_config_exact() {
        let config = SignatureValidationConfig::new(vec!["/api".to_string()]);
        
        assert!(config.should_validate("/api"));
        assert!(!config.should_validate("/api/images"));
        assert!(!config.should_validate("/public"));
    }

    #[test]
    fn test_signature_validation_config_multiple() {
        let config = SignatureValidationConfig::new(vec![
            "/api/*".to_string(),
            "/admin/*".to_string(),
        ]);
        
        assert!(config.should_validate("/api/images"));
        assert!(config.should_validate("/admin/users"));
        assert!(!config.should_validate("/public/images"));
    }
}
