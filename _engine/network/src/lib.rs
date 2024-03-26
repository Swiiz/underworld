use std::marker::PhantomData;

use genmap::Handle;
use platform::{colored::Color, set_log_side};

pub mod connection;
pub mod ctx;
pub mod protocol;

pub trait NetworkSide: 'static + Copy + Clone {
    type ClientOnly<T>;
    type ServerOnly<T>;
    type OppositeSide;
    type ConnectionHandle: Clone;
    const ID: &'static str;
    fn client_only<T>(v: T) -> ClientOnly<Self, T>;
    fn server_only<T>(v: T) -> ServerOnly<Self, T>;
    fn set_log_side();
}
#[derive(Copy, Clone)]
pub struct Client;
#[derive(Copy, Clone)]
pub struct Server;
impl NetworkSide for Client {
    type ClientOnly<T> = T;
    type ServerOnly<T> = PhantomData<T>;
    type OppositeSide = Server;
    type ConnectionHandle = ();
    const ID: &'static str = "client";
    fn client_only<T>(v: T) -> ClientOnly<Self, T> {
        v
    }
    fn server_only<T>(_: T) -> ServerOnly<Self, T> {
        PhantomData
    }
    fn set_log_side() {
        set_log_side("CLIENT".to_string(), Color::Cyan);
    }
}
impl NetworkSide for Server {
    type ClientOnly<T> = PhantomData<T>;
    type ServerOnly<T> = T;
    type OppositeSide = Client;
    type ConnectionHandle = Handle;
    const ID: &'static str = "server";
    fn client_only<T>(_: T) -> ClientOnly<Self, T> {
        PhantomData
    }
    fn server_only<T>(v: T) -> ServerOnly<Self, T> {
        v
    }
    fn set_log_side() {
        set_log_side("SERVER".to_string(), Color::Yellow);
    }
}

pub type ClientOnly<S, T> = <S as NetworkSide>::ClientOnly<T>;
pub type ServerOnly<S, T> = <S as NetworkSide>::ServerOnly<T>;
