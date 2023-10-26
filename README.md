axum-inertia
============

Implementation of the [inertia.js] protocol for axum.

Currently very work-in-progress. Basic idea is to provide an `Inertia`
axum extractor like so:

```rust
async fn get_posts(i: Inertia) -> impl IntoResponse {
    i.render("Posts/Index", json!({ "posts": vec!["post one", "post two"] }))
}
```

[inertia.js]: https://inertiajs.com
