use std::marker::PhantomData;

pub mod connection;
pub mod ctx;
pub mod protocol;

pub trait NetworkSide: 'static {
    type ClientOnly<T>;
    type ServerOnly<T>;
    type OppositeSide;
    fn client_only<T>(v: T) -> ClientOnly<Self, T>;
    fn server_only<T>(v: T) -> ServerOnly<Self, T>;
}
pub struct Client;
pub struct Server;
impl NetworkSide for Client {
    type ClientOnly<T> = T;
    type ServerOnly<T> = PhantomData<T>;
    type OppositeSide = Server;
    fn client_only<T>(v: T) -> ClientOnly<Self, T> {
        v
    }
    fn server_only<T>(_: T) -> ServerOnly<Self, T> {
        PhantomData
    }
}
impl NetworkSide for Server {
    type ClientOnly<T> = PhantomData<T>;
    type ServerOnly<T> = T;
    type OppositeSide = Client;
    fn client_only<T>(_: T) -> ClientOnly<Self, T> {
        PhantomData
    }
    fn server_only<T>(v: T) -> ServerOnly<Self, T> {
        v
    }
}

pub type ClientOnly<S, T> = <S as NetworkSide>::ClientOnly<T>;
pub type ServerOnly<S, T> = <S as NetworkSide>::ServerOnly<T>;
