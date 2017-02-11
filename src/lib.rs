#![crate_name="sponge"]
#![allow(dead_code)]

#[macro_use]
extern crate glium;
extern crate glutin;
extern crate image;
extern crate cgmath;

use std::thread;
use std::thread::JoinHandle;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};

use image::GrayImage;

use glium::index::PrimitiveType;
use glium::texture::RawImage2d;

pub type SpongeResult<T> = Result<T, SpongeError>;
#[derive(Debug)]
pub enum SpongeError {
    ProgramCreation(glium::program::ProgramChooserCreationError),
    WindowCreation(glium::GliumCreationError<glutin::CreationError>),
}


#[derive(Copy, Clone)]
struct Vertex {
    pos: [f32; 2],
    tex: [f32; 2],
}

implement_vertex!(Vertex, pos, tex);

pub fn start_renderer(width: u32, height: u32) -> Renderer {
    let (send_tx, send_rx) = channel::<RendererUpdate>();
    let (reply_tx, reply_rx) = channel::<RendererReply>();
    // let mut img = RgbaImage::from_pixel(image_size, image_size, Rgba { data: [25,25,25,255] });

    let join_handle = thread::spawn(move || {
        println!("about to build window");
        let window = build_window("Sponge", width, height).unwrap();
        println!("I built the window");
        let program = simple_program(&window).unwrap();
        println!("I built the program");
        let vertex_buffer = glium::VertexBuffer::new(&window, 
            &[
                Vertex { pos: [-1.0, -1.0], tex: [0.0, 0.0] },
                Vertex { pos: [-1.0,  1.0], tex: [0.0, 1.0] },
                Vertex { pos: [ 1.0,  1.0], tex: [1.0, 1.0] },
                Vertex { pos: [ 1.0, -1.0], tex: [1.0, 0.0] }
            ]
        ).unwrap();
        let index_buffer = glium::IndexBuffer::new(&window, PrimitiveType::TriangleStrip, &[1 as u16, 2, 0, 3]).unwrap();
        println!("I built the geometry");
        // setup permanent stuff
        reply_tx.send(RendererReply::WindowReady).unwrap();

        'main: loop {
            let (n, image) : (u64, GrayImage) = match send_rx.recv() {
                Ok(RendererUpdate::Render(n, update)) => (n, update),
                Ok(RendererUpdate::ShutdownRenderer) => {
                    reply_tx.send(RendererReply::RendererShutdown).unwrap();
                    break 'main;
                },
                Err(err) => {
                    println!("renderer couldnt receive event from send_rx failed -> {:?}", err);
                    break 'main
                }
            };

            println!("pre poll");
            // loop over events
            for event in window.poll_events() {
                println!("got an event -> {:?}", event);
                match event {
                    glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) |
                    glutin::Event::Closed => {
                        println!("renderer has received a shutdown");
                        reply_tx.send(RendererReply::RendererShutdown).unwrap();
                        break 'main;
                    },
                    glutin::Event::Resized(_width, _height) => {
                        
                    },
                    _ => {},
                }
            }
            println!("post poll");

            let (width, height) = image.dimensions();
            let raw_image = image.into_raw();

            use std::borrow::Cow;

            let raw_image = RawImage2d {
                data: Cow::from(&raw_image[..]),
                width: width,
                height: height,
                format: glium::texture::ClientFormat::U8,
            };

            println!("post raw image");


            // glium::texture::UncompressedFloatFormat
            // glium::texture::UncompressedFloatFormat
            // let glium_image = glium::texture::RawImage2d::from_raw_rgba(image.into_raw(), dimensions);
            let opengl_texture = glium::texture::texture2d::Texture2d::with_format(&window, raw_image, glium::texture::UncompressedFloatFormat::U8, glium::texture::MipmapsOption::NoMipmap).unwrap();
            
            println!("post texture");

            let uniforms = uniform! {
                main_texture: opengl_texture.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest).minify_filter(glium::uniforms::MinifySamplerFilter::Nearest)
            };

            println!("post uniforms");

            use glium::Surface;

            let mut target = window.draw();

            println!("got target");

            target.clear_color(0.0, 0.0, 0.0, 0.0);
            println!("post clear");
            target.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &Default::default()).unwrap();
            println!("post draw");
            target.finish().unwrap();
            println!("post finish");

            reply_tx.send(RendererReply::Rendered(n)).unwrap();
        }
        println!("renderer thread done");
   });
   Renderer {
        send_channel: send_tx,
        receive_channel: reply_rx,
        join_handle: join_handle,
   }
}

pub enum RendererUpdate {
    Render(u64, GrayImage),
    ShutdownRenderer,
}

pub enum RendererReply {
    Rendered(u64),
    WindowReady,
    RendererShutdown,
}


pub struct Renderer {
    pub send_channel: Sender<RendererUpdate>,
    pub receive_channel: Receiver<RendererReply>,
    pub join_handle: JoinHandle<()>
}

pub fn build_window(title:&str, width: u32, height: u32) -> SpongeResult<glium::Display> { 
    use glium::DisplayBuild;
    use glium::glutin::GlRequest;
    use glium::glutin::GlProfile;
    use glium::glutin::Api;
    use glium::glutin::WindowBuilder;

    let builder = WindowBuilder::new()
        .with_title(title)
        .with_dimensions(width, height)    
        .with_gl_profile(GlProfile::Core)
        .with_gl(GlRequest::Specific(Api::OpenGl,(4,1)))
        .with_depth_buffer(24);

    builder.build_glium().map_err(SpongeError::WindowCreation)
}

pub fn simple_program<T>(display : &T) -> SpongeResult<glium::Program> where T : glium::backend::Facade {
    let pr = program!(display,
        330 => {
            vertex: "
                #version 330
                
                in vec2 pos;
                in vec2 tex;

                out vec2 v_tex_coords;

                void main() {
                    gl_Position = vec4(pos, 0.0, 1.0);
                    v_tex_coords = tex;
                }
            ",

            fragment: "
                #version 330

                uniform sampler2D main_texture;

                in vec2 v_tex_coords;
                out vec4 f_color;

                void main() {
                    float gray = texture(main_texture, v_tex_coords).r;
                    f_color = vec4(gray, gray, gray, 1.0);
                }
            "
        },
    );
    pr.map_err(SpongeError::ProgramCreation)    
}

