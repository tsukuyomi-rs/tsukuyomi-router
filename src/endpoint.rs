use crate::param::ParamNames;
use std::ops::{Deref, DerefMut};

/// The identifier of `Endpoint`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct EndpointId(pub(crate) usize);

/// An endpoint in `Router`.
#[derive(Debug)]
pub struct Endpoint<T> {
    pub(crate) id: EndpointId,
    pub(crate) path: String,
    pub(crate) names: Option<ParamNames>,
    pub(crate) data: T,
}

impl<T> Endpoint<T> {
    /// Returns the identifier associated with this endpoint.
    pub fn id(&self) -> EndpointId {
        self.id
    }

    /// Returns the original path of this endpoint.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns a reference to the data associated with this endpoint.
    pub fn data(&self) -> &T {
        &self.data
    }

    /// Returns a reference to the data associated with this endpoint.
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

impl<T> Deref for Endpoint<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.data()
    }
}

impl<T> DerefMut for Endpoint<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data_mut()
    }
}
