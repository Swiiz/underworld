use serde::{de::DeserializeOwned, Serialize};

pub trait HandleType: Sized {
    type Handle<T: HandleTypeUnion>: DeserializeOwned + Serialize;
}
pub trait HandleTypeUnion {
    type Static: DeserializeOwned + Serialize;
    type Dynamic: DeserializeOwned + Serialize;
}

pub struct StaticHandle;
pub struct DynamicHandle;

impl HandleType for StaticHandle {
    type Handle<T: HandleTypeUnion> = T::Static;
}
impl HandleType for DynamicHandle {
    type Handle<T: HandleTypeUnion> = T::Dynamic;
}
