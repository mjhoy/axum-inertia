//! The [Props] trait describes objects that can be used as Inertia
//! props, and allows for handling around [inertia partial
//! reloads](partial-reloads). See the trait documentation for more.
//!
//! [partial-reloads]: https://inertiajs.com/the-protocol#partial-reloads

use serde::Serialize;
use serde_json::Value;
use std::error::Error;

use crate::partial::Partial;

/// Objects that can be used as Inertia props.
pub trait Props {
    /// Serialize to json, given data about partial reloads.
    ///
    /// This method is called when rendering Inertia responses. The
    /// [Partial] object is parsed from the request. Implementations
    /// should return all fields requested in the `props` field. More
    /// information is available in the [inertia docs].
    ///
    /// [inertia docs]: https://inertiajs.com/the-protocol#partial-reloads
    fn serialize(self, partial: Option<&Partial>) -> Result<Value, impl Error>;
}

/// A naive, blanket implementation for all types that implement
/// Serde's [serialize](serde::Serialize). By default, there is no
/// logic around partial handling; the object is just serialized.
impl<T> Props for T
where
    T: Serialize,
{
    fn serialize(self, _: Option<&Partial>) -> Result<Value, impl Error> {
        serde_json::to_value(self)
    }
}
