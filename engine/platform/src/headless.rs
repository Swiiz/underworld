use crate::init_logger;

pub struct HeadlessPlatform {}

#[derive(PartialEq)]
pub enum HeadlessPlatformEvent {
    Update,
}

impl HeadlessPlatform {
    pub fn new() -> Self {
        init_logger();
        Self {}
    }

    pub fn run<T>(
        #[allow(unused_mut)] mut self,
        app: &mut T,
        mut event_handler: impl FnMut(&mut T, HeadlessPlatformEvent),
    ) {
        loop {
            event_handler(app, HeadlessPlatformEvent::Update);
        }
    }
}
