use axum::{Router, http::StatusCode, routing::get};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Listening on http://{}", listener.local_addr()?);

    axum::serve(listener, router())
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to listen for Ctrl-C");
        })
        .await
}

fn router() -> Router {
    Router::new().route("/healthz", get(|| async { StatusCode::OK }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    #[tokio::test]
    async fn healthz_returns_ok_with_empty_body() {
        let response = router()
            .oneshot(Request::get("/healthz").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), 0).await.unwrap();
        assert!(body.is_empty());
    }
}
