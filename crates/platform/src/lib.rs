use std::sync::Arc;

use winit::{
    event::{Event as WinitEvent, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
    window::Window,
};

pub use winit::window::WindowBuilder;

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
    pub fn new(window_builder: WindowBuilder) -> Self {
        let event_loop = EventLoop::new().expect("Could not create platform event_loop.");
        Self {
            window: Arc::new(
                window_builder
                    .build(&event_loop)
                    .expect("Could not create platform window."),
            ),
            event_loop: Some(event_loop),
        }
    }

    pub fn run(mut self, mut event_handler: impl FnMut(Event)) {
        self.event_loop
            .take()
            .unwrap()
            .run(|event, elwt| self.handle_event(event, elwt, |event| event_handler(event)))
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
