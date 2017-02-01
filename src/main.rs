extern crate sponge;
extern crate image;
extern crate rayon;
extern crate time;
extern crate cgmath;



use image::RgbaImage;
use image::Rgba;

use sponge::*;
use sponge::RendererReply::*;
use sponge::RendererUpdate::*;

use rayon::{Configuration};

use cgmath::*;

fn main() {
    let config = Configuration::new();
    rayon::initialize(config.set_num_threads(3)).unwrap();

    println!("starting renderer {:?}", 0b101u32);

    let render_width = 256;
    let render_height = 256;

    let scale = 4;

    let window_width : u32 = render_width * scale;
    let window_height : u32 = render_height * scale;

    let renderer = start_renderer(window_width, window_height);
    
    renderer.receive_channel.recv().unwrap(); // swallow the window ready message

    let mut renderer_initiated_shutdown = false;

    'outter: for i in 0..1000000 {
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
        
        println!("send {:?}", i);
        sponge_renderer_3d(i, &mut img);

        match renderer.send_channel.send(Render(i, img)) {
            Ok(_) => (),
            Err(_) => {
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

pub type Vec3f = cgmath::Vector3<f32>;


const MAX_DEPTH: u8 = 5;
const CUBE_MASK: u32 = 0b111101111_101000101_111101111u32;

fn point_to_offset(x: f32, y: f32, z: f32) -> u8 {
    (x * 3.0) as u8 + ((y * 3.0) as u8) * 3 + ((z * 3.0) as u8) * 9 // it's arbitrary you fool :D
}

const ONE_THIRD : f32 = 1.0 / 3.0;

fn test_cube(x: f32, y: f32, z: f32) -> Option<f32> {
    let mut ux = x;
    let mut uy = y;
    let mut uz = z;

    let mut test_cube_size = ONE_THIRD;

    for d in 0..MAX_DEPTH {
        let offset = point_to_offset(ux, uy, uz);
        if (1u32 << offset) & CUBE_MASK == 0 {
            return Some(test_cube_size);
        } else {
            // I THINK THIS BIT IS FUCKED
            ux = (ux * 3.0) % 1.0;
            uy = (uy * 3.0) % 1.0;
            uz = (uz * 3.0) % 1.0;
        }
        test_cube_size * ONE_THIRD;
    }

    None
}

fn test_ray(at:Vec3f, direction:Vec3f) -> bool {
    true
}

fn sponge_renderer_3d(n: u64, img: &mut RgbaImage) {
    let (width, height) = img.dimensions();

    let nu = (n % 256) as u8;

    let pixel = 1.0 / (width) as f32;
    let half_pixel: f32 = 1.0 / (width as f32);

    let origin = Vec3f::new(0.5, 0.5, 0.5 - (n as f32) * 0.005);

    let theta : Rad<f32> = Rad((n as f32) * 0.01);

    let rotation : cgmath::Matrix3<f32> = Matrix3::from_angle_y(theta);

    let o_forward = Vec3f::new(0.0, 0.0, -1.0);
    let o_right = Vec3f::new(1.0, 0.0, 0.0);
    let o_down = Vec3f::new(0.0, -1.0, 0.0);



    let forward = rotation * o_forward;
    let right = rotation * o_right;
    let down = rotation * o_down;

    println!("forward {:?} right {:?} down {:?}", forward, right, down);


    let target = origin + forward;

    let distance = 0.65;
    let moves : u8 = 192;
    let per_move_init : f32 = distance / (moves as f32);
    
    let mut color_lookup : Vec<u8> = Vec::new();

    for i in 0..256 {
        color_lookup.push((i as f32).powf(0.8) as u8);
    }



    let colour_multiplier : u8 = 255 / moves;

    for x in 0..width {   
        for y in 0..height {
            let fx = (x as f32) / (width as f32) * 2.0 - 1.0; // -0.5 .. 0.5
            let fy = (y as f32) / (height as f32) * 2.0 - 1.0; // -0.5 .. 0.5

            let pixel_target = fx * right + fy * down + target;
            let direction = (pixel_target - origin).normalize();

            let mut per_move = per_move_init;
            let mut point = origin + direction * per_move;
            

            let mut hit = false;
            for d in 0..moves {
                if let Some(rejected_cube_size) =  test_cube(point.x % 1.0, point.y % 1.0, point.z % 1.0) {
                    point += direction * per_move;
                } else {
                    hit = true;
                    let darkness = color_lookup[d as usize];
                    let pixel : Rgba<u8> = Rgba { data: [darkness, darkness, darkness, 255] };
                    img.put_pixel(x, y, pixel);
                    break;
                }
            }

            if !hit {
                let darkness = color_lookup[moves as usize];
                let pixel : Rgba<u8> = Rgba { data: [darkness, darkness, darkness, 255] };
                img.put_pixel(x, y, pixel);
            }





        }
    }
}

fn text_pattern_render(n: u64, img: &mut RgbaImage) {
    let (width, height) = img.dimensions();
    for x in (0..width) {
        for y in 0..height {
            let pixel : Rgba<u8> = Rgba { data: [(x % 256) as u8,(y % 256) as u8,(n % 256) as u8, 255] };
            img.put_pixel(x, y, pixel);
        }
    }
}
