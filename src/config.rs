use std::sync::Arc;

type LayoutResolver<'a> = Box<dyn Fn(String) -> String + Send + Sync + 'a>;

struct Inner<'a> {
    version: Option<String>,
    layout: LayoutResolver<'a>,
}

#[derive(Clone)]
pub struct InertiaConfig<'a> {
    inner: Arc<Inner<'a>>,
}

impl InertiaConfig<'_> {
    /// Constructs a new InertiaConfig object.
    ///
    /// `layout` provides information about how to render the initial
    /// page load. See the [crate::vite] module for an implementation
    /// of this for vite.
    pub fn new(version: Option<String>, layout: LayoutResolver) -> InertiaConfig {
        let inner = Inner { version, layout };
        InertiaConfig {
            inner: Arc::new(inner),
        }
    }

    /// Returns a cloned optional version string.
    pub fn version(&self) -> Option<String> {
        self.inner.version.clone()
    }

    /// Returns a reference to the layout function.
    pub fn layout(&self) -> &(dyn Fn(String) -> String + Send + Sync) {
        &self.inner.layout
    }
}
