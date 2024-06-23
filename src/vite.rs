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
    port: u16,
    main: &'static str,
    lang: &'static str,
    title: &'static str,
    react: bool,
}

impl Default for Development {
    fn default() -> Self {
        Development {
            port: 5173,
            main: "src/main.ts",
            lang: "en",
            title: "Vite",
            react: false,
        }
    }
}

impl Development {
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

    pub fn into_config(self) -> InertiaConfig {
        let layout = Box::new(move |props| {
            let vite_src = format!("http://localhost:{}/@vite/client", self.port);
            let main_src = format!("http://localhost:{}/{}", self.port, self.main);
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
        format!(
            r#"
import RefreshRuntime from "http://localhost:{}/@react-refresh"
RefreshRuntime.injectIntoGlobalHook(window)
window.$RefreshReg$ = () => {{}}
window.$RefreshSig$ = () => (type) => type
window.__vite_plugin_react_preamble_installed__ = true
"#,
            self.port
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
        manifest_path: &'static str,
        main: &'static str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let bytes = std::fs::read(manifest_path)?;
        let manifest: HashMap<String, ManifestEntry> =
            serde_json::from_str(&String::from_utf8(bytes.clone())?)?;
        let entry = manifest.get(main).ok_or(ViteError::EntryMissing(main))?;
        let mut hasher = Sha1::new();
        hasher.update(&bytes);
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
            main: entry.clone(),
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
            let main_integrity = self.main.integrity.clone().unwrap_or("".to_string());
            html! {
                html lang=(self.lang) {
                    head {
                        title { (self.title) }
                        meta charset="utf-8";
                        meta name="viewport" content="width=device-width, initial-scale=1.0";
                        script type="module" src=(main_path) integrity=(main_integrity) {}
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
