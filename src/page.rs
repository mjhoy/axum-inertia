use serde::Serialize;
use serde_json::Value;

/// Holds data for the Inertia page object.
///
/// Serializes to json. Included in the `data-page` attribute of the
/// initial html page, or sent as the payload for Inertia requests.
///
/// More info at: https://inertiajs.com/the-protocol#the-page-object
#[derive(Serialize)]
pub(crate) struct Page {
    pub(crate) component: String,
    pub(crate) props: Value,
    pub(crate) url: String,
    pub(crate) version: Option<String>,
}
