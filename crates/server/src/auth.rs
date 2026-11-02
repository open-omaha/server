//! Bearer-token authentication for the management API.

use axum::{
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::{io, path::Path, sync::Arc};
use subtle::ConstantTimeEq;

// Deliberately has no Debug implementation: the value is a credential.
#[derive(Clone)]
pub(crate) struct ManagementToken(Arc<str>);

impl ManagementToken {
    pub(crate) fn load(path: &Path) -> io::Result<Self> {
        Self::parse(&std::fs::read_to_string(path)?)
    }

    pub(crate) fn parse(text: &str) -> io::Result<Self> {
        let secret = text.trim_end_matches(['\r', '\n']);
        if secret.len() < 16 || !secret.bytes().all(|b| b.is_ascii_graphic()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "token file must contain one token of at least 16 printable ASCII bytes without whitespace",
            ));
        }
        Ok(Self(Arc::from(secret)))
    }
}

pub(crate) async fn require_token(
    State(token): State<ManagementToken>,
    request: Request,
    next: Next,
) -> Response {
    let headers = request.headers();
    let supplied = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split_once(' '))
        .filter(|(scheme, _)| scheme.eq_ignore_ascii_case("Bearer"))
        .map(|(_, secret)| secret);
    let valid = headers.get_all(header::AUTHORIZATION).iter().count() == 1
        && supplied.is_some_and(|secret| bool::from(token.0.as_bytes().ct_eq(secret.as_bytes())));
    if valid {
        next.run(request).await
    } else {
        (
            StatusCode::UNAUTHORIZED,
            [(header::WWW_AUTHENTICATE, "Bearer")],
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_token_file_with_trailing_newline() {
        let file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(file.path(), "test-only-secret-0123456789\n").unwrap();
        let token = ManagementToken::load(file.path()).unwrap();
        assert!(token.0.as_ref() == "test-only-secret-0123456789");
    }

    #[test]
    fn rejects_invalid_token_configuration_without_echoing_it() {
        for text in [
            "",
            "short",
            "                ",
            "secret with whitespace",
            "first-token-012345\nsecond-token-012345",
            "non-ascii-secret-é",
        ] {
            let file = tempfile::NamedTempFile::new().unwrap();
            std::fs::write(file.path(), text).unwrap();
            let error = ManagementToken::load(file.path())
                .err()
                .expect("invalid token accepted");
            assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
            if !text.is_empty() {
                assert!(!error.to_string().contains(text));
            }
        }
    }

    #[test]
    fn rejects_missing_token_file() {
        let directory = tempfile::tempdir().unwrap();
        let error = ManagementToken::load(&directory.path().join("missing"))
            .err()
            .expect("missing token file accepted");
        assert_eq!(error.kind(), io::ErrorKind::NotFound);
    }
}
