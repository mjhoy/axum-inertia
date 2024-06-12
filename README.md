[![Crates.io](https://img.shields.io/crates/v/axum-inertia.svg)](https://crates.io/crates/axum-inertia)
[![Documentation](https://docs.rs/axum-inertia/badge.svg)](https://docs.rs/axum-inertia/)

axum-inertia
============

Implementation of the [inertia.js] protocol for axum.

Provides an `Inertia` axum extractor to render responses like so:

```rust
async fn get_posts(i: Inertia) -> impl IntoResponse {
    i.render("Posts/Index", json!({ "posts": vec!["post one", "post two"] }))
}
```

See [crate documentation] for more information.

[inertia.js]: https://inertiajs.com
[crate documentation]: https://docs.rs/axum-inertia/latest/axum_inertia/

## Making a new release

1. Spin off a `bump-vX.X.X` branch
2. Update the `CHANGELOG`; start a new `[Unreleased]` section
3. Bump the version number in `Cargo.toml`
4. Run `cargo release --execute`
5. Merge PR if all goes well
