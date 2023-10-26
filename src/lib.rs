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
pub mod vite;

#[derive(Clone)]
pub struct Inertia {
    request: Option<Request>,
    html_head: String,
    html_lang: String,
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
    pub fn new(layout: impl HtmlLayout) -> Inertia {
        Inertia {
            request: None,
            html_head: layout.html_head(),
            html_lang: layout.html_lang(),
        }
    }

    pub fn render<'a, S: Serialize>(self, component: &'static str, props: S) -> Response {
        let request = self.request.expect("no request set");
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
            html_head: self.html_head,
            html_lang: self.html_lang,
        }
    }
}

pub trait HtmlLayout {
    fn html_lang(&self) -> String;
    fn html_head(&self) -> String;
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{self, routing::get, Router, Server};
    use reqwest::StatusCode;
    use std::net::TcpListener;

    struct DumbHtmlLayout {
        html_lang: String,
        html_head: String,
    }

    impl HtmlLayout for DumbHtmlLayout {
        fn html_lang(&self) -> String {
            self.html_lang.clone()
        }
        fn html_head(&self) -> String {
            self.html_head.clone()
        }
    }

    #[tokio::test]
    async fn it_works() {
        async fn handler(_: Inertia) {}

        let layout = DumbHtmlLayout {
            html_lang: "en".to_string(),
            html_head: "<title>Foo</title>".to_string(),
        };

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
