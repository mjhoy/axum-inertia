axum-inertia
============

Implementation of the [inertia.js] protocol for axum.

Provides an `Inertia` axum extractor to render responses like so:

```rust
async fn get_posts(i: Inertia) -> impl IntoResponse {
    i.render("Posts/Index", json!({ "posts": vec!["post one", "post two"] }))
}
```

See crate documentation for more information.

[inertia.js]: https://inertiajs.com
