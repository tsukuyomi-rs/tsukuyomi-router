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
}

impl<T> Default for Router<T> {
    fn default() -> Self {
        Self {
            tree: Tree::default(),
            routes: IndexMap::new(),
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
        let route_id = RouteId(self.routes.len());

        let mut names = None;
        let leaf = self.tree.insert(path.as_ref(), &mut names)?;
        leaf.route = Some(route_id);

        self.routes.insert(
            route_id,
            Route {
                id: route_id,
                path: path.to_owned(),
                names,
                data,
            },
        );

        Ok(route_id)
    }

    /// Recognizes the location of the provided path on this router.
    pub fn recognize<'r>(&'r self, path: &'r str) -> Recognize<'r, T> {
        let recognize = self.tree.recognize(path.as_ref());

        let route = recognize.route.and_then(|RouteId(i)| {
            self.routes //
                .get_index(i)
                .map(|(_k, route)| route)
        });

        Recognize {
            route,
            path,
            params: recognize.params,
            wildcard: recognize.wildcard,
        }
    }
}

impl<T> std::ops::Index<RouteId> for Router<T> {
    type Output = Route<T>;

    fn index(&self, route: RouteId) -> &Self::Output {
        self.routes
            .get(&route)
            .unwrap_or_else(|| panic!("invalid route ID"))
    }
}

impl<T> std::ops::IndexMut<RouteId> for Router<T> {
    fn index_mut(&mut self, route: RouteId) -> &mut Self::Output {
        self.routes
            .get_mut(&route)
            .unwrap_or_else(|| panic!("invalid route ID"))
    }
}

/// The value for identifying the instance of `Route`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct RouteId(pub(crate) usize);

/// A value associated with a specific HTTP path.
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

/// A value that contains the recognition result of the router.
#[derive(Debug)]
pub struct Recognize<'r, T> {
    route: Option<&'r Route<T>>,
    path: &'r str,
    params: Vec<(usize, usize)>,
    wildcard: Option<(usize, usize)>,
}

impl<'r, T> Recognize<'r, T> {
    /// Returns a reference to the matched route if possible.
    pub fn route(&self) -> Option<&Route<T>> {
        self.route
    }

    /// Creates a `Params` that contains the captured parameter values, if possible.
    pub fn params(&self) -> Option<Params<'_>> {
        let names = self.route?.names.as_ref()?;
        Some(Params {
            names: Cow::Borrowed(names),
            path: Cow::Borrowed(&self.path),
            spans: Cow::Borrowed(&self.params),
            wildcard: self.wildcard,
        })
    }
}
