//! Convenience builders for Inertia using [vitejs].
//!
//! This module provides [Development] and [Production] structs for
//! different environments, e.g.:
//!
//! ```rust
//! use axum_inertia::vite;
//!
//! // are we production?
//! let is_production = std::env::var("APP_ENV").map_or(false, |s| &s[..] == "production");
//!
//! let inertia = if is_production {
//!     vite::Production::new("client/dist/manifest.json", "src/main.ts")
//!         .unwrap()
//!         .lang("en")
//!         .title("My app")
//!         .into_config()
//! } else {
//!     vite::Development::default()
//!         .port(5173)
//!         .main("src/main.ts")
//!         .lang("en")
//!         .title("My app")
//!         .react() // call if using react
//!         .into_config()
//! };
//! ```
//!
//! [vitejs]: https://vitejs.dev
use crate::config::InertiaConfig;
use hex::encode;
use maud::{html, PreEscaped};
use serde::Deserialize;
use sha1::{Digest, Sha1};
use std::collections::HashMap;

pub struct Development {
    base: &'static str,
    port: u16,
    main: &'static str,
    lang: &'static str,
    title: &'static str,
    react: bool,
    https: bool,
}

impl Default for Development {
    fn default() -> Self {
        Development {
            base: "/",
            port: 5173,
            main: "src/main.ts",
            lang: "en",
            title: "Vite",
            react: false,
            https: false,
        }
    }
}

impl Development {
    /// ```rust
    /// use axum_inertia::vite;
    ///
    ///     vite::Development::default()
    ///         .base("/app/") // Must pass slash before and after
    ///         .into_config();
    /// ```
    pub fn base(mut self, base: &'static str) -> Self {
        self.base = base;
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn main(mut self, main: &'static str) -> Self {
        self.main = main;
        self
    }

    pub fn lang(mut self, lang: &'static str) -> Self {
        self.lang = lang;
        self
    }

    pub fn title(mut self, title: &'static str) -> Self {
        self.title = title;
        self
    }

    /// Sets up vite for react usage.
    ///
    /// Currently, this will include preamble code for using react-refresh in the html head.
    /// Some context here: https://github.com/vitejs/vite/issues/1984
    pub fn react(mut self) -> Self {
        self.react = true;
        self
    }

    pub fn https(mut self, https: bool) -> Self {
        self.https = https;
        self
    }

    pub fn into_config(self) -> InertiaConfig {
        let layout = Box::new(move |props| {
            let http_protocol = if self.https { "https" } else { "http" };
            let vite_src = format!(
                "{}://localhost:{}{}@vite/client",
                http_protocol, self.port, self.base
            );
            let main_src = format!(
                "{}://localhost:{}{}{}",
                http_protocol, self.port, self.base, self.main
            );
            let preamble_code = if self.react {
                Some(PreEscaped(self.build_react_preamble()))
            } else {
                None
            };
            html! {
                html lang=(self.lang) {
                    head {
                        title { (self.title) }
                        meta charset="utf-8";
                        meta name="viewport" content="width=device-width, initial-scale=1.0";
                        @if let Some(preamble_code) = preamble_code {
                            script type="module" { (preamble_code) }
                        }
                        script type="module" src=(vite_src) {}
                        script type="module" src=(main_src) {}
                    }

                    body {
                        div #app data-page=(props) {}
                    }
                }
            }
            .into_string()
        });

        InertiaConfig::new(None, layout)
    }

    fn build_react_preamble(&self) -> String {
        let http_protocol = if self.https { "https" } else { "http" };
        format!(
            r#"
import RefreshRuntime from "{}://localhost:{}/@react-refresh"
RefreshRuntime.injectIntoGlobalHook(window)
window.$RefreshReg$ = () => {{}}
window.$RefreshSig$ = () => (type) => type
window.__vite_plugin_react_preamble_installed__ = true
"#,
            http_protocol, self.port
        )
    }
}

pub struct Production {
    main: ManifestEntry,
    css: Option<String>,
    title: &'static str,
    lang: &'static str,
    /// SHA1 hash of the contents of the manifest file.
    version: String,
}

impl Production {
    pub fn new(
        manifest_path: &str,
        main: &'static str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let bytes = std::fs::read(manifest_path)?;
        let manifest: &'static str = Box::leak(String::from_utf8(bytes)?.into_boxed_str());

        Self::new_from_string(manifest, main)
    }

