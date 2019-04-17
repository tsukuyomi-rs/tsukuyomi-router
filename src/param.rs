use std::borrow::Cow;

#[derive(Clone, Debug, Default)]
pub struct ParamNames {
    pub(crate) names: Vec<Vec<u8>>,
    pub(crate) has_wildcard: bool,
}

impl ParamNames {
    /// Returns the position of the specified parameter.
    pub fn position(&self, name: &str) -> Option<usize> {
        self.names.iter().position(|n| *n == name.as_bytes())
    }
}

/// A set of captured parameter values from an HTTP path.
#[derive(Debug, Clone)]
pub struct Params<'r> {
    pub(crate) path: Cow<'r, str>,
    pub(crate) names: Cow<'r, ParamNames>,
    pub(crate) spans: Cow<'r, Vec<(usize, usize)>>,
    pub(crate) wildcard: Option<(usize, usize)>,
}

impl<'r> Params<'r> {
    /// Finds a parameter value by position.
    pub fn get(&self, i: usize) -> Option<&str> {
        self.spans.get(i).map(|&(s, e)| &self.path[s..e])
    }

    /// Finds a parameter value by name.
    pub fn name(&self, name: &str) -> Option<&str> {
        match name {
            "*" => self.get_wildcard(),
            name => self.names.position(name).and_then(|i| self.get(i)),
        }
    }

    /// Returns the value of extracted wildcard parameter if possible.
    pub fn get_wildcard(&self) -> Option<&str> {
        if self.names.has_wildcard {
            self.wildcard.map(|(s, e)| &self.path[s..e])
        } else {
            None
        }
    }

    /// Clones the internal values if they are borrowed.
    pub fn into_owned(self) -> Params<'static> {
        Params {
            path: Cow::Owned(self.path.into_owned()),
            names: Cow::Owned(self.names.into_owned()),
            spans: Cow::Owned(self.spans.into_owned()),
            wildcard: self.wildcard,
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
            .unwrap_or_else(|| panic!("invalid param name"))
    }
}
