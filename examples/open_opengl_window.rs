use std::time::Duration;

use rtrb::{Consumer, RingBuffer};

use glow::*;

#[cfg(target_os = "macos")]
use baseview::copy_to_clipboard;
use baseview::{Event, EventStatus, Window, WindowHandler, WindowScalePolicy};

#[derive(Debug, Clone)]
enum Message {
    Hello,
}

struct OpenWindowExample {
    rx: Consumer<Message>,
    gl: glow::Context,
    program: NativeProgram,
    vao: NativeVertexArray,
}

impl OpenWindowExample {
    fn new(window: &Window, rx: Consumer<Message>) -> Result<Self, String> {
        let context = window
            .gl_context()
            .expect("failed to get baseview gl context");
        unsafe {
            context.make_current();
        }

        unsafe {
            let gl =
                glow::Context::from_loader_function(|s| context.get_proc_address(s) as *const _);

            let program = gl.create_program().expect("Cannot create program");

            let (vertex_shader_src, fragment_shader_src) = (
                r#"const vec2 verts[3] = vec2[3](
                vec2(0.5f, 1.0f),
                vec2(0.0f, 0.0f),
                vec2(1.0f, 0.0f)
            );
            out vec2 vert;
            void main() {
                vert = verts[gl_VertexID];
                gl_Position = vec4(vert - 0.5, 0.0, 1.0);
            }"#,
                r#"precision mediump float;
            in vec2 vert;
            out vec4 color;
            void main() {
                color = vec4(vert, 0.5, 1.0);
            }"#,
            );

            let shader_version = "#version 300 es";

            let vertex_shader = gl.create_shader(glow::VERTEX_SHADER).unwrap();
            gl.shader_source(
                vertex_shader,
                &format!("{}\n{}", shader_version, vertex_shader_src),
            );
            gl.compile_shader(vertex_shader);
            if !gl.get_shader_compile_status(vertex_shader) {
                panic!("{}", gl.get_shader_info_log(vertex_shader));
            }

            gl.attach_shader(program, vertex_shader);

            let fragment_shader = gl.create_shader(glow::FRAGMENT_SHADER).unwrap();
            gl.shader_source(
                fragment_shader,
                &format!("{}\n{}", shader_version, fragment_shader_src),
            );
            gl.compile_shader(fragment_shader);
            gl.compile_shader(fragment_shader);
            if !gl.get_shader_compile_status(fragment_shader) {
                panic!("{}", gl.get_shader_info_log(fragment_shader));
            }

            gl.attach_shader(program, fragment_shader);

            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                panic!("{}", gl.get_program_info_log(program));
            }

            gl.detach_shader(program, vertex_shader);
            gl.delete_shader(vertex_shader);

            gl.detach_shader(program, fragment_shader);
            gl.delete_shader(fragment_shader);

            let vao = gl
                .create_vertex_array()
                .expect("Cannot create vertex array");
            gl.bind_vertex_array(Some(vao));

            Ok(OpenWindowExample {
                rx,
                gl,
                program,
                vao,
            })
        }
    }

    fn draw(&self) {
        unsafe {
            self.gl.clear_color(0.2, 0.3, 0.3, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT);

            self.gl.use_program(Some(self.program));
            self.gl.bind_vertex_array(Some(self.vao));

            self.gl.draw_arrays(glow::TRIANGLES, 0, 3);
        }
    }
}

impl WindowHandler for OpenWindowExample {
    fn on_frame(&mut self, window: &mut Window) {
        let context = window
            .gl_context()
            .expect("Failed to get baseview GL context");

        self.draw();

        while let Ok(message) = self.rx.pop() {
            println!("Message: {:?}", message);
        }
        unsafe {
            context.make_current();
            context.swap_buffers();
        }
    }

    fn on_event(&mut self, _window: &mut Window, event: Event) -> EventStatus {
        match event {
            Event::Mouse(e) => {
                println!("Mouse event: {:?}", e);

                #[cfg(target_os = "macos")]
                match e {
                    MouseEvent::ButtonPressed { .. } => copy_to_clipboard(&"This is a test!"),
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
        gl_config: Some(Default::default()),
        #[cfg(feature = "vulkan")]
        vk_config: None,
    };

    let (mut tx, rx) = RingBuffer::new(128);

    ::std::thread::spawn(move || loop {
        ::std::thread::sleep(Duration::from_secs(5));

        if let Err(_) = tx.push(Message::Hello) {
            println!("Failed sending message");
        }
    });

    Window::open_blocking(window_open_options, move |window| {
        OpenWindowExample::new(&window, rx).expect("Failed to initialize OpenWindowExample")
    });
}