    fn new_from_string(
        manifest_string: &str,
        main: &'static str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut manifest: HashMap<String, ManifestEntry> = serde_json::from_str(manifest_string)?;
        let entry = manifest.remove(main).ok_or(ViteError::EntryMissing(main))?;
        let mut hasher = Sha1::new();
        hasher.update(manifest_string.as_bytes());
        let result = hasher.finalize();
        let version = encode(result);
        let css = {
            if let Some(css_sources) = &entry.css {
                let mut css = String::new();
                for source in css_sources {
                    css.push_str(&format!(r#"<link rel="stylesheet" href="/{source}"/>"#));
                }
                Some(css)
            } else {
                None
            }
        };
        Ok(Self {
            main: entry,
            css,
            title: "Vite",
            lang: "en",
            version,
        })
    }

    pub fn lang(mut self, lang: &'static str) -> Self {
        self.lang = lang;
        self
    }

    pub fn title(mut self, title: &'static str) -> Self {
        self.title = title;
        self
    }

    pub fn into_config(self) -> InertiaConfig {
        let layout = Box::new(move |props| {
            let css = self.css.clone().unwrap_or("".to_string());
            let main_path = format!("/{}", self.main.file);
            let main_integrity = self.main.integrity.clone();

            html! {
                html lang=(self.lang) {
                    head {
                        title { (self.title) }
                        meta charset="utf-8";
                        meta name="viewport" content="width=device-width, initial-scale=1.0";
                        @if let Some(integrity) = main_integrity {
                            script type="module" src=(main_path) integrity=(integrity) {}
                        } else {
                            script type="module" src=(main_path) {}
                        }
                        (PreEscaped(css))
                    }
                    body {
                        div #app data-page=(props) {}
                    }
                }
            }
            .into_string()
        });

        InertiaConfig::new(Some(self.version), layout)
    }
}

#[derive(Debug)]
pub enum ViteError {
    ManifestMissing(std::io::Error),
    EntryMissing(&'static str),
}

impl std::fmt::Display for ViteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ManifestMissing(_) => write!(f, "couldn't open manifest file"),
            Self::EntryMissing(entry) => write!(f, "manifest missing entry for {}", entry),
        }
    }
}

impl std::error::Error for ViteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ManifestMissing(e) => Some(e),
            _ => None,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
struct ManifestEntry {
    file: String,
    integrity: Option<String>,
    css: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_development_default() {
        let development = Development::default();

        assert_eq!(development.port, 5173);
        assert_eq!(development.main, "src/main.ts");
        assert_eq!(development.lang, "en");
        assert_eq!(development.title, "Vite");
        assert!(!development.react);
    }

    #[test]
    fn test_development_builder_methods() {
        let development = Development::default()
            .port(8080)
            .main("src/deep/index.ts")
            .lang("id")
            .title("Untitled Axum Inertia App")
            .react();

        assert_eq!(development.port, 8080);
        assert_eq!(development.main, "src/deep/index.ts");
        assert_eq!(development.lang, "id");
        assert_eq!(development.title, "Untitled Axum Inertia App");
        assert!(development.react);
    }

