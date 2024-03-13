use crate::partial::Partial;
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
    pub(crate) partial: Option<Partial>,
}

impl Request {
    #[cfg(test)]
    pub(crate) fn test_request() -> Request {
        Request {
            is_xhr: true,
            version: None,
            url: "/foo/bar".to_string(),
            partial: None,
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
        let partial_data = parts
            .headers
            .get("X-Inertia-Partial-Data")
            .map(|s| s.to_str().map(|s| s.to_string()))
            .transpose()
            .map(|s| s.map(|s| s.split(",").map(|s| s.to_owned()).collect::<Vec<_>>()))
            .map_err(|_err| (StatusCode::BAD_REQUEST, HeaderMap::new()))?;
        let partial_component = parts
            .headers
            .get("X-Inertia-Partial-Component")
            .map(|s| s.to_str().map(|s| s.to_string()))
            .transpose()
            .map_err(|_err| (StatusCode::BAD_REQUEST, HeaderMap::new()))?;
        // TODO: trace warning if we have one of data/component without the other
        // TODO: should this enforce is_xhr is true?
        let partial = match (partial_data, partial_component) {
            (Some(props), Some(component)) => Some(Partial { props, component }),
            _ => None,
        };

        Ok(Request {
            is_xhr,
            version,
            url,
            partial,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use super::*;
    use axum::{self, routing::get, Router};
    use reqwest::StatusCode;
    use tokio::net::TcpListener;
    use tokio::task::JoinHandle;

    async fn spawn_test_app(app: Router) -> (JoinHandle<()>, SocketAddr) {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Could not bind ephemeral socket");
        let addr = listener.local_addr().unwrap();

        (
            tokio::spawn(async move {
                axum::serve(listener, app).await.expect("server error");
            }),
            addr,
        )
    }

    #[tokio::test]
    async fn it_extracts_inertia_xhr() {
        async fn handler(req: Request) {
            assert!(req.is_xhr);
        }
        let app = Router::new().route("/test", get(handler));
        let (_, addr) = spawn_test_app(app).await;
        let client = reqwest::Client::new();

        let res = client
            .get(format!("http://{}/test", &addr))
            .header("X-Inertia", "true")
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn it_extracts_a_false_value_for_x_inertia() {
        async fn handler(req: Request) {
            assert!(!req.is_xhr);
        }
        let app = Router::new().route("/test", get(handler));
        let (_, addr) = spawn_test_app(app).await;

        let client = reqwest::Client::new();

        let res = client
            .get(format!("http://{}/test", &addr))
            .header("X-Inertia", "false")
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn it_extracts_a_version_string() {
        async fn handler(req: Request) {
            assert_eq!(req.version, Some("version".to_string()));
        }
        let app = Router::new().route("/test", get(handler));
        let (_, addr) = spawn_test_app(app).await;

        let client = reqwest::Client::new();

        let res = client
            .get(format!("http://{}/test", &addr))
            .header("X-Inertia-Version", "version")
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn it_works_with_no_version() {
        async fn handler(req: Request) {
            assert_eq!(req.version, None);
        }
        let app = Router::new().route("/test", get(handler));
        let (_, addr) = spawn_test_app(app).await;

        let client = reqwest::Client::new();

        let res = client
            .get(format!("http://{}/test", &addr))
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn it_extracts_partial_data() {
        async fn handler(req: Request) {
            assert!(req.partial.is_some());
            let partial = req.partial.unwrap();
            assert_eq!(partial.props, vec!("one".to_string(), "two".to_string()));
            assert_eq!(partial.component, "PartialComponent");
        }
        let app = Router::new().route("/test", get(handler));
        let (_, addr) = spawn_test_app(app).await;

        let client = reqwest::Client::new();

        let res = client
            .get(format!("http://{}/test", &addr))
            .header("X-Inertia", "true")
            .header("X-Inertia-Partial-Component", "PartialComponent")
            .header("X-Inertia-Partial-Data", "one,two")
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn it_does_not_extract_partial_data_when_missing_headers() {
        async fn handler(req: Request) {
            assert!(req.partial.is_none());
        }
        let app = Router::new().route("/test", get(handler));
        let (_, addr) = spawn_test_app(app).await;

        let client = reqwest::Client::new();

        let res = client
            .get(format!("http://{}/test", &addr))
            .header("X-Inertia", "true")
            .header("X-Inertia-Partial-Data", "one,two")
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let res = client
            .get(format!("http://{}/test", &addr))
            .header("X-Inertia", "true")
            .header("X-Inertia-Partial-Component", "PartialComponent")
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }
}
