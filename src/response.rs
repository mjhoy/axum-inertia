use std::sync::Arc;

use crate::{page::Page, request::Request};
use axum::response::{Html, IntoResponse, Json};

/// An Inertia response.
///
/// More information at:
/// https://inertiajs.com/the-protocol#inertia-responses
pub struct Response {
    pub(crate) request: Request,
    pub(crate) page: Page,
    pub(crate) layout: Arc<dyn Fn(String) -> String + Send + Send>,
}

impl Response {
    fn initial_html(&self) -> String {
        (self.layout)(serde_json::to_string(&self.page).unwrap())
    }
}

impl IntoResponse for Response {
    fn into_response(self) -> axum::response::Response {
        if self.request.is_xhr {
            ([("X-Inertia", "true")], Json(self.page)).into_response()
        } else {
            let html = self.initial_html();
            Html(html).into_response()
        }
    }
}

#[cfg(test)]
mod tests {
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

        let layout = Arc::new(|string| format!("foo {}", string));

        let response = Response {
            request,
            page,
            layout,
        }
        .into_response();
        let body = hyper::body::to_bytes(response.into_body())
            .await
            .expect("got bytes");
        let body = String::from_utf8(body.into()).expect("decoded string");

        assert!(body.contains(r#""props":{"test":"test"}"#));
    }
}
