extern crate glium;
extern crate glutin;
extern crate image;
extern crate cgmath;

use image::RgbaImage;
use image::Rgba;

use std::thread;
use std::thread::JoinHandle;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};

use std::{time};

fn main() {
    println!("starting renderer");

    let width : u32 = 512;
    let height : u32 = 512;

    let renderer = start_renderer(width, height);

    let ten_millis = time::Duration::from_millis(1000);
    thread::sleep(ten_millis);

    for i in 0..255 {
        println!("sender :: rendering frame {:?}", i);
        let img = RgbaImage::from_pixel(width, width, Rgba { data: [i,i,i,255] });
        renderer.send_channel.send(RendererUpdate::Render(img)).unwrap();
    }

    renderer.send_channel.send(RendererUpdate::Shutdown).unwrap();
    renderer.join_handle.join().unwrap()
}

pub fn start_renderer(width: u32, height: u32) -> Renderer {
    let (send_tx, send_rx) = channel::<RendererUpdate>();
    let (reply_tx, reply_rx) = channel::<RendererReply>();
    // let mut img = RgbaImage::from_pixel(image_size, image_size, Rgba { data: [25,25,25,255] });

    let join_handle = thread::spawn(move || {
        let window = build_window("Sponge", width, height);

        // setup permanent stuff
        println!("renderer :: rendering in seperate thread");

        'main: loop {
            println!("renderer :: ahout to await event");
            let image : RgbaImage = match send_rx.recv() {
                Ok(RendererUpdate::Render(update)) => update,
                Ok(RendererUpdate::Shutdown) => {
                    reply_tx.send(RendererReply::Shutdown).unwrap();
                    break 'main;
                },
                Err(err) => {
                    println!("renderer couldnt receive event from send_rx failed -> {:?}", err);
                    break 'main
                }
            };

            // loop over events
            println!("renderer :: polling window events");
            for event in window.poll_events() {
                match event {
                    glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) |
                    glutin::Event::Closed => {
                        println!("renderer has received a shutdown");
                        reply_tx.send(RendererReply::Shutdown).unwrap();
                        break 'main;
                    },
                    glutin::Event::Resized(_width, _height) => {
                        
                    },
                    _ => {},
                }
            }

            println!("render :: rendering");

            reply_tx.send(RendererReply::Ok).unwrap();
        }
   });
   Renderer {
        send_channel: send_tx,
        receive_channel: reply_rx,
        join_handle: join_handle,
   }
}

pub enum RendererUpdate {
    Render(RgbaImage),
    Shutdown,
}

pub enum RendererReply {
    Ok,
    Shutdown,
}


pub struct Renderer {
    pub send_channel: Sender<RendererUpdate>,
    pub receive_channel: Receiver<RendererReply>,
    pub join_handle: JoinHandle<()>
}

pub fn build_window(title:&str, width: u32, height: u32) -> glium::Display { 
    use glium::DisplayBuild;
    use glium::glutin::GlRequest;
    use glium::glutin::GlProfile;
    use glium::glutin::Api;
    use glium::glutin::WindowBuilder;

    let builder = WindowBuilder::new()
        .with_title(title)
        .with_dimensions(width, height)    
        .with_gl_profile(GlProfile::Core)
        .with_gl(GlRequest::Specific(Api::OpenGl,(3,3)))
        .with_depth_buffer(24);

    builder.build_glium()
        .unwrap()
}