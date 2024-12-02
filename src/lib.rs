//! An implementation of the [inertia.js] protocol for [axum].
//!
//! The basic idea is that any axum handler that accepts the `Inertia`
//! struct as a function parameter is an inertia endpoint. For
//! instance:
//!
//! ```rust
//! use axum_inertia::Inertia;
//! use axum::{Json, response::IntoResponse};
//! use serde_json::json;
//!
//! async fn my_handler_fn(i: Inertia) -> impl IntoResponse {
//!     i.render("Pages/MyPageComponent", json!({"myPageProps": "true"}))
//! }
//! ```
//!
//! This does the following:
//!
//! - If the incoming request is the initial page load (i.e., does not
//! have the `X-Inertia` header set to `true`), the
//! [render](Inertia::render) method responds with an html page, which
//! is configurable when setting up the initial Inertia state (see
//! [Getting started](#getting-started) below).
//!
//! - Otherwise, the handler responses with the standard inertia
//! "Page" object json, with the included component and page props
//! passed to [render](Inertia::render).
//!
//! - If the request has a mismatching asset version (again, this is
//! configurable), the handler responds with a `409 Conflict` to tell
//! the client to reload the page. The function body of the handler is
//! not executed in this case.
//!
//! # Getting started
//!
//! First, you'll need to provide your axum routes with
//! [InertiaConfig] state. This state boils down to two things: an
//! optional string representing the [asset version] and a function
//! that takes serialized props and returns an HTML string for the
//! initial page load.
//!
//! The [vite] module provides a convenient way to set up this state
//! with [axum::Router::with_state]. For instance, the following code
//! sets up a standard development server:
//!
//! ```rust
//! use axum_inertia::{vite, Inertia};
//! use axum::{Router, routing::get, response::IntoResponse};
//!
//! // Configuration for Inertia when using `vite dev`:
//! let inertia = vite::Development::default()
//!     .port(5173)
//!     .main("src/main.ts")
//!     .lang("en")
//!     .title("My inertia app")
//!     .into_config();
//! let app: Router = Router::new()
//!     .route("/", get(get_root))
//!     .with_state(inertia);
//!
//! # async fn get_root(_i: Inertia) -> impl IntoResponse { "foo" }
//! ```
//!
//! The [Inertia] struct is then available as an axum [Extractor] and
//! can be used in handlers like so:
//!
//! ```rust
//! use axum::response::IntoResponse;
//! # use axum_inertia::Inertia;
//! use serde_json::json;
//!
//! async fn get_root(i: Inertia) -> impl IntoResponse {
//!     i.render("Pages/Home", json!({ "posts": vec!["post one", "post two"] }))
//! }
//! ```
//!
//! The [Inertia::render] method takes care of building a response
//! conforming to the [inertia.js protocol]. It takes two parameters:
//! the name of the component to render, and the page props
//! (serializable to json).
//!
//! Using the extractor in a handler *requires* that you use
//! [axum::Router::with_state] to initialize Inertia in your
//! routes. In fact, it won't compile if you don't!
//!
//! # Using InertiaConfig as substate
//!
//! It's likely you'll want other pieces of state beyond
//! [InertiaConfig]. You'll just need to implement
//! [axum::extract::FromRef] for your state type for
//! [InertiaConfig]. For instance:
//!
//! ```rust
//! use axum_inertia::{vite, Inertia, InertiaConfig};
//! use axum::{Router, routing::get, extract::FromRef};
//! # use axum::response::IntoResponse;
//!
//! #[derive(Clone)]
//! struct AppState {
//!     inertia: InertiaConfig,
//!     name: String
//! }
//!
//! impl FromRef<AppState> for InertiaConfig {
//!     fn from_ref(app_state: &AppState) -> InertiaConfig {
//!         app_state.inertia.clone()
//!     }
//! }
//!
//! let inertia = vite::Development::default()
//!     .port(5173)
//!     .main("src/main.ts")
//!     .lang("en")
//!     .title("My inertia app")
//!     .into_config();
//! let app_state = AppState { inertia, name: "foo".to_string() };
//! let app: Router = Router::new()
//!     .route("/", get(get_root))
//!     .with_state(app_state);
//!
//! # async fn get_root(_i: Inertia) -> impl IntoResponse { "foo" }
//! ```
//!
//! # Configuring development and production
//!
//! See the [vite] module for more information.
//!
//! [Router::with_state]: https://docs.rs/axum/latest/axum/struct.Router.html#method.with_state
//! [asset version]: https://inertiajs.com/the-protocol#asset-versioning
//! [inertia.js]: https://inertiajs.com
//! [inertia.js protocol]: https://inertiajs.com/the-protocol
//! [axum]: https://crates.io/crates/axum
//! [Extractor]: https://docs.rs/axum/latest/axum/#extractors

