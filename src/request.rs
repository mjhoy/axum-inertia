use async_trait::async_trait;
use axum::extract::FromRequestParts;
use http::{request::Parts, HeaderMap, HeaderValue, StatusCode};

/// Inertia-related information in the request.
///
/// See more info here: https://inertiajs.com/the-protocol.
#[derive(Clone, Debug)]
pub(crate) struct Request {
    pub(crate) is_xhr: bool,
    pub(crate) version: Option<String>,
    pub(crate) url: String,
}

impl Request {
    #[cfg(test)]
    pub(crate) fn test_request() -> Request {
        Request {
            is_xhr: true,
            version: None,
            url: "/foo/bar".to_string(),
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Request
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, HeaderMap<HeaderValue>);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let url = parts.uri.path().to_string();
        let is_xhr = parts
            .headers
            .get("X-Inertia")
            .map(|s| s.to_str().map(|s| s == "true"))
            .transpose()
            .map_err(|_err| (StatusCode::BAD_REQUEST, HeaderMap::new()))?
            .unwrap_or(false);
        let version = parts
            .headers
            .get("X-Inertia-Version")
            .map(|s| s.to_str().map(|s| s.to_string()))
            .transpose()
            .map_err(|_err| (StatusCode::BAD_REQUEST, HeaderMap::new()))?;
        Ok(Request {
            is_xhr,
            version,
            url,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{self, routing::get, Router, Server};
    use reqwest::StatusCode;
    use std::net::TcpListener;

    #[tokio::test]
    async fn it_extracts_inertia_request_info() {
        async fn handler_expect_inertia(req: Request) {
            assert!(req.is_xhr);
            assert_eq!(req.version, Some("required".to_string()))
        }
        async fn handler_expect_not_inertia(req: Request) {
            assert!(!req.is_xhr);
            assert_eq!(req.version, Some("not-required".to_string()))
        }
        async fn handler_expect_no_version(req: Request) {
            assert_eq!(req.version, None)
        }

        let app = Router::new()
            .route("/expect_inertia", get(handler_expect_inertia))
            .route("/expect_not_inertia", get(handler_expect_not_inertia))
            .route("/expect_no_version", get(handler_expect_no_version));

        let listener = TcpListener::bind("127.0.0.1:0").expect("Could not bind ephemeral socket");
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            let server = Server::from_tcp(listener)
                .unwrap()
                .serve(app.into_make_service());
            server.await.expect("server error");
        });

        let client = reqwest::Client::new();

        let res = client
            .get(format!("http://{}/expect_inertia", &addr))
            .header("X-Inertia", "true")
            .header("X-Inertia-Version", "required")
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let res = client
            .get(format!("http://{}/expect_not_inertia", &addr))
            .header("X-Inertia", "false")
            .header("X-Inertia-Version", "not-required")
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let res = client
            .get(format!("http://{}/expect_no_version", &addr))
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }
}
