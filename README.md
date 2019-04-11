# `tsukuyomi-router`

An experimental HTTP router for general purpose.

## Example

```rust
use tsukuyomi_router::Router;

let mut router = Router::new();
router.add_route("/domain/mime", "mime")?;
router.add_route("/domain/yours", "yours")?;

router.find_route("/domain/mime").route // => Some("mime")
router.find_route("/domain/mi").route   // => None
```

## Status
WIP
