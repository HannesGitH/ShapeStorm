use egui::Event;

struct InputHandler{}

impl InputHandler {
    fn new() -> Self {
        Self{}
    }
}

impl InputHandler {
    fn on_event(&mut self, event: &Event) {
        match event {
            Event::Key{key, pressed, modifiers, .. } => {
                println!("{:?} = {:?}", key, pressed);
            },
            _ => {}
        }
    }
}