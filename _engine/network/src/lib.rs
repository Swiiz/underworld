use commons::NetworkSide;

pub mod client;
pub mod commons;
pub mod server;

pub type Network<S> = <S as NetworkSide>::Context;
