use common::logger::debug;
use graphics::ctx::Frame;
use winit::keyboard::KeyCode;

use crate::core::{assets::ClientAssets, platform::PlatformInput};

pub mod inventory;

pub struct GuiManager {
    current_open: Option<Box<dyn Gui>>,
}

impl GuiManager {
    pub fn new() -> Self {
        Self { current_open: None }
    }

    pub fn is_open(&self) -> bool {
        self.current_open.is_some()
    }

    pub fn open(&mut self, gui: impl Gui + 'static) {
        self.current_open = Some(Box::new(gui));
    }

    pub fn close(&mut self) {
        if let Some(mut gui) = self.current_open.take() {
            gui.close();
        }
    }

    pub fn input(&mut self, input: &PlatformInput) {
        if let Some(gui) = self.current_open.as_mut() {
            if matches!(
                input,
                PlatformInput::Keyboard {
                    key: KeyCode::Escape,
                    ..
                }
            ) {
                gui.close();
                self.current_open = None;
                return;
            }
            gui.input(input);
        }
    }

    pub fn render_if_open(&mut self, frame: &mut Frame, assets: &ClientAssets) {
        if let Some(gui) = self.current_open.as_mut() {
            gui.render(frame, assets);
        }
    }
}

pub trait Gui {
    fn close(&mut self) {}

    fn input(&mut self, input: &PlatformInput);
    fn render(&self, frame: &mut Frame, assets: &ClientAssets);
}
