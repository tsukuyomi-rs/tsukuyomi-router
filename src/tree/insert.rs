use super::{Metadata, Node, ParamNames, StaticSegment, Tree, WildcardSegment};
use crate::error::Result;

impl Tree {
    pub(crate) fn insert(&mut self, path: &[u8], metadata: Metadata) -> Result<ParamNames> {
        let mut param_names = ParamNames::default();
        let mut cx = InsertContext {
            path,
            param_names: &mut param_names,
            metadata: Some(metadata),
        };
        cx.run(&mut self.root)?;
        Ok(param_names)
    }
}

struct InsertContext<'a> {
    path: &'a [u8],
    param_names: &'a mut ParamNames,
    metadata: Option<Metadata>,
}

impl<'a> InsertContext<'a> {
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
        self.param_names.names.push(self.path[1..end].to_owned());
        self.path = &self.path[end..];
        Ok(())
    }

    fn run(&mut self, mut current: &mut Node) -> Result<()> {
        loop {
            match self.path.get(0) {
                Some(b':') => {
                    if current.param_segment.is_some() {
                        self.extract_param()?;
                        current = &mut *current.param_segment.as_mut().unwrap();
                        continue;
                    }
                }
                Some(b'*') => {
                    self.insert_wildcard_segment(current)?;
                    return Ok(());
                }
                Some(_) => {
                    if let Some(pos) = self.find_static_segment(current)? {
                        current = &mut { current }.static_segments[pos].child;
                        continue;
                    }
                }
                None => (),
            }

            if self.path.len() > 0 {
                self.insert_remaining_path(current)?;
                return Ok(());
            }

            current.metadata.merge(
                self.metadata
                    .take()
                    .expect("the node has already been inserted"),
            )?;

            return Ok(());
        }
    }

    fn insert_remaining_path(&mut self, mut node: &mut Node) -> Result<()> {
        while let Some(c) = self.path.get(0) {
            match c {
                b':' => {
                    self.extract_param()?;
                    node = node
                        .param_segment
                        .get_or_insert_with(|| Box::new(Node::default()));
                }
                b'*' => {
                    self.insert_wildcard_segment(node)?;
                    return Ok(());
                }
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

        node.metadata = self
            .metadata
            .take()
            .expect("the node has already been inserted");

        Ok(())
    }

    fn insert_wildcard_segment(&mut self, node: &mut Node) -> Result<()> {
        let slug = &self.path[1..];
        if let Some(n) = node.wildcard_segments.iter_mut().find(|s| s.slug == slug) {
            n.child.metadata.merge(
                self.metadata
                    .take()
                    .expect("the node has already been inserted."),
            )?;
        } else {
            node.wildcard_segments.push(WildcardSegment {
                slug: slug.to_owned(),
                child: Node {
                    metadata: self
                        .metadata
                        .take()
                        .expect("the node has already been inserted."),
                    ..Default::default()
                },
            });
        }

        self.param_names.has_wildcard = true;
        Ok(())
    }
}

fn longest_common_prefix(s1: &[u8], s2: &[u8]) -> usize {
    s1.iter().zip(s2).take_while(|(c1, c2)| c1 == c2).count()
}

#[cfg(test)]
mod tests {
    use super::super::RouteId;
    use super::*;

    #[test]
    fn root() {
        let mut tree = Tree::default();
        tree.insert(
            b"/",
            Metadata {
                route: Some(RouteId(0)),
            },
        )
        .unwrap();

        assert_eq!(
            tree.root,
            Node {
                static_segments: vec![StaticSegment {
                    segment: "/".into(),
                    child: Node {
                        metadata: Metadata {
                            route: Some(RouteId(0))
                        },
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
        tree.insert(
            b"/foo",
            Metadata {
                route: Some(RouteId(0)),
            },
        )
        .unwrap();
        tree.insert(
            b"/foo/bar",
            Metadata {
                route: Some(RouteId(1)),
            },
        )
        .unwrap();

        assert_eq!(
            tree.root,
            Node {
                static_segments: vec![StaticSegment {
                    segment: "/foo".into(),
                    child: Node {
                        metadata: Metadata {
                            route: Some(RouteId(0))
                        },
                        static_segments: vec![StaticSegment {
                            segment: "/bar".into(),
                            child: Node {
                                metadata: Metadata {
                                    route: Some(RouteId(1))
                                },
                                ..Default::default()
                            },
                        }],
                        ..Default::default()
                    }
                },],
                ..Default::default()
            }
        );
    }

    #[test]
    fn different_suffix() {
        let mut tree = Tree::default();
        tree.insert(
            b"/foo/bar",
            Metadata {
                route: Some(RouteId(0)),
            },
        )
        .unwrap();
        tree.insert(
            b"/foo/zoo",
            Metadata {
                route: Some(RouteId(1)),
            },
        )
        .unwrap();

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
                                    metadata: Metadata {
                                        route: Some(RouteId(0))
                                    },
                                    ..Default::default()
                                },
                            },
                            StaticSegment {
                                segment: "zoo".into(),
                                child: Node {
                                    metadata: Metadata {
                                        route: Some(RouteId(1))
                                    },
                                    ..Default::default()
                                }
                            },
                        ],
                        ..Default::default()
                    }
                },],
                ..Default::default()
            }
        );
    }

    #[test]
    fn param() {
        let mut tree = Tree::default();
        let params = tree
            .insert(
                b"/posts/:post",
                Metadata {
                    route: Some(RouteId(0)),
                },
            )
            .unwrap();

        assert_eq!(
            tree.root,
            Node {
                static_segments: vec![StaticSegment {
                    segment: "/posts/".into(),
                    child: Node {
                        param_segment: Some(Box::new(Node {
                            metadata: Metadata {
                                route: Some(RouteId(0))
                            },
                            ..Default::default()
                        })),
                        ..Default::default()
                    },
                }],
                ..Default::default()
            }
        );

        assert_eq!(params.names, vec![b"post".to_owned()]);
    }

    #[test]
    fn param_with_suffix() {
        let mut tree = Tree::default();
        let params = tree
            .insert(
                b"/posts/:post/edit",
                Metadata {
                    route: Some(RouteId(0)),
                },
            )
            .unwrap();

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
                                    metadata: Metadata {
                                        route: Some(RouteId(0))
                                    },
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

        assert_eq!(params.names, vec![b"post".to_owned()]);
    }

    #[test]
    fn parameters() {
        let mut tree = Tree::default();
        let p1 = tree
            .insert(
                b"/users/:id",
                Metadata {
                    route: Some(RouteId(0)),
                },
            )
            .unwrap();
        let p2 = tree
            .insert(
                b"/users/:id/books",
                Metadata {
                    route: Some(RouteId(1)),
                },
            )
            .unwrap();
        let p3 = tree
            .insert(
                b"/users/admin/books",
                Metadata {
                    route: Some(RouteId(2)),
                },
            )
            .unwrap();

        assert_eq!(
            tree.root,
            Node {
                static_segments: vec![StaticSegment {
                    segment: "/users/".into(),
                    child: Node {
                        static_segments: vec![StaticSegment {
                            segment: "admin/books".into(),
                            child: Node {
                                metadata: Metadata {
                                    route: Some(RouteId(2))
                                },
                                ..Default::default()
                            },
                        }],
                        param_segment: Some(Box::new(Node {
                            metadata: Metadata {
                                route: Some(RouteId(0))
                            },
                            static_segments: vec![StaticSegment {
                                segment: "/books".into(),
                                child: Node {
                                    metadata: Metadata {
                                        route: Some(RouteId(1))
                                    },
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

        assert_eq!(p1.names, vec![b"id".to_owned()]);
        assert_eq!(p2.names, vec![b"id".to_owned()]);
        assert!(p3.names.is_empty());
    }

    #[test]
    fn wildcard() {
        let mut tree = Tree::default();
        tree.insert(
            b"/static/*",
            Metadata {
                route: Some(RouteId(0)),
            },
        )
        .unwrap();

        assert_eq!(
            tree.root,
            Node {
                static_segments: vec![StaticSegment {
                    segment: "/static/".into(),
                    child: Node {
                        wildcard_segments: vec![WildcardSegment {
                            slug: "".into(),
                            child: Node {
                                metadata: Metadata {
                                    route: Some(RouteId(0)),
                                },
                                ..Default::default()
                            },
                        }],
                        ..Default::default()
                    },
                }],
                ..Default::default()
            }
        );
    }

    #[test]
    fn wildcard_with_slug() {
        let mut tree = Tree::default();
        tree.insert(
            b"/static/*/index.html",
            Metadata {
                route: Some(RouteId(0)),
            },
        )
        .unwrap();

        assert_eq!(
            tree.root,
            Node {
                static_segments: vec![StaticSegment {
                    segment: "/static/".into(),
                    child: Node {
                        wildcard_segments: vec![WildcardSegment {
                            slug: "/index.html".into(),
                            child: Node {
                                metadata: Metadata {
                                    route: Some(RouteId(0)),
                                },
                                ..Default::default()
                            },
                        }],
                        ..Default::default()
                    },
                }],
                ..Default::default()
            }
        );
    }

    #[test]
    fn wildcard_with_different_slugs() {
        let mut tree = Tree::default();
        tree.insert(
            b"/static/*/index.html",
            Metadata {
                route: Some(RouteId(0)),
            },
        )
        .unwrap();
        tree.insert(
            b"/static/*/index.js",
            Metadata {
                route: Some(RouteId(1)),
            },
        )
        .unwrap();

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
                                    metadata: Metadata {
                                        route: Some(RouteId(0)),
                                    },
                                    ..Default::default()
                                },
                            },
                            WildcardSegment {
                                slug: "/index.js".into(),
                                child: Node {
                                    metadata: Metadata {
                                        route: Some(RouteId(1)),
                                    },
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

    #[test]
    fn failcase_duplicated_path() {
        let mut tree = Tree::default();
        assert!(tree
            .insert(
                b"/foo/bar",
                Metadata {
                    route: Some(RouteId(0))
                }
            )
            .is_ok());
        assert!(tree
            .insert(
                b"/foo/bar",
                Metadata {
                    route: Some(RouteId(1))
                }
            )
            .is_err());
    }

    #[test]
    fn failcase_duplicated_wildcard() {
        let mut tree = Tree::default();
        assert!(tree
            .insert(
                b"/*/a",
                Metadata {
                    route: Some(RouteId(0))
                }
            )
            .is_ok());
        assert!(tree
            .insert(
                b"/*/a",
                Metadata {
                    route: Some(RouteId(1))
                }
            )
            .is_err());
    }

}
