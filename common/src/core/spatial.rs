use std::ops::{Deref, DerefMut};

use cgmath::Vector2;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Position(pub Vector2<f32>);
#[rustfmt::skip] impl Deref for Position { type Target = Vector2<f32>; fn deref(&self) -> &Self::Target { &self.0 } }
#[rustfmt::skip] impl DerefMut for Position { fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 } }
