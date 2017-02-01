extern crate sponge;
extern crate image;

use image::RgbaImage;
use image::Rgba;

use sponge::*;
use sponge::RendererReply::*;
use sponge::RendererUpdate::*;

fn main() {
    println!("starting renderer");

    let render_width = 512;
    let render_height = 512;

    let scale = 2;

    let window_width : u32 = render_width * scale;
    let window_height : u32 = render_height * scale;

    let renderer = start_renderer(window_width, window_height);
    
    renderer.receive_channel.recv().unwrap(); // swallow the window ready message

    let mut renderer_initiated_shutdown = false;

    'outter: for i in 0..100 {
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
                Ok(RendererShutdown) => {
                    renderer_initiated_shutdown = true;
                    break 'outter
                }
            }    
        }
        
        let mut img = RgbaImage::from_pixel(render_width, render_height, Rgba { data: [(i % 256) as u8,0,0,255] });
        
        text_pattern_render(i, &mut img);

        match renderer.send_channel.send(Render(i, img)) {
            Ok(_) => (),
            Err(err) => {
                renderer_initiated_shutdown = true;
                break 'outter;
            },
        }
    }

    if !renderer_initiated_shutdown {
        renderer.send_channel.send(ShutdownRenderer).unwrap();    
    }
    renderer.join_handle.join().unwrap()
}

fn sponge_renderer(n: u64, img: &mut RgbaImage) {
    let (width, height) = img.dimensions();


}

fn text_pattern_render(n: u64, img: &mut RgbaImage) {
    let (width, height) = img.dimensions();
    for x in 0..width {
        for y in 0..height {
            let pixel : Rgba<u8> = Rgba { data: [(x % 256) as u8,(y % 256) as u8,(n % 256) as u8, 255] };
            img.put_pixel(x, y, pixel);
        }
    }
}
