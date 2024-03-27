use std::marker::PhantomData;

use genmap::Handle;
use platform::{colored::Color, set_log_side};
use serde::{de::Visitor, Deserialize, Serialize};

pub mod connection;
pub mod ctx;
pub mod protocol;

pub trait NetworkSide: 'static + Copy + Clone + Serialize {
    type ClientOnly<T>;
    type ServerOnly<T>;
    type ClientOnlySerde<T: Serialize + for<'a> Deserialize<'a>>: Serialize
        + for<'a> Deserialize<'a>;
    type ServerOnlySerde<T: Serialize + for<'a> Deserialize<'a>>: Serialize
        + for<'a> Deserialize<'a>;
    type OppositeSide;
    type ConnectionHandle: Clone;
    const ID: &'static str;
    fn client_only<T>(v: T) -> ClientOnly<Self, T>;
    fn server_only<T>(v: T) -> ServerOnly<Self, T>;
    fn client_only_serde<T: Serialize + for<'a> Deserialize<'a>>(v: T) -> ClientOnlySerde<Self, T>;
    fn server_only_serde<T: Serialize + for<'a> Deserialize<'a>>(v: T) -> ServerOnlySerde<Self, T>;
    fn set_log_side();
}
#[derive(Copy, Clone)]
pub struct Client;
#[derive(Copy, Clone)]
pub struct Server;
impl NetworkSide for Client {
    type ClientOnly<T> = T;
    type ServerOnly<T> = PhantomData<T>;
    type ClientOnlySerde<T: Serialize + for<'a> Deserialize<'a>> = Self::ClientOnly<T>;
    type ServerOnlySerde<T: Serialize + for<'a> Deserialize<'a>> = Self::ServerOnly<T>;
    type OppositeSide = Server;
    type ConnectionHandle = ();
    const ID: &'static str = "client";
    fn client_only<T>(v: T) -> ClientOnly<Self, T> {
        v
    }
    fn server_only<T>(_: T) -> ServerOnly<Self, T> {
        PhantomData
    }
    fn client_only_serde<T: Serialize + for<'a> Deserialize<'a>>(v: T) -> ClientOnlySerde<Self, T> {
        v
    }
    fn server_only_serde<T: Serialize + for<'a> Deserialize<'a>>(_: T) -> ServerOnlySerde<Self, T> {
        PhantomData
    }
    fn set_log_side() {
        set_log_side("CLIENT".to_string(), Color::Cyan);
    }
}
impl NetworkSide for Server {
    type ClientOnly<T> = PhantomData<T>;
    type ServerOnly<T> = T;
    type ClientOnlySerde<T: Serialize + for<'a> Deserialize<'a>> = Self::ClientOnly<T>;
    type ServerOnlySerde<T: Serialize + for<'a> Deserialize<'a>> = Self::ServerOnly<T>;
    type OppositeSide = Client;
    type ConnectionHandle = Handle;
    const ID: &'static str = "server";
    fn client_only<T>(_: T) -> ClientOnly<Self, T> {
        PhantomData
    }
    fn server_only<T>(v: T) -> ServerOnly<Self, T> {
        v
    }
    fn client_only_serde<T: Serialize + for<'a> Deserialize<'a>>(_: T) -> ClientOnlySerde<Self, T> {
        PhantomData
    }
    fn server_only_serde<T: Serialize + for<'a> Deserialize<'a>>(v: T) -> ServerOnlySerde<Self, T> {
        v
    }
    fn set_log_side() {
        set_log_side("SERVER".to_string(), Color::Yellow);
    }
}

pub type ClientOnly<S, T> = <S as NetworkSide>::ClientOnly<T>;
pub type ServerOnly<S, T> = <S as NetworkSide>::ServerOnly<T>;
pub type ClientOnlySerde<S, T> = <S as NetworkSide>::ClientOnlySerde<T>;
pub type ServerOnlySerde<S, T> = <S as NetworkSide>::ServerOnlySerde<T>;

impl Serialize for Client {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(Self::ID)
    }
}
impl Serialize for Server {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(Self::ID)
    }
}

impl<'a> Deserialize<'a> for Client {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        panic!("This type cannot be deserialized!")
    }
}

impl<'a> Deserialize<'a> for Server {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        panic!("This type cannot be deserialized!")
    }
}
