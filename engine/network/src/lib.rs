use std::marker::PhantomData;

use genmap::Handle;
use platform::{colored::Color, set_log_side};
use serde::{
    de::{DeserializeOwned, Visitor},
    Deserialize, Serialize,
};

pub mod connection;
pub mod ctx;
pub mod protocol;

pub trait BaseNetworkSide: 'static + Copy + Clone {
    type ClientOnly<T>;
    type ServerOnly<T>;
    type OppositeSide;
    type ConnectionHandle: Clone;

    const ID: &'static str;

    type ClientOnlySerde<T: Serialize + for<'a> Deserialize<'a>>: Serialize
        + for<'a> Deserialize<'a>;
    type ServerOnlySerde<T: Serialize + for<'a> Deserialize<'a>>: Serialize
        + for<'a> Deserialize<'a>;
    fn client_only<T>(v: T) -> ClientOnly<Self, T>;
    fn server_only<T>(v: T) -> ServerOnly<Self, T>;
    fn client_only_serde<T: Serialize + for<'a> Deserialize<'a>>(v: T) -> ClientOnlySerde<Self, T>;
    fn server_only_serde<T: Serialize + for<'a> Deserialize<'a>>(v: T) -> ServerOnlySerde<Self, T>;
    fn set_log_side();
}

pub trait NetworkSide: BaseNetworkSide + Serialize + for<'a> Deserialize<'a> {}
impl<T: Serialize + for<'a> Deserialize<'a> + BaseNetworkSide> NetworkSide for T {}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct Client;
#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct Server;
impl BaseNetworkSide for Client {
    type ClientOnly<T> = T;
    type ServerOnly<T> = SerdePhantomData<T>;
    type ClientOnlySerde<T: Serialize + for<'a> Deserialize<'a>> = Self::ClientOnly<T>;
    type ServerOnlySerde<T: Serialize + for<'a> Deserialize<'a>> = Self::ServerOnly<T>;
    type OppositeSide = Server;
    type ConnectionHandle = ();
    const ID: &'static str = "client";
    fn client_only<T>(v: T) -> ClientOnly<Self, T> {
        v
    }
    fn server_only<T>(_: T) -> ServerOnly<Self, T> {
        SerdePhantomData::new()
    }
    fn client_only_serde<T: Serialize + for<'a> Deserialize<'a>>(v: T) -> ClientOnlySerde<Self, T> {
        v
    }
    fn server_only_serde<T: Serialize + for<'a> Deserialize<'a>>(_: T) -> ServerOnlySerde<Self, T> {
        SerdePhantomData::new()
    }
    fn set_log_side() {
        set_log_side("CLIENT".to_string(), Color::Cyan);
    }
}
impl BaseNetworkSide for Server {
    type ClientOnly<T> = SerdePhantomData<T>;
    type ServerOnly<T> = T;
    type ClientOnlySerde<T: Serialize + for<'a> Deserialize<'a>> = Self::ClientOnly<T>;
    type ServerOnlySerde<T: Serialize + for<'a> Deserialize<'a>> = Self::ServerOnly<T>;
    type OppositeSide = Client;
    type ConnectionHandle = Handle;
    const ID: &'static str = "server";
    fn client_only<T>(_: T) -> ClientOnly<Self, T> {
        SerdePhantomData::new()
    }
    fn server_only<T>(v: T) -> ServerOnly<Self, T> {
        v
    }
    fn client_only_serde<T: Serialize + for<'a> Deserialize<'a>>(_: T) -> ClientOnlySerde<Self, T> {
        SerdePhantomData::new()
    }
    fn server_only_serde<T: Serialize + for<'a> Deserialize<'a>>(v: T) -> ServerOnlySerde<Self, T> {
        v
    }
    fn set_log_side() {
        set_log_side("SERVER".to_string(), Color::Yellow);
    }
}

pub type ClientOnly<S, T> = <S as BaseNetworkSide>::ClientOnly<T>;
pub type ServerOnly<S, T> = <S as BaseNetworkSide>::ServerOnly<T>;
pub type ClientOnlySerde<S, T> = <S as BaseNetworkSide>::ClientOnlySerde<T>;
pub type ServerOnlySerde<S, T> = <S as BaseNetworkSide>::ServerOnlySerde<T>;

#[derive(Serialize)]
pub struct SerdePhantomData<T> {
    _marker: PhantomData<T>,
}

impl<T> SerdePhantomData<T> {
    pub const fn new() -> Self {
        SerdePhantomData {
            _marker: PhantomData,
        }
    }
}

impl<'a, T> Deserialize<'a> for SerdePhantomData<T> {
    fn deserialize<D>(_: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        Ok(SerdePhantomData::new())
    }
}
