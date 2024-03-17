
use ash::vk;
pub use ash::{Device, Instance};

use std::time::Duration;

use rtrb::{Consumer, RingBuffer};

#[cfg(target_os = "macos")]
use baseview::copy_to_clipboard;
use baseview::{Event, EventStatus, MouseEvent, Window, WindowHandler, WindowScalePolicy};

#[derive(Debug, Clone)]
enum Message {
    Hello,
}

struct OpenVulkanWindowExample {
    rx: Consumer<Message>,

}

impl OpenVulkanWindowExample {
    fn new(window: &Window, rx: Consumer<Message>) -> Self {
        let context = window.vk_context().expect("Failed to obtain Vulkan context");
        Self { rx }
    }


}

impl WindowHandler for OpenVulkanWindowExample {
    fn on_frame(&mut self, _window: &mut Window) {
        while let Ok(message) = self.rx.pop() {
            println!("Message: {:?}", message);
        }
    }

    fn on_event(&mut self, _window: &mut Window, event: Event) -> EventStatus {
        match event {
            Event::Mouse(e) => {
                println!("Mouse event: {:?}", e);

                #[cfg(target_os = "macos")]
                match e {
                    MouseEvent::ButtonPressed { .. } => {
                        copy_to_clipboard(&"This is a test!")
                    }
                    _ => (),
                }
            }
            Event::Keyboard(e) => println!("Keyboard event: {:?}", e),
            Event::Window(e) => println!("Window event: {:?}", e),
        }

        EventStatus::Captured
    }
}

fn main() {
    let window_open_options = baseview::WindowOpenOptions {
        title: "baseview".into(),
        size: baseview::Size::new(512.0, 512.0),
        scale: WindowScalePolicy::SystemScaleFactor,
        #[cfg(feature = "opengl")]
        gl_config: None,
        #[cfg(feature = "vulkan")]
        vk_config: Some(Default::default()),
    };

    let (mut tx, rx) = RingBuffer::new(128);

    ::std::thread::spawn(move || loop {
        ::std::thread::sleep(Duration::from_secs(5));

        if let Err(_) = tx.push(Message::Hello) {
            println!("Failed sending message");
        }
    });

    Window::open_blocking(window_open_options, move |window| OpenVulkanWindowExample::new(&window, rx));
}
