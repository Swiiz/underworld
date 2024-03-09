use std::sync::Arc;

use winit::{
    event::{Event as WinitEvent, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
};

pub use winit::window::*;

pub struct Platform {
    event_loop: Option<EventLoop<()>>, // None when running
    pub window: Arc<Window>,           // No need to support multiple windows
}

pub enum Event {
    Update,
    Render,
    Resize,
}

impl Platform {
    pub fn new_with_window() -> Self {
        let event_loop = EventLoop::new().expect("Could not create platform event_loop.");
        Self {
            window: Arc::new(WindowBuilder::new().build(&event_loop).unwrap()),
            event_loop: Some(event_loop),
        }
    }

    pub fn run<T>(mut self, app: &mut T, mut event_handler: impl FnMut(&mut T, Event)) {
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
        mut emit: impl FnMut(Event),
    ) {
        match event {
            WinitEvent::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                elwt.exit();
            }
            WinitEvent::AboutToWait => {
                emit(Event::Update);

                self.window.request_redraw();
            }
            WinitEvent::WindowEvent { event, .. } => match event {
                WindowEvent::RedrawRequested => {
                    emit(Event::Render);
                }
                WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                    emit(Event::Resize)
                }
                _ => (),
            },
            _ => (),
        }
    }
}
