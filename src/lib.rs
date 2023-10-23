use std::sync::Arc;

use async_trait::async_trait;
use axum::extract::{FromRef, FromRequestParts};
use http::{request::Parts, StatusCode};
use page::Page;
use request::Request;
use response::Response;
use serde::Serialize;

mod page;
mod request;
mod response;

#[derive(Clone)]
pub struct Inertia {
    request: Option<Request>,
    layout: Arc<dyn Fn(String) -> String + Sync + Send>,
}

#[async_trait]
impl<S> FromRequestParts<S> for Inertia
where
    S: Send + Sync,
    Inertia: FromRef<S>,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let mut inertia = Inertia::from_ref(state);
        let request = Request::from_request_parts(parts, state).await?;
        inertia.request = Some(request);
        Ok(inertia)
    }
}

impl Inertia {
    /// Constructs a new Inertia object.
    ///
    /// `layout` is a function from a json string (props) to the HTML
    /// layout.
    pub fn new(layout: Box<dyn Fn(String) -> String + Sync + Send>) -> Inertia {
        Inertia {
            request: None,
            layout: Arc::new(layout),
        }
    }

    pub fn render<S: Serialize>(self, component: &'static str, props: S) -> Response {
        let request = self.request.expect("request set on inertia");
        let url = request.url.clone();
        let page = Page {
            component,
            props: serde_json::to_value(props).expect("serialize"),
            url,
            version: request.version.clone(),
        };
        Response {
            page,
            request,
            layout: self.layout.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{self, routing::get, Router, Server};
    use reqwest::StatusCode;
    use std::net::TcpListener;

    #[tokio::test]
    async fn it_works() {
        async fn handler(_: Inertia) {}

        let layout = Box::new(|string| format!("{}", string));

        let inertia = Inertia::new(layout);

        let app = Router::new()
            .route("/test", get(handler))
            .with_state(inertia);

        let listener = TcpListener::bind("127.0.0.1:0").expect("Could not bind ephemeral socket");
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            let server = Server::from_tcp(listener)
                .unwrap()
                .serve(app.into_make_service());
            server.await.expect("server error");
        });

        let res = reqwest::get(format!("http://{}/test", &addr))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }
}
