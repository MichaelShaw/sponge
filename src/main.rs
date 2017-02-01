extern crate sponge;
extern crate image;

use image::RgbaImage;
use image::Rgba;

use sponge::*;
use sponge::RendererReply::*;
use sponge::RendererUpdate::*;

use std::num;

fn main() {
    println!("starting renderer");

    let render_width = 512;
    let render_height = 512;

    let scale = 1;

    let window_width : u32 = render_width * scale;
    let window_height : u32 = render_height * scale;

    let renderer = start_renderer(window_width, window_height);
    
    renderer.receive_channel.recv().unwrap(); // swallow the window ready message

    'outter: for i in 0..100000 {
        'check_render: loop {
            match renderer.receive_channel.try_recv() {
                Err(_) => {
                    break 'check_render
                },
                Ok(Rendered(frame_n)) => {
                    let frame_delta = i64::abs((frame_n as i64) - (i as i64));
                    if frame_delta > 10 {
                        println!("main :: we're up to frame {:?} but renderer has only rendered {:?} frame delta is {:?}", i, frame_n, frame_delta);
                    }
                },
                Ok(WindowReady) => (),
                Ok(RendererShutdown) => break 'outter,
            }    
        }
        
        let mut img = RgbaImage::from_pixel(render_width, render_height, Rgba { data: [(i % 256) as u8,0,0,255] });
        // for x in 0..render_width {
        //     for y in 0..render_height {
        //         let pixel : Rgba<u8> = Rgba { data: [(x % 256) as u8,(y % 256) as u8,(i % 256) as u8, 255] };
        //         img.put_pixel(x, y, pixel);
        //     }
        // }

        renderer.send_channel.send(Render(i, img)).unwrap();
    }

    renderer.send_channel.send(ShutdownRenderer).unwrap();
    renderer.join_handle.join().unwrap()
}

