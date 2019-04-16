# `tsukuyomi-router`

[![Build Status](https://dev.azure.com/tsukuyomi-rs/tsukuyomi-router/_apis/build/status/tsukuyomi-rs.tsukuyomi-router?branchName=master)](https://dev.azure.com/tsukuyomi-rs/tsukuyomi-router/_build/latest?definitionId=2&branchName=master)

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

## License

This project is licensed under either of

* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT),
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.
