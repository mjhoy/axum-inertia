use indoc::formatdoc;

use crate::HtmlLayout;

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
