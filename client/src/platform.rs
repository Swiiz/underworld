use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};
use winit::{application::ApplicationHandler, event::DeviceEvent};

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

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let Some(app) = &mut self.app else {
            return;
        };

        app.update();
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _: winit::event::DeviceId, event: DeviceEvent) {
        let Some(app) = &mut self.app else {
            return;
        };

        app.input(event);
    }
}

pub trait AppLayer {
    fn new(event_loop: &ActiveEventLoop) -> Self;
    fn render(&mut self, _: WindowId) {}
    fn update(&mut self) {}
    fn input(&mut self, _: DeviceEvent) {}
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
