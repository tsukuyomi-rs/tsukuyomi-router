use super::{Node, StaticSegment, Tree, WildcardSegment};
use crate::{error::Result, param::ParamNames};

impl Tree {
    pub(crate) fn insert(
        &mut self,
        path: &[u8],
        names: &mut Option<ParamNames>,
    ) -> Result<&mut Node> {
        let mut cx = InsertContext {
            path,
            names: &mut *names,
        };
        cx.run(&mut self.root)
    }
}

struct InsertContext<'a> {
    path: &'a [u8],
    names: &'a mut Option<ParamNames>,
}

impl<'a> InsertContext<'a> {
    fn run<'n>(&mut self, mut current: &'n mut Node) -> Result<&'n mut Node> {
        loop {
            match self.path.get(0) {
                Some(b':') => {
                    if current.param_segment.is_some() {
                        self.extract_param()?;
                        current = &mut *current.param_segment.as_mut().unwrap();
                        continue;
                    }
                }
                Some(b'*') => return self.insert_wildcard_segment(current),
                Some(_) => {
                    if let Some(pos) = self.find_static_segment(current)? {
                        current = &mut { current }.static_segments[pos].child;
                        continue;
                    }
                }
                None => (),
            }

            if self.path.len() > 0 {
                return self.insert_remaining_path(current);
            }

            return Ok(current);
        }
    }

    fn find_static_segment(&mut self, current: &mut Node) -> Result<Option<usize>> {
        for (i, s) in &mut current.static_segments.iter_mut().enumerate() {
            let lcp = longest_common_prefix(&s.segment, self.path);
            if lcp > 0 {
                if lcp < s.segment.len() {
                    s.split_at(lcp);
                }
                self.path = &self.path[lcp..];
                return Ok(Some(i));
            }
        }
        Ok(None)
    }

    fn extract_param(&mut self) -> Result<()> {
        let end = self
            .path
            .iter()
            .position(|&c| c == b'/')
            .unwrap_or_else(|| self.path.len());
        self.names
            .get_or_insert_with(Default::default)
            .names
            .push(self.path[1..end].to_owned());
        self.path = &self.path[end..];
        Ok(())
    }

    fn insert_remaining_path<'n>(&mut self, mut node: &'n mut Node) -> Result<&'n mut Node> {
        while let Some(c) = self.path.get(0) {
            match c {
                b':' => {
                    self.extract_param()?;
                    node = node
                        .param_segment
                        .get_or_insert_with(|| Box::new(Node::default()));
                }
                b'*' => return self.insert_wildcard_segment(node),
                _ => {
                    let end =
                        if let Some(n) = self.path.iter().position(|&c| c == b':' || c == b'*') {
                            if let Some(b'/') = self.path.get(n - 1) {
                                n
                            } else {
                                bail!("the position of wildcard parameter (':' or '*') is invalid");
                            }
                        } else {
                            self.path.len()
                        };

                    node.static_segments.push(StaticSegment {
                        segment: self.path[..end].to_owned(),
                        child: Node::default(),
                    });
                    self.path = &self.path[end..];
                    node = &mut node.static_segments.iter_mut().last().unwrap().child;
                }
            }
        }

        Ok(node)
    }

    fn insert_wildcard_segment<'n>(&mut self, node: &'n mut Node) -> Result<&'n mut Node> {
        self.names.get_or_insert_with(Default::default).has_wildcard = true;

        let slug = &self.path[1..];
        if let Some(pos) = node
            .wildcard_segments
            .iter_mut()
            .position(|s| s.slug == slug)
        {
            return Ok(&mut node.wildcard_segments[pos].child);
        }

        node.wildcard_segments.push(WildcardSegment {
            slug: slug.to_owned(),
            child: Node::default(),
        });

        Ok(&mut node.wildcard_segments.iter_mut().last().unwrap().child)
    }
}