use async_trait::async_trait;
use axum::extract::{FromRef, FromRequestParts};
pub use config::InertiaConfig;
use http::{request::Parts, HeaderMap, HeaderValue, StatusCode};
use page::Page;
use props::Props;
use request::Request;
use response::Response;

pub mod config;
mod page;
pub mod partial;
pub mod props;
mod request;
mod response;
pub mod vite;

#[derive(Clone)]
pub struct Inertia {
    request: Request,
    config: InertiaConfig,
}

#[async_trait]
impl<S> FromRequestParts<S> for Inertia
where
    S: Send + Sync,
    InertiaConfig: FromRef<S>,
{
    type Rejection = (StatusCode, HeaderMap<HeaderValue>);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let config = InertiaConfig::from_ref(state);
        let request = Request::from_request_parts(parts, state).await?;

        // Respond with a 409 conflict if X-Inertia-Version values
        // don't match for GET requests. See more at:
        // https://inertiajs.com/the-protocol#asset-versioning
        if parts.method == "GET"
            && request.is_xhr
            && config.version().is_some()
            && request.version != config.version()
        {
            let mut headers = HeaderMap::new();
            headers.insert("X-Inertia-Location", parts.uri.path().parse().unwrap());
            return Err((StatusCode::CONFLICT, headers));
        }

        Ok(Inertia::new(request, config))
    }
}

impl Inertia {
    fn new(request: Request, config: InertiaConfig) -> Inertia {
        Inertia { request, config }
    }

    /// Renders an Inertia response.
    pub fn render<S: Props>(self, component: &str, props: S) -> Response {
        let request = self.request;
        let url = request.url.clone();
        let page = Page {
            component,
            props: props
                .serialize(request.partial.as_ref())
                // TODO: error handling
                .expect("serialization failure"),
            url,
            version: self.config.version().clone(),
        };
        Response {
            page,
            request,
            config: self.config,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{self, response::IntoResponse, routing::get, Router};
    use reqwest::StatusCode;
    use serde_json::json;
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn it_works() {
        async fn handler(i: Inertia) -> impl IntoResponse {
            i.render("foo!", json!({"bar": "baz"}))
        }

        let layout =
            Box::new(|props| format!(r#"<html><body><div id="app" data-page='{}'></div>"#, props));

        let config = InertiaConfig::new(Some("123".to_string()), layout);

        let app = Router::new()
            .route("/test", get(handler))
            .with_state(config);

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Could not bind ephemeral socket");
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app).await.expect("server error");
        });

        let res = reqwest::get(format!("http://{}/test", &addr))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(
            res.headers()
                .get("X-Inertia-Version")
                .map(|h| h.to_str().unwrap()),
            Some("123")
        );
    }

    #[tokio::test]
    async fn it_responds_with_conflict_on_version_mismatch() {
        async fn handler(i: Inertia) -> impl IntoResponse {
            i.render("foo!", json!({"bar": "baz"}))
        }

        let layout =
            Box::new(|props| format!(r#"<html><body><div id="app" data-page='{}'></div>"#, props));

        let inertia = InertiaConfig::new(Some("123".to_string()), layout);

        let app = Router::new()
            .route("/test", get(handler))
            .with_state(inertia);

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Could not bind ephemeral socket");
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app).await.expect("server error");
        });

        let client = reqwest::Client::new();

        let res = client
            .get(format!("http://{}/test", &addr))
            .header("X-Inertia", "true")
            .header("X-Inertia-Version", "456")
            .send()
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::CONFLICT);
        assert_eq!(
            res.headers()
                .get("X-Inertia-Location")
                .map(|h| h.to_str().unwrap()),
            Some("/test")
        );
    }
}
