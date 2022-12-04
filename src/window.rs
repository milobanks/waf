use winit::{
    event_loop::EventLoop,
    window::{WindowBuilder, Window},
};

pub fn create_window(event_loop: &EventLoop<()>) -> Window         {
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    window.set_title("Sit");
    // window.set_decorations(false);
    window.set_cursor_grab(true).unwrap();
    window.set_cursor_visible(false);
    window.set_maximized(false);

    window
}

