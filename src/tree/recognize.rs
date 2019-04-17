use super::{Node, RouteId, ScopeId, StaticSegment, Tree, WildcardSegment};

#[derive(Debug)]
pub(crate) struct Recognize {
    pub(crate) route: Option<RouteId>,
    pub(crate) params: Vec<(usize, usize)>,
    pub(crate) wildcard: Option<(usize, usize)>,
    pub(crate) scopes: Vec<ScopeId>,
    _p: (),
}

impl Tree {
    pub(crate) fn recognize<'p>(&'p self, path: &'p [u8]) -> Recognize {
        let mut params = vec![];
        let mut wildcard = None;
        let mut scopes = vec![];

        let mut cx = RecognizeContext {
            path,
            offset: 0,
            scopes: &mut scopes,
            params: &mut params,
            wildcard: &mut wildcard,
        };
        let node = cx.run(&self.root);

        let route = if path.len() == cx.offset {
            node.route
        } else {
            None
        };

        Recognize {
            route,
            scopes,
            params,
            wildcard,
            _p: (),
        }
    }
}

#[derive(Debug)]
struct RecognizeContext<'a> {
    path: &'a [u8],
    offset: usize,
    scopes: &'a mut Vec<ScopeId>,
    params: &'a mut Vec<(usize, usize)>,
    wildcard: &'a mut Option<(usize, usize)>,
}

