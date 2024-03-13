/// Partial reload data.
///
/// Clients can request a subset of the props if a page component is
/// being refreshed. They must also include a desired component -- the
/// server may respond with a different end component, which will
/// include a full response.
#[derive(Clone, Debug)]
pub struct Partial {
    pub props: Vec<String>,
    pub component: String,
}
