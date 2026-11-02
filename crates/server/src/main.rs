mod auth;

use auth::ManagementToken;
use axum::{Router, http::StatusCode, middleware, routing::get};
use std::path::Path;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let token_file = std::env::var_os("OPEN_OMAHA_TOKEN_FILE").ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "OPEN_OMAHA_TOKEN_FILE is required",
        )
    })?;
    let token = ManagementToken::load(Path::new(&token_file))?;
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Listening on http://{}", listener.local_addr()?);

    axum::serve(listener, router(token))
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to listen for Ctrl-C");
        })
        .await
}

fn router(token: ManagementToken) -> Router {
    let management = Router::new()
        .fallback(|| async { StatusCode::NOT_FOUND })
        .layer(middleware::from_fn_with_state(token, auth::require_token));
    Router::new()
        .route("/healthz", get(|| async { StatusCode::OK }))
        .nest_service("/v1", management)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    const TOKEN: &str = "test-only-secret-0123456789";

    fn test_router() -> Router {
        router(ManagementToken::parse(TOKEN).unwrap())
    }

    #[tokio::test]
    async fn healthz_returns_ok_with_empty_body() {
        let response = test_router()
            .oneshot(Request::get("/healthz").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), 0).await.unwrap();
        assert!(body.is_empty());
    }

    #[tokio::test]
    async fn management_requests_require_a_valid_bearer_token() {
        for authorization in [
            None,
            Some("Bearer incorrect"),
            Some("Basic test-only-secret-0123456789"),
        ] {
            let mut request = Request::get("/v1/applications");
            if let Some(value) = authorization {
                request = request.header(axum::http::header::AUTHORIZATION, value);
            }
            let response = test_router()
                .oneshot(request.body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
            assert_eq!(
                response.headers()[axum::http::header::WWW_AUTHENTICATE],
                "Bearer"
            );
            assert!(
                axum::body::to_bytes(response.into_body(), 0)
                    .await
                    .unwrap()
                    .is_empty()
            );
        }
    }

    #[tokio::test]
    async fn valid_token_reaches_management_router() {
        for scheme in ["Bearer", "bearer"] {
            let request = Request::get("/v1/applications")
                .header(
                    axum::http::header::AUTHORIZATION,
                    format!("{scheme} {TOKEN}"),
                )
                .body(Body::empty())
                .unwrap();
            let response = test_router().oneshot(request).await.unwrap();
            // The management namespace is protected but has no endpoints yet.
            assert_eq!(response.status(), StatusCode::NOT_FOUND);
        }
    }

    #[tokio::test]
    async fn duplicate_authorization_headers_are_rejected() {
        let request = Request::get("/v1/applications")
            .header(axum::http::header::AUTHORIZATION, format!("Bearer {TOKEN}"))
            .header(axum::http::header::AUTHORIZATION, format!("Bearer {TOKEN}"))
            .body(Body::empty())
            .unwrap();
        let response = test_router().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn management_namespace_roots_require_authentication() {
        for path in ["/v1", "/v1/"] {
            let request = Request::get(path).body(Body::empty()).unwrap();
            let response = test_router().oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "{path}");
        }
    }
}
