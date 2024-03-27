use std::sync::Arc;

use log::info;
use winit::{
    event::{Event as WinitEvent, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
};

pub use winit::window::*;

use crate::init_logger;

pub struct WindowPlatform {
    event_loop: Option<EventLoop<()>>, // None when running
    pub window: Arc<Window>,           // No need to support multiple windows
}

#[derive(PartialEq)]
pub enum WindowPlatformEvent {
    Update,
    Render,
    Resize,
}

impl WindowPlatform {
    pub fn new() -> Self {
        init_logger();
        let event_loop = EventLoop::new().expect("Could not create platform event_loop.");
        let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());
        info!("Platfrom window context created.");

        Self {
            window,
            event_loop: Some(event_loop),
        }
    }

    pub fn run<T>(
        mut self,
        app: &mut T,
        mut event_handler: impl FnMut(&mut T, WindowPlatformEvent),
    ) {
        self.event_loop
            .take()
            .unwrap()
            .run(|event, elwt| self.handle_event(event, elwt, |event| event_handler(app, event)))
            .expect("A platform error occured while running window.")
    }

    fn handle_event(
        &self,
        event: WinitEvent<()>,
        elwt: &EventLoopWindowTarget<()>,
        mut emit: impl FnMut(WindowPlatformEvent),
    ) {
        match event {
            WinitEvent::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                elwt.exit();
            }
            WinitEvent::AboutToWait => {
                emit(WindowPlatformEvent::Update);

                self.window.request_redraw();
            }
            WinitEvent::WindowEvent { event, .. } => match event {
                WindowEvent::RedrawRequested => {
                    emit(WindowPlatformEvent::Render);
                }
                WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                    emit(WindowPlatformEvent::Resize)
                }
                _ => (),
            },
            _ => (),
        }
    }
}
