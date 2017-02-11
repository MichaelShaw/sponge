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
    VertexCreation(glium::vertex::BufferCreationError),
    IndexCreation(glium::index::BufferCreationError),
}


#[derive(Copy, Clone)]
pub struct Vertex {
    pos: [f32; 2],
    tex: [f32; 2],
}

implement_vertex!(Vertex, pos, tex);

pub struct Renderer {
    pub display: glium::Display,
    pub program: glium::Program,
    pub vertex_buffer: glium::VertexBuffer<Vertex>,
    pub index_buffer: glium::IndexBuffer<u16>, 
}

impl Renderer {
    pub fn new(width: u32, height: u32) -> SpongeResult<Renderer> {
        let display : glium::Display = build_window("Sponge", width, height)?;
        let program = simple_program(&display)?;
        let vertex_buffer = glium::VertexBuffer::new(&display,
            &[
                Vertex { pos: [-1.0, -1.0], tex: [0.0, 0.0] },
                Vertex { pos: [-1.0,  1.0], tex: [0.0, 1.0] },
                Vertex { pos: [ 1.0,  1.0], tex: [1.0, 1.0] },
                Vertex { pos: [ 1.0, -1.0], tex: [1.0, 0.0] }
            ]
        ).map_err(SpongeError::VertexCreation)?;
        // 
        let index_buffer = glium::IndexBuffer::new(
            &display, 
            PrimitiveType::TriangleStrip, &[1 as u16, 2, 0, 3]
        ).map_err(SpongeError::IndexCreation)?;

        Ok(Renderer {
            display: display,
            program: program,
            vertex_buffer: vertex_buffer,
            index_buffer: index_buffer,
        })      
    }

    pub fn render(&mut self, image: GrayImage) -> bool {
        let mut should_close = false;
        // loop over events
        for event in self.display.poll_events() {
            println!("got an event -> {:?}", event);
            match event {
                glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) |
                glutin::Event::Closed => {
                    println!("renderer has received a shutdown");
                    should_close = true
                },
                glutin::Event::Resized(_width, _height) => {
                    
                },
                _ => {},
            }
        }


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
        let opengl_texture = glium::texture::texture2d::Texture2d::with_format(&self.display, raw_image, glium::texture::UncompressedFloatFormat::U8, glium::texture::MipmapsOption::NoMipmap).unwrap();
        
        println!("post texture");

        let uniforms = uniform! {
            main_texture: opengl_texture.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest).minify_filter(glium::uniforms::MinifySamplerFilter::Nearest)
        };

        println!("post uniforms");

        use glium::Surface;

        let mut target = self.display.draw();

        println!("got target");

        target.clear_color(0.0, 0.0, 0.0, 0.0);
        println!("post clear");
        target.draw(&self.vertex_buffer, &self.index_buffer, &self.program, &uniforms, &Default::default()).unwrap();
        println!("post draw");
        target.finish().unwrap();
        println!("post finish");

        should_close
    }
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

