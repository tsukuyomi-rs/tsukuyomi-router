mod insert;
mod recognize;

use crate::RouteId;

#[derive(Debug, Default)]
pub(crate) struct Tree {
    root: Node,
}

#[derive(Debug, Default)]
#[cfg_attr(test, derive(PartialEq))]
pub(crate) struct Node {
    static_segments: Vec<StaticSegment>,
    param_segment: Option<Box<Node>>,
    wildcard_segments: Vec<WildcardSegment>,

    pub(crate) route: Option<RouteId>,
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
struct StaticSegment {
    segment: Vec<u8>,
    child: Node,
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
struct WildcardSegment {
    slug: Vec<u8>,
    child: Node,
}

impl StaticSegment {
    fn split_at(&mut self, i: usize) {
        let (seg1, seg2) = self.segment.split_at(i);
        *self = Self {
            segment: seg1.to_owned(),
            child: Node {
                static_segments: vec![Self {
                    segment: seg2.to_owned(),
                    child: std::mem::replace(&mut self.child, Node::default()),
                }],
                ..Default::default()
            },
        };
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ParamNames {
    names: Vec<Vec<u8>>,
    has_wildcard: bool,
}

impl ParamNames {
    pub(crate) fn position(&self, name: impl AsRef<[u8]>) -> Option<usize> {
        self.names.iter().position(|n| *n == name.as_ref())
    }

    pub(crate) fn has_wildcard(&self) -> bool {
        self.has_wildcard
    }
}