impl<'a> RecognizeContext<'a> {
    fn run<'n>(&mut self, mut current: &'n Node) -> &'n Node {
        loop {
            if let Some(scope) = current.scope {
                self.scopes.push(scope);
            }

            if self.path.len() <= self.offset {
                return current;
            }

            if let Some(ch) = self.find_static_segment(current) {
                current = ch;
                continue;
            }

            if let Some(ch) = &current.param_segment {
                let end = self
                    .path
                    .iter()
                    .skip(self.offset)
                    .position(|&c| c == b'/')
                    .map(|pos| self.offset + pos)
                    .unwrap_or_else(|| self.path.len());
                self.params.push((self.offset, end));
                self.offset = end;

                current = &*ch;
                continue;
            }

            if let Some(ch) = self.find_wildcard_segment(current) {
                return ch;
            }

            return current;
        }
    }

    fn find_static_segment<'n>(&mut self, current: &'n Node) -> Option<&'n Node> {
        for StaticSegment {
            ref segment,
            ref child,
        } in &current.static_segments
        {
            if self.offset + segment.len() <= self.path.len()
                && self.path[self.offset..self.offset + segment.len()] == segment[..]
            {
                self.offset += segment.len();
                return Some(child);
            }
        }
        None
    }

    fn find_wildcard_segment<'n>(&mut self, current: &'n Node) -> Option<&'n Node> {
        for WildcardSegment {
            ref slug,
            ref child,
        } in &current.wildcard_segments
        {
            if self.offset + slug.len() <= self.path.len()
                && self.path[self.path.len() - slug.len()..] == slug[..]
            {
                *self.wildcard = Some((self.offset, self.path.len() - slug.len()));
                self.offset = self.path.len();
                return Some(child);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let tree = Tree::default();
        assert!(tree.recognize(b"/").route.is_none());
    }

    #[test]
    fn root_node() {
        let mut tree = Tree::default();
        tree.insert(b"/", &mut None).unwrap().route = Some(RouteId(0));

        assert_eq!(tree.recognize(b"/").route, Some(RouteId(0)));
    }

    #[test]
    fn nested_url() {
        let mut tree = Tree::default();
        tree.insert(b"/books/23/chapters", &mut None).unwrap().route = Some(RouteId(0));

        assert_eq!(
            tree.recognize(b"/books/23/chapters").route,
            Some(RouteId(0))
        );

        assert!(tree.recognize(b"/").route.is_none());
        assert!(tree.recognize(b"/books/34/chapters").route.is_none());
        assert!(tree.recognize(b"/books/23/chapters/").route.is_none());
    }

    #[test]
    fn multiple_routes() {
        let mut tree = Tree::default();
        tree.insert(b"/domains/mime", &mut None).unwrap().route = Some(RouteId(0));
        tree.insert(b"/domains/yours", &mut None).unwrap().route = Some(RouteId(1));

        assert_eq!(tree.recognize(b"/domains/mime").route, Some(RouteId(0)));
        assert_eq!(tree.recognize(b"/domains/yours").route, Some(RouteId(1)));

        assert!(tree.recognize(b"/domains/").route.is_none());
        assert!(tree.recognize(b"/domains/me").route.is_none());
    }

    #[test]
    fn single_param() {
        let mut tree = Tree::default();
        tree.insert(b"/posts/:post", &mut None).unwrap().route = Some(RouteId(0));

        let recognize = tree.recognize(b"/posts/42");
        assert_eq!(recognize.route, Some(RouteId(0)));
        assert_eq!(recognize.params[0], (7, 9));
    }

    #[test]
    fn param_with_suffix() {
        let mut tree = Tree::default();
        tree.insert(b"/posts/:post/edit", &mut None).unwrap().route = Some(RouteId(0));

        let recognize = tree.recognize(b"/posts/42/edit");
        assert_eq!(recognize.route, Some(RouteId(0)));
        assert_eq!(recognize.params[0], (7, 9));

        assert!(tree.recognize(b"/posts/42/new").route.is_none());
    }

    #[test]
    fn many_params() {
        let mut tree = Tree::default();
        tree.insert(b"/:year/:month/:date", &mut None)
            .unwrap()
            .route = Some(RouteId(0));

        let recognize = tree.recognize(b"/2019/05/01");
        assert_eq!(recognize.route, Some(RouteId(0)));
        assert_eq!(recognize.params[0], (1, 5));
        assert_eq!(recognize.params[1], (6, 8));
        assert_eq!(recognize.params[2], (9, 11));
    }

    #[test]
    fn param_with_static_segment() {
        let mut tree = Tree::default();
        tree.insert(b"/posts/new", &mut None).unwrap().route = Some(RouteId(0));
        tree.insert(b"/posts/:post", &mut None).unwrap().route = Some(RouteId(1));

        assert_eq!(tree.recognize(b"/posts/new").route, Some(RouteId(0)));

        let recognize = tree.recognize(b"/posts/10");
        assert_eq!(recognize.route, Some(RouteId(1)));
        assert_eq!(recognize.params[0], (7, 9));
    }

    #[test]
    fn wildcard() {
        let mut tree = Tree::default();
        tree.insert(b"/static/*", &mut None).unwrap().route = Some(RouteId(0));

        let recognize = tree.recognize(b"/static/path/to/index.html");
        assert_eq!(recognize.route, Some(RouteId(0)));
        assert_eq!(recognize.wildcard, Some((8, 26)));
    }

    #[test]
    fn wildcard_with_slug() {
        let mut tree = Tree::default();
        tree.insert(b"/static/*/index.html", &mut None)
            .unwrap()
            .route = Some(RouteId(0));

        let recognize = tree.recognize(b"/static/path/to/index.html");
        assert_eq!(recognize.route, Some(RouteId(0)));
        assert_eq!(recognize.wildcard, Some((8, 15)));
    }

    #[test]
    fn many_wildcards() {
        let mut tree = Tree::default();
        tree.insert(b"/static/*/index.html", &mut None)
            .unwrap()
            .route = Some(RouteId(0));
        tree.insert(b"/static/*.html", &mut None).unwrap().route = Some(RouteId(1));

        assert_eq!(
            tree.recognize(b"/static/path/to/index.html").route,
            Some(RouteId(0))
        );
        assert_eq!(
            tree.recognize(b"/static/about.html").route,
            Some(RouteId(1))
        );
    }

    #[test]
    fn scopes() {
        let mut tree = Tree::default();
        tree.insert(b"/path/to/index.html", &mut None) //
            .unwrap()
            .route = Some(RouteId(0));

        tree.insert(b"/path/", &mut None).unwrap().scope = Some(ScopeId(0));
        tree.insert(b"/path/to", &mut None) //
            .unwrap()
            .scope = Some(ScopeId(1));

        assert_eq!(tree.recognize(b"/path/foo").scopes, vec![ScopeId(0)]);
        assert_eq!(
            tree.recognize(b"/path/to/index").scopes,
            vec![ScopeId(0), ScopeId(1)]
        );
        assert_eq!(tree.recognize(b"/pattern").scopes, vec![]);
    }
}