fn longest_common_prefix(s1: &[u8], s2: &[u8]) -> usize {
    s1.iter().zip(s2).take_while(|(c1, c2)| c1 == c2).count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::endpoint::EndpointId;

    #[test]
    fn root() {
        let mut tree = Tree::default();
        tree.insert(b"/", &mut None).unwrap().route = Some(EndpointId(0));

        assert_eq!(
            tree.root,
            Node {
                static_segments: vec![StaticSegment {
                    segment: "/".into(),
                    child: Node {
                        route: Some(EndpointId(0)),
                        ..Default::default()
                    }
                },],
                ..Default::default()
            }
        );
    }

    #[test]
    fn inclusive() {
        let mut tree = Tree::default();
        tree.insert(b"/foo", &mut None).unwrap().route = Some(EndpointId(0));
        tree.insert(b"/foo/bar", &mut None).unwrap().route = Some(EndpointId(1));

        assert_eq!(
            tree.root,
            Node {
                static_segments: vec![StaticSegment {
                    segment: "/foo".into(),
                    child: Node {
                        route: Some(EndpointId(0)),
                        static_segments: vec![StaticSegment {
                            segment: "/bar".into(),
                            child: Node {
                                route: Some(EndpointId(1)),
                                ..Default::default()
                            },
                        }],
                        ..Default::default()
                    }
                }],
                ..Default::default()
            }
        );
    }

    #[test]
    fn different_suffix() {
        let mut tree = Tree::default();
        tree.insert(b"/foo/bar", &mut None).unwrap().route = Some(EndpointId(0));
        tree.insert(b"/foo/zoo", &mut None).unwrap().route = Some(EndpointId(1));

        assert_eq!(
            tree.root,
            Node {
                static_segments: vec![StaticSegment {
                    segment: "/foo/".into(),
                    child: Node {
                        static_segments: vec![
                            StaticSegment {
                                segment: "bar".into(),
                                child: Node {
                                    route: Some(EndpointId(0)),
                                    ..Default::default()
                                },
                            },
                            StaticSegment {
                                segment: "zoo".into(),
                                child: Node {
                                    route: Some(EndpointId(1)),
                                    ..Default::default()
                                }
                            },
                        ],
                        ..Default::default()
                    }
                }],
                ..Default::default()
            }
        );
    }

    #[test]
    fn param() {
        let mut tree = Tree::default();
        let mut params = None;
        tree.insert(b"/posts/:post", &mut params).unwrap().route = Some(EndpointId(0));

        assert_eq!(
            tree.root,
            Node {
                static_segments: vec![StaticSegment {
                    segment: "/posts/".into(),
                    child: Node {
                        param_segment: Some(Box::new(Node {
                            route: Some(EndpointId(0)),
                            ..Default::default()
                        })),
                        ..Default::default()
                    },
                }],
                ..Default::default()
            }
        );

        assert_eq!(params.unwrap().names, vec![b"post".to_owned()]);
    }

    #[test]
    fn param_with_suffix() {
        let mut tree = Tree::default();
        let mut params = None;
        tree.insert(b"/posts/:post/edit", &mut params)
            .unwrap()
            .route = Some(EndpointId(0));

        assert_eq!(
            tree.root,
            Node {
                static_segments: vec![StaticSegment {
                    segment: "/posts/".into(),
                    child: Node {
                        param_segment: Some(Box::new(Node {
                            static_segments: vec![StaticSegment {
                                segment: "/edit".into(),
                                child: Node {
                                    route: Some(EndpointId(0)),
                                    ..Default::default()
                                },
                            }],
                            ..Default::default()
                        })),
                        ..Default::default()
                    },
                }],
                ..Default::default()
            }
        );

        assert_eq!(params.unwrap().names, vec![b"post".to_owned()]);
    }

    #[test]
    fn parameters() {
        let mut tree = Tree::default();
        let (mut p1, mut p2, mut p3) = (None, None, None);
        tree.insert(b"/users/:id", &mut p1).unwrap().route = Some(EndpointId(0));
        tree.insert(b"/users/:id/books", &mut p2).unwrap().route = Some(EndpointId(1));
        tree.insert(b"/users/admin/books", &mut p3).unwrap().route = Some(EndpointId(2));

        assert_eq!(
            tree.root,
            Node {
                static_segments: vec![StaticSegment {
                    segment: "/users/".into(),
                    child: Node {
                        static_segments: vec![StaticSegment {
                            segment: "admin/books".into(),
                            child: Node {
                                route: Some(EndpointId(2)),
                                ..Default::default()
                            },
                        }],
                        param_segment: Some(Box::new(Node {
                            route: Some(EndpointId(0)),
                            static_segments: vec![StaticSegment {
                                segment: "/books".into(),
                                child: Node {
                                    route: Some(EndpointId(1)),
                                    ..Default::default()
                                },
                            }],
                            ..Default::default()
                        })),
                        ..Default::default()
                    },
                }],
                ..Default::default()
            }
        );

        assert_eq!(p1.unwrap().names, vec![b"id".to_owned()]);
        assert_eq!(p2.unwrap().names, vec![b"id".to_owned()]);
        assert!(p3.is_none());
    }

    #[test]
    fn wildcard() {
        let mut tree = Tree::default();
        let mut params = None;
        tree.insert(b"/static/*", &mut params).unwrap().route = Some(EndpointId(0));

        assert_eq!(
            tree.root,
            Node {
                static_segments: vec![StaticSegment {
                    segment: "/static/".into(),
                    child: Node {
                        wildcard_segments: vec![WildcardSegment {
                            slug: "".into(),
                            child: Node {
                                route: Some(EndpointId(0)),
                                ..Default::default()
                            },
                        }],
                        ..Default::default()
                    },
                }],
                ..Default::default()
            }
        );
        assert!(params.map_or(false, |p| p.has_wildcard));
    }

    #[test]
    fn wildcard_with_slug() {
        let mut tree = Tree::default();
        let mut params = None;
        tree.insert(b"/static/*/index.html", &mut params)
            .unwrap()
            .route = Some(EndpointId(0));

        assert_eq!(
            tree.root,
            Node {
                static_segments: vec![StaticSegment {
                    segment: "/static/".into(),
                    child: Node {
                        wildcard_segments: vec![WildcardSegment {
                            slug: "/index.html".into(),
                            child: Node {
                                route: Some(EndpointId(0)),
                                ..Default::default()
                            },
                        }],
                        ..Default::default()
                    },
                }],
                ..Default::default()
            }
        );
        assert!(params.map_or(false, |p| p.has_wildcard));
    }

    #[test]
    fn wildcard_with_different_slugs() {
        let mut tree = Tree::default();
        tree.insert(b"/static/*/index.html", &mut None)
            .unwrap()
            .route = Some(EndpointId(0));
        tree.insert(b"/static/*/index.js", &mut None).unwrap().route = Some(EndpointId(1));

        assert_eq!(
            tree.root,
            Node {
                static_segments: vec![StaticSegment {
                    segment: "/static/".into(),
                    child: Node {
                        wildcard_segments: vec![
                            WildcardSegment {
                                slug: "/index.html".into(),
                                child: Node {
                                    route: Some(EndpointId(0)),
                                    ..Default::default()
                                },
                            },
                            WildcardSegment {
                                slug: "/index.js".into(),
                                child: Node {
                                    route: Some(EndpointId(1)),
                                    ..Default::default()
                                },
                            },
                        ],
                        ..Default::default()
                    },
                }],
                ..Default::default()
            }
        );
    }
}
