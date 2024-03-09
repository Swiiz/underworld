use std::marker::PhantomData;

use crate::commons::Connection;

pub struct NetworkClient<P> {
    connection: Box<dyn Connection>,
    _packet_marker: PhantomData<P>,
}

impl<P> NetworkClient<P> {
    pub fn new(conn: impl Connection + 'static) -> Self {
        Self {
            connection: Box::new(conn),
            _packet_marker: PhantomData,
        }
    }

    pub fn update(&mut self) {}
}
