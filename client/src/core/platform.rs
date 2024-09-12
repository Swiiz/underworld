use std::sync::Arc;

use winit::window::{Window, WindowId};
use winit::{application::ApplicationHandler, window::WindowAttributes};
use winit::{
    event::{ElementState, WindowEvent},
    keyboard::KeyCode,
};
use winit::{
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::PhysicalKey,
};

pub enum Platform<T: AppLayer> {
    Init(T::Config),
    Paused,
    Running(T),
}

impl<T: AppLayer> ApplicationHandler for Platform<T> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        match std::mem::replace(self, Platform::Paused) {
            Platform::Init(config) => {
                *self = Platform::Running(T::new(PlatformHandle(event_loop), config));
            }
            _ => {}
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, wid: WindowId, event: WindowEvent) {
        let Platform::Running(app) = self else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                app.render(wid);

                for w in app.windows() {
                    w.request_redraw();
                }
            }
            WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                app.window_resized();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(key) = event.physical_key {
                    app.input(
                        wid,
                        PlatformInput::Keyboard {
                            key,
                            state: event.state,
                        },
                    );
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                app.input(
                    wid,
                    PlatformInput::CursorMoved {
                        x: position.x as f32,
                        y: position.y as f32,
                    },
                );
            }
            WindowEvent::MouseWheel { delta, .. } => {
                app.input(
                    wid,
                    match delta {
                        winit::event::MouseScrollDelta::LineDelta(x, y) => {
                            PlatformInput::MouseScrolled { x, y }
                        }
                        winit::event::MouseScrollDelta::PixelDelta(_) => {
                            return;
                        }
                    },
                );
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        let Platform::Running(app) = self else {
            return;
        };

        app.update();
    }

    fn exiting(&mut self, _: &ActiveEventLoop) {
        let Platform::Running(app) = self else {
            return;
        };

        app.exit();
    }
}

pub enum PlatformInput {
    Keyboard { key: KeyCode, state: ElementState },
    CursorMoved { x: f32, y: f32 },
    MouseScrolled { x: f32, y: f32 },
}

pub struct PlatformHandle<'a>(&'a ActiveEventLoop);

impl PlatformHandle<'_> {
    pub fn create_window(&self, attrs: WindowAttributes) -> Arc<Window> {
        Arc::new(self.0.create_window(attrs).unwrap())
    }
}

pub trait AppLayer {
    type Config;
    fn new(handle: PlatformHandle, config: Self::Config) -> Self;
    fn render(&mut self, _: WindowId) {}
    fn update(&mut self) {}
    fn input(&mut self, _: WindowId, _: PlatformInput) {}
    fn exit(&mut self) {}
    fn window_resized(&mut self) {}
    fn windows(&self) -> Vec<&Window>;
}

pub fn run_app<T: AppLayer>(config: T::Config) {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = Platform::<T>::Init(config);
    if let Err(e) = event_loop.run_app(&mut app) {
        panic!("{e:?}");
    }
}
