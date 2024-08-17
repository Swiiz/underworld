use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

pub struct Platform<T: AppLayer> {
    app: Option<T>,
}

impl<T: AppLayer> ApplicationHandler for Platform<T> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.app = Some(T::new(event_loop));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, wid: WindowId, event: WindowEvent) {
        let Some(app) = &mut self.app else {
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
            _ => (),
        }
    }
}

pub trait AppLayer {
    fn new(event_loop: &ActiveEventLoop) -> Self;
    fn render(&mut self, _: WindowId) {}
    fn update(&mut self, _: WindowId) {}
    fn window_resized(&mut self) {}
    fn windows(&self) -> Vec<&Window>;
}

pub fn run_app<T: AppLayer>() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = Platform::<T> { app: None };
    if let Err(e) = event_loop.run_app(&mut app) {
        panic!("{e:?}");
    }
}
