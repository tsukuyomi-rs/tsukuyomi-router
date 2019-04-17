//! An experimental implementation of HTTP router for Tsukuyomi Web framework.
//!
//! # Example
//!
//! ```
//! # use tsukuyomi_router::Router;
//! let mut router = Router::default();
//! router.add_route("/", "root")?;
//! router.add_route("/users/:id", "users")?;
//! router.add_route("/users/:id/books", "users_books")?;
//! router.add_route("/*/slug", "slug")?;
//! router.add_route("/*", "catch_all")?;
//!
//! assert_eq!(router.recognize("/users/3").route().map(|r| r.data()), Some(&"users"));
//! assert_eq!(router.recognize("/users/3/books").route().map(|r| r.data()), Some(&"users_books"));
//! assert_eq!(router.recognize("/coffee_maker/slug").route().map(|r| r.data()), Some(&"slug"));
//! assert_eq!(router.recognize("/made/up/url").route().map(|r| r.data()), Some(&"catch_all"));
//! # Ok::<(), tsukuyomi_router::Error>(())
//! ```
//!
//! # Trailing Slash Recommendation
//!
//! ```ignore
//! # use tsukuyomi_router::Router;
//! let mut router = Router::default();
//! router.add("/path/to/dir", "payload")?;
//!
//! assert_eq!(router.find("/path/to/dir").route(), Some(&"payload"));
//!
//! assert!(router.find("/path/to/dir/").route().is_none());
//! assert_eq!(router.find("/path/to/dir/").tsr(), Some(&"payload"));
//! # Ok::<(), tsukuyomi_router::Error>(())
//! ```
//!
//! ```ignore
//! # use tsukuyomi_router::Router;
//! let mut router = Router::default();
//! router.add("/path/to/dir", "payload1")?;
//! router.add("/path/to/dir/", "payload2")?;
//!
//! assert_eq!(router.find("/path/to/dir").route(), Some(&"payload1"));
//! assert_eq!(router.find("/path/to/dir/").route(), Some(&"payload2"));
//! # Ok::<(), tsukuyomi_router::Error>(())
//! ```
//!
//! # Scope
//!
//! ```ignore
//! # use tsukuyomi_router::Router;
//! let mut router = Router::default();
//! router.add("/api/v1/posts/:id", "the_post")?;
//! router.add("/api/v1/posts/new", "new_post")?;
//! router.add_scope("/api/v1/", "api_scope")?;
//!
//! assert!(router.find("/api/v1/users").route().is_none());
//! assert_eq!(router.find("/api/v1/users").scope(), Some(&"api_scope"));
//! assert!(router.find("/api/v1").scope().is_none());
//! # Ok::<(), tsukuyomi_router::Error>(())
//! ```

#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_compatibility,
    rust_2018_idioms,
    unsafe_code,
    unused,
    clippy::unimplemented
)]

#[macro_use]
mod error;
mod param;
mod router;
mod tree;

pub use crate::{
    error::{Error, Result},
    param::Params,
    router::{Recognize, Route, RouteId, Router},
};
