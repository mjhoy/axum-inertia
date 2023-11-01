use crate::Inertia;
use hex::encode;
use indoc::formatdoc;
use serde::Deserialize;
use sha1::{Digest, Sha1};
use std::collections::HashMap;

pub struct Development {
    port: u16,
    main: &'static str,
    lang: &'static str,
    title: &'static str,
}

impl Development {
    pub fn new() -> Self {
        Development {
            port: 5173,
            main: "src/main.ts",
            lang: "en",
            title: "Vite",
        }
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

    pub fn inertia(self) -> Inertia {
        let layout = Box::new(move |props| {
            formatdoc! {r#"
                <html lang={lang}>
                <head>
                    <title>{title}</title>
                    <meta charset='utf-8' />
                    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                    <script type="module" src="http://localhost:{port}/@vite/client"></script>
                    <script type="module" src="http://localhost:{port}/{main}"></script>
                </head>
                <body>
                    <div id="app" data-page='{props}'></div>
                </body>
                </html>
              "#, title = self.title, port = self.port, main = self.main, lang = self.lang
            }
        });
        Inertia::new(None, layout)
    }
}

pub struct Production {
    main: String,
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
            main: format!("/{}", entry.file),
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

    pub fn inertia(self) -> Inertia {
        let layout = Box::new(move |props| {
            formatdoc! {r#"
                <html lang={lang}>
                <head>
                    <title>{title}</title>
                    <meta charset='utf-8' />
                    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                    <script type="module" src="{main}"></script>
                    {css}
                </head>
                <body>
                    <div id="app" data-page='{props}'></div>
                </body>
                </html>
              "#, title = self.title, main = self.main, lang = self.lang, css = self.css.clone().unwrap_or("".to_string())
            }
        });
        Inertia::new(Some(self.version), layout)
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
