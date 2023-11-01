use crate::{page::Page, request::Request};
use axum::response::{Html, IntoResponse, Json};
use http::HeaderMap;
use std::sync::Arc;

/// An Inertia response.
///
/// More information at:
/// https://inertiajs.com/the-protocol#inertia-responses
pub struct Response {
    pub(crate) request: Request,
    pub(crate) page: Page,
    pub(crate) layout: Arc<Box<dyn Fn(String) -> String + Send + Sync>>,
    pub(crate) version: Option<String>,
}

impl IntoResponse for Response {
    fn into_response(self) -> axum::response::Response {
        let mut headers = HeaderMap::new();
        if let Some(version) = &self.version {
            headers.insert("X-Inertia-Version", version.parse().unwrap());
        }
        if self.request.is_xhr {
            headers.insert("X-Inertia", "true".parse().unwrap());
            (headers, Json(self.page)).into_response()
        } else {
            let html = (self.layout)(serde_json::to_string(&self.page).unwrap());
            (headers, Html(html)).into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::formatdoc;

    use super::*;

    #[tokio::test]
    async fn test_into_html_response() {
        let request = Request {
            is_xhr: false,
            ..Request::test_request()
        };
        let page = Page {
            component: "Testing",
            props: serde_json::json!({ "test": "test" }),
            url: "/test".to_string(),
            version: None,
        };

        let layout = |props| {
            formatdoc! {r#"
            <html>
            <head>
            <title>Foo!</title>
            </head>
            <body>
                <div id="app" data-page='{}'></div>
            </body>
            </html>
        "#, props}
            .to_string()
        };

        let response = Response {
            request,
            page,
            layout: Arc::new(Box::new(layout)),
            version: Some("123".to_string()),
        }
        .into_response();
        let body = hyper::body::to_bytes(response.into_body())
            .await
            .expect("got bytes");
        let body = String::from_utf8(body.into()).expect("decoded string");

        assert!(body.contains(r#""props":{"test":"test"}"#));
    }
}
