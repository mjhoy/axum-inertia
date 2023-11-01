use crate::AssetConfig;
use hex::encode;
use indoc::formatdoc;
use serde::Deserialize;
use sha1::{Digest, Sha1};
use std::collections::HashMap;

/// Struct representing a Vite configuration.
///
/// Can be passed to `Inertia::new()` to configure Inertia's initial
/// page load with references to vite scripts. E.g., the following
/// configuration:
///
/// ```rust
/// use axum_inertia::{Inertia, vite::Vite};
///
/// let vite = Vite::new_dev(5173, "src/main.ts", "en", "My cool app");
/// let inertia = Inertia::new(vite);
/// ```
///
/// will produce the following template when rendered with Inertia:
///
/// ```html
/// <!doctype html>
/// <html lang="en">
///     <head>
///         <title>My cool app</title>
///         <meta charset='utf-8' />
///         <meta name="viewport" content="width=device-width, initial-scale=1.0" />
///         <script type="module" src="http://localhost:5173/@vite/client"></script>
///         <script type="module" src="http://localhost:5173/src/main.ts"></script>
///     </head>
///     <body>
///         <div id="app" data-page='{inertia props here}'></div>
///     </body>
/// </html>
/// ```
pub enum Vite {
    Development {
        port: u16,
        main: &'static str,
        lang: &'static str,
        title: &'static str,
    },
    Production {
        main: String,
        css: Option<String>,
        title: &'static str,
        lang: &'static str,
        /// SHA1 hash of the contents of the manifest file.
        version: String,
    },
}

impl Vite {
    /// Create a new development vite configuration.
    pub fn new_dev(port: u16, main: &'static str, lang: &'static str, title: &'static str) -> Self {
        Self::Development {
            port,
            main,
            lang,
            title,
        }
    }

    /// Create a new production vite configuration from a vite manifest.
    pub fn new_prod(
        manifest_path: &'static str,
        main: &'static str,
        lang: &'static str,
        title: &'static str,
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
        Ok(Self::Production {
            main: format!("/{}", entry.file),
            css,
            title,
            lang,
            version,
        })
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

#[derive(Debug, Deserialize)]
struct ManifestEntry {
    file: String,
    css: Option<Vec<String>>,
}

impl AssetConfig for Vite {
    fn version(&self) -> Option<String> {
        match self {
            Self::Development { .. } => None,
            Self::Production {
                main: _,
                css: _,
                title: _,
                lang: _,
                version,
            } => Some(version.clone()),
        }
    }

    fn html_lang(&self) -> String {
        match self {
            Self::Development {
                port: _,
                main: _,
                lang,
                title: _,
            } => lang.to_string(),
            Self::Production {
                main: _,
                css: _,
                title: _,
                lang,
                version: _,
            } => lang.to_string(),
        }
    }

    fn html_head(&self) -> String {
        match self {
            Self::Development {
                port,
                main,
                lang: _,
                title,
            } => {
                formatdoc! {r#"
                    <title>{title}</title>
                    <meta charset='utf-8' />
                    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                    <script type="module" src="http://localhost:{port}/@vite/client"></script>
                    <script type="module" src="http://localhost:{port}/{main}"></script>
                  "#,
                }
            }
            Self::Production {
                main,
                css,
                title,
                lang: _,
                version: _,
            } => {
                let blank = "".to_string();
                let css = css.as_ref().unwrap_or(&blank);
                formatdoc! {r#"
                    <title>{title}</title>
                    <meta charset='utf-8' />
                    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                    <script type="module" src="{main}"></script>
                    {css}
                  "#,
                }
            }
        }
    }
}
