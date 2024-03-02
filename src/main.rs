use cgmath::{Array, Matrix3, Vector2, Zero};
use graphics::{
    color::Color3,
    sprite::{renderer::SpriteRenderer, Sprite, SpriteParams, SpriteRegistry, SpriteSheetData},
    Graphics,
};
use platform::{Event, Platform, WindowBuilder};

fn main() {
    let platform = Platform::new(WindowBuilder::new().with_title("Underworld"));
    let window = platform.window.clone();
    let window_size = window.inner_size().into();

    let mut graphics = Graphics::new(window_size, window.clone());

    let mut sprites = SpriteRegistry::new();

    let characters_spritesheet = sprites.register(SpriteSheetData {
        path: "assets/characters.png".into(),
        sprite_px_size: Vector2 { x: 16, y: 16 },
    });
    let player_sprite = Sprite {
        sheet: characters_spritesheet,
        position: Vector2::zero(), // At 0*sprite_px_size, 0*sprite_px_size in the spritesheet
        size: Vector2::from_value(1), // Size of 1*sprite_px_size, 1*sprite_px_size in the spritesheet
    };

    graphics
        .renderer
        .add_plugin(SpriteRenderer::new(&graphics.ctx, sprites, window_size));

    platform.run(|event| match event {
        Event::Update => {
            // update!
        }
        Event::Render => {
            graphics.render(|frame| {
                frame.draw(
                    player_sprite,
                    SpriteParams {
                        transform: Matrix3::from_translation(Vector2::from_value(-0.5)), // At -0.5, -0.5
                        tint: Color3::WHITE,
                        depth: 0.0,
                    },
                );
            });
        }
        Event::Resize => {
            graphics.resize(window.inner_size().into());
        }
    })
}
