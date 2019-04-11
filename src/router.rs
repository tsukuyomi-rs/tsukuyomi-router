use crate::{
    error::Result, //
    tree::{Metadata, ParamNames, RouteId, Tree},
};
use indexmap::IndexMap;
use std::borrow::Cow;

#[derive(Debug)]
struct Route<T> {
    data: T,
    names: ParamNames,
}

/// An HTTP router.
#[derive(Debug)]
pub struct Router<T> {
    tree: Tree,
    routes: IndexMap<String, Route<T>>,
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

    /// Adds a route to the router with the specified path.
    pub fn add_route(&mut self, path: &str, data: T) -> Result<&mut Self> {
        let names = self.tree.insert(
            path.as_ref(),
            Metadata {
                route: Some(RouteId(self.routes.len())),
            },
        )?;
        self.routes.insert(path.to_owned(), Route { data, names });
        Ok(self)
    }

    /// Finds the route that maches to the provided path.
    pub fn find_route<'r>(&'r self, path: &'r str) -> RouterResult<'r, T> {
        let recognize = self.tree.recognize(path.as_ref());

        let route = recognize
            .route
            .and_then(|RouteId(i)| self.routes.get_index(i).map(|(_k, route)| route));

        let data = route.map(|r| &r.data);
        let names = route.map(|r| &r.names);
        let wildcard = names.map_or(None, |names| {
            if names.has_wildcard() {
                recognize.wildcard.map(|(s, e)| Cow::Borrowed(&path[s..e]))
            } else {
                None
            }
        });

        RouterResult {
            data,
            params: Params {
                path: Cow::Borrowed(path),
                names: names.map(Cow::Borrowed),
                spans: recognize.params,
            },
            wildcard,
            _p: (),
        }
    }
}

/// The type that contains the result of router.
#[derive(Debug)]
pub struct RouterResult<'r, T> {
    pub data: Option<&'r T>,
    pub params: Params<'r>,
    pub wildcard: Option<Cow<'r, str>>,
    _p: (),
}

#[derive(Debug)]
pub struct Params<'r> {
    path: Cow<'r, str>,
    names: Option<Cow<'r, ParamNames>>,
    spans: Vec<(usize, usize)>,
}

impl<'r> Params<'r> {
    pub fn is_empty(&self) -> bool {
        self.names.is_none() || self.spans.is_empty()
    }

    pub fn len(&self) -> usize {
        match self.names {
            Some(..) => self.spans.len(),
            None => 0,
        }
    }

    pub fn get(&self, i: usize) -> Option<&str> {
        match self.names {
            Some(..) => self.spans.get(i).map(|&(s, e)| &self.path[s..e]),
            None => None,
        }
    }

    pub fn name(&self, name: &str) -> Option<&str> {
        match self.names {
            Some(ref names) => self
                .spans
                .get(names.position(name)?)
                .map(|&(s, e)| &self.path[s..e]),
            None => None,
        }
    }

    pub fn into_owned(self) -> Params<'static> {
        Params {
            path: Cow::Owned(self.path.into_owned()),
            names: self.names.map(|names| Cow::Owned(names.into_owned())),
            spans: self.spans,
        }
    }
}

impl<'r> std::ops::Index<usize> for Params<'r> {
    type Output = str;

    fn index(&self, i: usize) -> &Self::Output {
        self.get(i).unwrap_or_else(|| panic!("out of range"))
    }
}

impl<'r, 's> std::ops::Index<&'s str> for Params<'r> {
    type Output = str;

    fn index(&self, name: &'s str) -> &Self::Output {
        self.name(name)
            .unwrap_or_else(|| panic!("invalid parameter name"))
    }
}
