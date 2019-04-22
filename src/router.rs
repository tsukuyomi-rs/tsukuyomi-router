use crate::{
    endpoint::{Endpoint, EndpointId}, //
    error::Result,
    param::{ParamNames, Params},
    tree::Tree,
};
use indexmap::IndexMap;
use std::{
    borrow::Cow,
    ops::{Index, IndexMut},
};

/// An HTTP router.
#[derive(Debug)]
pub struct Router<T> {
    tree: Tree,
    endpoints: IndexMap<EndpointId, Endpoint<T>>,
}

impl<T> Default for Router<T> {
    fn default() -> Self {
        Self {
            tree: Tree::default(),
            endpoints: IndexMap::new(),
        }
    }
}

impl<T> Router<T> {
    /// Create an empty router.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a route to this router.
    pub fn add_route(&mut self, path: &str, data: T) -> Result<EndpointId> {
        let id = EndpointId(self.endpoints.len());

        let mut names = None;
        let leaf = self.tree.insert(path.as_ref(), &mut names)?;
        leaf.route = Some(id);

        self.endpoints.insert(
            id,
            Endpoint {
                id,
                path: path.to_owned(),
                names,
                data,
            },
        );

        Ok(id)
    }

    /// Adds a scope to this router.
    pub fn add_scope(&mut self, path: &str, data: T) -> Result<EndpointId> {
        let id = EndpointId(self.endpoints.len());

        let mut names = None;
        let leaf = self.tree.insert(path.as_ref(), &mut names)?;
        leaf.scope = Some(id);

        self.endpoints.insert(
            id,
            Endpoint {
                id,
                path: path.to_owned(),
                names,
                data,
            },
        );

        Ok(id)
    }

    /// Returns a reference to the endpoint with the specified ID.
    pub fn endpoint(&self, id: EndpointId) -> Option<&Endpoint<T>> {
        self.endpoints.get(&id)
    }

    /// Returns a mutable reference to the endpoint with the specified ID.
    pub fn endpoint_mut(&mut self, id: EndpointId) -> Option<&mut Endpoint<T>> {
        self.endpoints.get_mut(&id)
    }

    /// Searches for the route(s) matching the provided path.
    pub fn recognize<'r>(&'r self, path: &'r str) -> Recognize<'r, T> {
        let recognize = self.tree.recognize(path.as_ref());

        Recognize {
            route: recognize.route.and_then(|id| self.endpoints.get(&id)),
            scope: recognize.scope.and_then(|id| self.endpoints.get(&id)),
            path,
            params: recognize.params,
            wildcard: recognize.wildcard,
        }
    }
}

impl<T> Index<EndpointId> for Router<T> {
    type Output = Endpoint<T>;

    fn index(&self, id: EndpointId) -> &Self::Output {
        self.endpoint(id)
            .unwrap_or_else(|| panic!("invalid route ID"))
    }
}

impl<T> IndexMut<EndpointId> for Router<T> {
    fn index_mut(&mut self, id: EndpointId) -> &mut Self::Output {
        self.endpoint_mut(id)
            .unwrap_or_else(|| panic!("invalid route ID"))
    }
}

/// A value that contains the recognition result of the router.
#[derive(Debug)]
pub struct Recognize<'r, T> {
    route: Option<&'r Endpoint<T>>,
    scope: Option<&'r Endpoint<T>>,
    path: &'r str,
    params: Vec<(usize, usize)>,
    wildcard: Option<(usize, usize)>,
}

impl<'r, T> Recognize<'r, T> {
    /// Returns a reference to the matched route if possible.
    pub fn route(&self) -> Option<(&Endpoint<T>, Option<Params<'_>>)> {
        let route = self.route?;
        let params = route.names.as_ref().map(|names| self.new_params(names));
        Some((route, params))
    }

    /// Returns a reference to the matched scope if possible.
    pub fn scope(&self) -> Option<(&Endpoint<T>, Option<Params<'_>>)> {
        let scope = self.scope?;
        let params = scope.names.as_ref().map(|names| self.new_params(names));
        Some((scope, params))
    }

    fn new_params<'a>(&'a self, names: &'a ParamNames) -> Params<'a> {
        Params {
            names: Cow::Borrowed(names),
            path: Cow::Borrowed(&self.path),
            spans: Cow::Borrowed(&self.params),
            wildcard: self.wildcard,
        }
    }
}
