use indoc::formatdoc;

use crate::HtmlLayout;

/// Struct representing a Vite configuration.
///
/// TODO: currently development only!
///
/// Can be passed to `Inertia::new()` to configure Inertia's initial
/// page load with references to vite scripts. E.g., the following
/// configuration:
///
/// ```rust
/// use axum_inertia::{Inertia, vite::Vite};
///
/// let vite = Vite {
///     port: 5173,
///     main: "src/main.ts",
///     lang: "en",
///     title: "My cool app",
/// };
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
pub struct Vite {
    pub port: u16,
    pub main: &'static str,
    pub lang: &'static str,
    pub title: &'static str,
}

impl HtmlLayout for Vite {
    fn html_lang(&self) -> String {
        self.lang.to_string()
    }

    fn html_head(&self) -> String {
        formatdoc! {r#"
          <title>{title}</title>
          <meta charset='utf-8' />
          <meta name="viewport" content="width=device-width, initial-scale=1.0" />
          <script type="module" src="http://localhost:{port}/@vite/client"></script>
          <script type="module" src="http://localhost:{port}/{main}"></script>        
        "#,
                    title = self.title,
                    port = self.port,
                    main = self.main,
        }
    }
}
