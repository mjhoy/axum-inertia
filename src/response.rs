use crate::{page::Page, request::Request};
use axum::response::{Html, IntoResponse, Json};
use indoc::formatdoc;

/// An Inertia response.
///
/// More information at:
/// https://inertiajs.com/the-protocol#inertia-responses
pub struct Response {
    pub(crate) request: Request,
    pub(crate) page: Page,
    pub(crate) html_head: String,
    pub(crate) html_lang: String,
}

impl Response {
    fn initial_html(&self) -> String {
        formatdoc! {r#"
            <!doctype html>
            <html lang="{}">
                <head>
                    {}
                </head>
                <body>
                    <div id="app" data-page='{}'></div>
                </body>
            </html>          
        "#, self.html_lang, self.html_head, serde_json::to_string(&self.page).unwrap()
        }
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
    use indoc::indoc;

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

        let html_head = indoc! {r#"
          <title>Foo!</title>
        "#};

        let response = Response {
            request,
            page,
            html_head: html_head.to_string(),
            html_lang: "en".to_string(),
        }
        .into_response();
        let body = hyper::body::to_bytes(response.into_body())
            .await
            .expect("got bytes");
        let body = String::from_utf8(body.into()).expect("decoded string");

        assert!(body.contains(r#""props":{"test":"test"}"#));
    }
}
