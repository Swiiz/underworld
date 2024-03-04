use std::sync::Arc;

use assets::SpriteSheets;
use cgmath::{Array, Matrix3, Vector2, Zero};
use graphics::{
    color::Color3,
    sprite::{renderer::SpriteRenderer, Sprite, SpriteParams},
    Graphics,
};
use platform::{Event, Platform, Window};
use world::World;

pub mod assets;
pub mod world;

pub struct App {
    window: Arc<Window>,
    graphics: Graphics<'static>,
    world: World,
}

impl App {
    pub fn new(platform: &Platform) -> Self {
        let window = platform.window.clone();
        let window_size = window.inner_size().into();

        let mut graphics = Graphics::new(window_size, window.clone());

        graphics
            .renderer
            .add_plugin(SpriteRenderer::<SpriteSheets>::new(
                &graphics.ctx,
                window_size,
            ));

        let world = World::generate();

        Self {
            window,
            graphics,
            world,
        }
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::Update => self.update(),
            Event::Render => self.render(),
            Event::Resize => {
                self.graphics.resize(self.window.inner_size().into());
            }
        }
    }

    fn update(&mut self) {}

    fn render(&mut self) {
        self.graphics.render(|frame| {
            frame.draw(
                Sprite {
                    sheet: SpriteSheets::Characters,
                    position: Vector2::zero(),
                    size: Vector2::from_value(1),
                },
                SpriteParams {
                    transform: Matrix3::from_translation(Vector2::from_value(-0.5)), // At -0.5, -0.5
                    tint: Color3::WHITE,
                    depth: 0.0,
                },
            );
        });
    }
}