    #[test]
    fn test_development_url() {
        let development = Development::default().base("/app/").https(true);
        assert!(development.https);
        assert_eq!(development.base, "/app/");

        let config = development.into_config();

        let config_layout = config.layout();
        let binding = config_layout(r#"{"someprops": "somevalues"}"#.to_string());
        let rendered_layout = binding.as_str();

        assert!(rendered_layout.contains(r#"https://localhost:5173/app/@vite/client"#));
        assert!(rendered_layout.contains(r#"https://localhost:5173/app/src/main.ts"#));
    }

    #[test]
    fn test_development_into_config() {
        let main_script = "src/index.ts";
        let development = Development::default()
            .port(8080)
            .main(main_script)
            .lang("lang-id")
            .title("app-title-here")
            .react();

        let config = development.into_config();

        assert_eq!(config.version(), None);

        let config_layout = config.layout();
        let binding = config_layout(r#"{"someprops": "somevalues"}"#.to_string());
        let rendered_layout = binding.as_str();

        assert!(rendered_layout.contains(r#"<html lang="lang-id">"#));
        assert!(rendered_layout.contains(r#"<title>app-title-here</title>"#));
        assert!(rendered_layout.contains(r#"{&quot;someprops&quot;: &quot;somevalues&quot;}"#));
        assert!(rendered_layout.contains(r#"http://localhost:8080/@vite/client"#));
        assert!(
            rendered_layout.contains(r#"window.__vite_plugin_react_preamble_installed__ = true"#)
        );
    }

    #[test]
    fn test_production_new_entry_missing() {
        let manifest_content = r#"{"main.js": {}}"#;
        let result = Production::new_from_string(manifest_content, "nonexistent.js");

        assert!(result.is_err());
    }

    #[test]
    fn test_production_new() {
        let manifest_content =
            r#"{"main.js": {"file": "main.hash-id-here.js", "css": ["style.css"]}}"#;
        let production_res = Production::new_from_string(manifest_content, "main.js");

        assert!(production_res.is_ok());

        let production = production_res.unwrap();
        let content_hash = encode(Sha1::digest(manifest_content.as_bytes()));

        assert_eq!(production.main.css, Some(vec!(String::from("style.css"))));
        assert_eq!(production.title, "Vite");
        assert_eq!(production.main.file, "main.hash-id-here.js");
        assert_eq!(production.main.integrity, None);
        assert_eq!(production.lang, "en");
        assert_eq!(production.version, content_hash);
    }

    #[test]
    fn test_production_builder_methods() {
        let manifest_content =
            r#"{"main.js": {"file": "main.hash-id-here.js", "css": ["style.css"]}}"#;
        let production = Production::new_from_string(manifest_content, "main.js")
            .unwrap()
            .lang("fr")
            .title("Untitled Axum Inertia App");

        assert_eq!(production.lang, "fr");
        assert_eq!(production.title, "Untitled Axum Inertia App");
    }

    #[test]
    fn test_production_into_config() {
        let manifest_content =
            r#"{"main.js": {"file": "main.hash-id-here.js", "css": ["style.css"]}}"#;
        let production = Production::new_from_string(manifest_content, "main.js")
            .unwrap()
            .lang("jv")
            .title("Untitled Axum Inertia App");

        let config = production.into_config();
        let config_layout = config.layout();
        let binding = config_layout(r#"{"someprops": "somevalues"}"#.to_string());
        let rendered_layout = binding.as_str();

        assert!(rendered_layout
            .contains(r#"<script type="module" src="/main.hash-id-here.js"></script>"#));
        assert!(rendered_layout.contains(r#"<link rel="stylesheet" href="/style.css"/>"#));
        assert!(rendered_layout.contains(r#"<html lang="jv">"#));
        assert!(rendered_layout.contains(r#"<title>Untitled Axum Inertia App</title>"#));
        assert!(rendered_layout.contains(r#"{&quot;someprops&quot;: &quot;somevalues&quot;}"#));
    }

    #[test]
    fn test_production_into_config_with_integrity() {
        let manifest_content = r#"{"main.js": {"file": "main.hash-id-here.js", "integrity": "sha000-shaHashHere1234", "css": ["style.css"]}}"#;
        let production = Production::new_from_string(manifest_content, "main.js")
            .unwrap()
            .lang("jv")
            .title("Untitled Axum Inertia App");

        let config = production.into_config();
        let config_layout = config.layout();
        let binding = config_layout(r#"{"someprops": "somevalues"}"#.to_string());
        let rendered_layout = binding.as_str();

        assert!(rendered_layout.contains(r#"<script type="module" src="/main.hash-id-here.js" integrity="sha000-shaHashHere1234"></script>"#));
        assert!(rendered_layout.contains(r#"<link rel="stylesheet" href="/style.css"/>"#));
        assert!(rendered_layout.contains(r#"<html lang="jv">"#));
        assert!(rendered_layout.contains(r#"<title>Untitled Axum Inertia App</title>"#));
        assert!(rendered_layout.contains(r#"{&quot;someprops&quot;: &quot;somevalues&quot;}"#));
    }
}
