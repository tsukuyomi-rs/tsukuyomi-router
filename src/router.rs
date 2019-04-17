use crate::{
    error::Result, //
    param::{ParamNames, Params},
    tree::Tree,
};
use indexmap::IndexMap;
use std::borrow::Cow;

/// An HTTP router.
#[derive(Debug)]
pub struct Router<T> {
    tree: Tree,
    routes: IndexMap<RouteId, Route<T>>,
    scopes: IndexMap<ScopeId, Scope<T>>,
}

impl<T> Default for Router<T> {
    fn default() -> Self {
        Self {
            tree: Tree::default(),
            routes: IndexMap::new(),
            scopes: IndexMap::new(),
        }
    }
}

impl<T> Router<T> {
    /// Create an empty router.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a route to this router associated with the specified path.
    pub fn add_route(&mut self, path: &str, data: T) -> Result<RouteId> {
        let id = RouteId(self.routes.len());

        let mut names = None;
        let leaf = self.tree.insert(path.as_ref(), &mut names)?;
        leaf.route = Some(id);

        self.routes.insert(
            id,
            Route {
                id,
                path: path.to_owned(),
                names,
                data,
            },
        );

        Ok(id)
    }

    /// Adds a scope with the specified path to this router.
    pub fn add_scope(&mut self, path: &str, data: T) -> Result<ScopeId> {
        let id = ScopeId(self.scopes.len());

        let mut names = None;
        let leaf = self.tree.insert(path.as_ref(), &mut names)?;
        leaf.scope = Some(id);

        self.scopes.insert(
            id,
            Scope {
                id,
                path: path.to_owned(),
                names,
                data,
            },
        );

        Ok(id)
    }

    /// Retrieves a reference to the route with the specified identifier.
    pub fn route(&self, id: RouteId) -> Option<&Route<T>> {
        self.routes.get(&id)
    }

    /// Retrieves a mutable reference to the route with the specified identifier.
    pub fn route_mut(&mut self, id: RouteId) -> Option<&mut Route<T>> {
        self.routes.get_mut(&id)
    }

    /// Retrieves a reference to the scope with the specified identifier.
    pub fn scope(&self, id: ScopeId) -> Option<&Scope<T>> {
        self.scopes.get(&id)
    }

    /// Retrieves a mutable reference to the scope with the specified identifier.
    pub fn scope_mut(&mut self, id: ScopeId) -> Option<&mut Scope<T>> {
        self.scopes.get_mut(&id)
    }

    /// Recognizes the location of the provided path on this router.
    pub fn recognize<'r>(&'r self, path: &'r str) -> Recognize<'r, T> {
        let recognize = self.tree.recognize(path.as_ref());

        Recognize {
            route: recognize.route.and_then(|id| self.route(id)),
            scope: recognize
                .scopes
                .iter()
                .last()
                .and_then(|&id| self.scope(id)),
            path,
            params: recognize.params,
            wildcard: recognize.wildcard,
        }
    }
}

impl<T> std::ops::Index<RouteId> for Router<T> {
    type Output = Route<T>;

    fn index(&self, id: RouteId) -> &Self::Output {
        self.route(id).unwrap_or_else(|| panic!("invalid route ID"))
    }
}

impl<T> std::ops::IndexMut<RouteId> for Router<T> {
    fn index_mut(&mut self, id: RouteId) -> &mut Self::Output {
        self.route_mut(id)
            .unwrap_or_else(|| panic!("invalid route ID"))
    }
}

impl<T> std::ops::Index<ScopeId> for Router<T> {
    type Output = Scope<T>;

    fn index(&self, id: ScopeId) -> &Self::Output {
        self.scope(id).unwrap_or_else(|| panic!("invalid route ID"))
    }
}

impl<T> std::ops::IndexMut<ScopeId> for Router<T> {
    fn index_mut(&mut self, id: ScopeId) -> &mut Self::Output {
        self.scope_mut(id)
            .unwrap_or_else(|| panic!("invalid route ID"))
    }
}

/// The value for identifying the instance of `Route`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct RouteId(pub(crate) usize);

/// A route in `Router`.
#[derive(Debug)]
pub struct Route<T> {
    id: RouteId,
    path: String,
    names: Option<ParamNames>,
    data: T,
}

impl<T> Route<T> {
    /// Returns the identifier associated with this route.
    pub fn id(&self) -> RouteId {
        self.id
    }

    /// Returns the original path of this route.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns a reference to the data associated with this route.
    pub fn data(&self) -> &T {
        &self.data
    }

    /// Returns a reference to the data associated with this route.
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

impl<T> std::ops::Deref for Route<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.data()
    }
}

impl<T> std::ops::DerefMut for Route<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data_mut()
    }
}

/// The value for identifying the instance of `Scope`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ScopeId(pub(crate) usize);

/// A scope in `Router`.
#[derive(Debug)]
pub struct Scope<T> {
    id: ScopeId,
    path: String,
    names: Option<ParamNames>,
    data: T,
}

impl<T> Scope<T> {
    /// Returns the identifier associated with this route.
    pub fn id(&self) -> ScopeId {
        self.id
    }

    /// Returns the original path of this route.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns a reference to the data associated with this route.
    pub fn data(&self) -> &T {
        &self.data
    }

    /// Returns a reference to the data associated with this route.
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

impl<T> std::ops::Deref for Scope<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.data()
    }
}

impl<T> std::ops::DerefMut for Scope<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data_mut()
    }
}

/// A value that contains the recognition result of the router.
#[derive(Debug)]
pub struct Recognize<'r, T> {
    route: Option<&'r Route<T>>,
    scope: Option<&'r Scope<T>>,
    path: &'r str,
    params: Vec<(usize, usize)>,
    wildcard: Option<(usize, usize)>,
}

impl<'r, T> Recognize<'r, T> {
    /// Returns a reference to the matched route if possible.
    pub fn route(&self) -> Option<&Route<T>> {
        self.route
    }

    /// Creates a `Params` associated with the matched route.
    pub fn params(&self) -> Option<Params<'_>> {
        let names = self.route?.names.as_ref()?;
        Some(self.new_params(names))
    }

    /// Returns a reference to the matched scope if possible.
    pub fn scope(&self) -> Option<&Scope<T>> {
        self.scope
    }

    /// Creates a `Params` associated with the matched scope.
    pub fn scope_params(&self) -> Option<Params<'_>> {
        let names = self.scope?.names.as_ref()?;
        Some(self.new_params(names))
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
