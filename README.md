# `tsukuyomi-router`

[![Build Status](https://dev.azure.com/tsukuyomi-rs/tsukuyomi-router/_apis/build/status/tsukuyomi-rs.tsukuyomi-router?branchName=master)](https://dev.azure.com/tsukuyomi-rs/tsukuyomi-router/_build/latest?definitionId=2&branchName=master)
[![codecov](https://codecov.io/gh/tsukuyomi-rs/tsukuyomi-router/branch/master/graph/badge.svg)](https://codecov.io/gh/tsukuyomi-rs/tsukuyomi-router)

The next-generation HTTP router for Tsukuyomi Web framework.

## Example

```rust
use tsukuyomi_router::Router;

let mut router = Router::new();
router.add_route("/domain/mime", "mime")?;
router.add_route("/domain/yours", "yours")?;

router.recognize("/domain/mime").route().map(|(r, _)| r.data()) // => Some(&"mime")
router.recognize("/domain/mi").route()   // => None
```

## Status
WIP

## License

This project is licensed under either of

* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT),
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.
