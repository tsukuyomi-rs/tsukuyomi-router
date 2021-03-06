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
//! assert_eq!(router.recognize("/users/3").route().map(|(r, _)| r.data()), Some(&"users"));
//! assert_eq!(router.recognize("/users/3/books").route().map(|(r, _)| r.data()), Some(&"users_books"));
//! assert_eq!(router.recognize("/coffee_maker/slug").route().map(|(r, _)| r.data()), Some(&"slug"));
//! assert_eq!(router.recognize("/made/up/url").route().map(|(r, _)| r.data()), Some(&"catch_all"));
//! # Ok::<(), tsukuyomi_router::Error>(())
//! ```
//!
//! # Scope
//!
//! ```
//! # use tsukuyomi_router::Router;
//! let mut router = Router::new();
//! router.add_route("/api/v1/posts/:id", "the_post")?;
//! router.add_route("/api/v1/posts/new", "new_post")?;
//!
//! router.add_scope("/api/", "api")?;
//! router.add_scope("/api/v1/", "api_v1")?;
//!
//! assert_eq!(
//!     router.recognize("/api/v1/posts/new").route().map(|(r, _)| r.data()),
//!     Some(&"new_post")
//! );
//! assert_eq!(
//!     router.recognize("/api/v1/posts/new").scope().map(|(s, _)| s.data()),
//!     Some(&"api_v1")
//! );
//!
//! assert!(router.recognize("/api/v1/users").route().is_none());
//! assert_eq!(
//!     router.recognize("/api/v1/users").scope().map(|(s, _)| s.data()),
//!     Some(&"api_v1")
//! );
//!
//! assert_eq!(
//!     router.recognize("/api/users").scope().map(|(s, _)| s.data()),
//!     Some(&"api")
//! );
//!
//! assert!(router.recognize("/favicon.ico").scope().is_none());
//! # Ok::<(), tsukuyomi_router::Error>(())
//! ```
//!

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
mod endpoint;
mod param;
mod router;
mod tree;

pub use crate::{
    endpoint::{Endpoint, EndpointId},
    error::{Error, Result},
    param::Params,
    router::{Recognize, Router},
};
