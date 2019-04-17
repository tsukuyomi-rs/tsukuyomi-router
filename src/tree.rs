mod insert;
mod recognize;

use crate::router::RouteId;

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
