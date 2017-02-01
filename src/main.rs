extern crate sponge;
extern crate image;

use image::RgbaImage;
use image::Rgba;

use sponge::*;

fn main() {
    println!("starting renderer");

    let render_width = 1024;
    let render_height = 1024;

    let scale = 1;

    let window_width : u32 = render_width * scale;
    let window_height : u32 = render_height * scale;

    let renderer = start_renderer(window_width, window_height);
    
    renderer.receive_channel.recv().unwrap(); // swallow the window ready message

    for i in 0..100000 {
        let mut img = RgbaImage::from_pixel(render_width, render_height, Rgba { data: [(i % 256) as u8,0,0,255] });
        for x in 0..render_width {
            for y in 0..render_height {
                // let pixel : Rgba<u8> = Rgba { data: [(x % 256) as u8,(y % 256) as u8,(i % 256) as u8, 255] };
                // img.put_pixel(x, y, pixel);
            }
        }

        renderer.send_channel.send(RendererUpdate::Render(i, img)).unwrap();
    }

    renderer.send_channel.send(RendererUpdate::Shutdown).unwrap();
    renderer.join_handle.join().unwrap()
}
