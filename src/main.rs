use chip_eight_emu::*;
use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::env;
use std::fs::File;
use std::io::Read;

const SCALE: u32 = 20;
const WINDOW_WIDTH: u32 = SCREEN_WIDTH as u32 * SCALE;
const WINDOW_HEIGHT: u32 = SCREEN_HEIGHT as u32 * SCALE;
fn main() {
    // parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <file>", args[0]);
        return;
    }
    let mut file = File::open(&args[1]).expect("Unable to open file, make sure it exists");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let rom = buffer;

    // Create an SDL2 window
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Chip-8 Emulator", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.clear();
    canvas.present();

    // Run the emulator
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut chip_eight = Emulator::new();
    chip_eight.load(&rom);

    // Game loop
    'gameloop: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    break 'gameloop;
                }
                _ => {}
            }
        }
        chip_eight.tick(); //execute next instruction (move program counter)
        draw_screen(&chip_eight, &mut canvas);
    }
}

struct Keys {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    a: bool,
    b: bool,
    c: bool,
}
impl Keys {
    fn new() -> Self {
        Self {
            up: false,
            down: false,
            left: false,
            right: false,
            a: false,
            b: false,
            c: false,
        }
    }
    fn read(rl: &prelude::RaylibHandle) -> Self {
        let up = rl.is_key_down(raylib::consts::KeyboardKey::KEY_UP);
        let down = rl.is_key_down(raylib::consts::KeyboardKey::KEY_DOWN);
        let left = rl.is_key_down(raylib::consts::KeyboardKey::KEY_LEFT);
        let right = rl.is_key_down(raylib::consts::KeyboardKey::KEY_RIGHT);
        let a = rl.is_key_down(raylib::consts::KeyboardKey::KEY_A);
        let b = rl.is_key_down(raylib::consts::KeyboardKey::KEY_B);
        let c = rl.is_key_down(raylib::consts::KeyboardKey::KEY_C);
        Self {
            up,
            down,
            left,
            right,
            a,
            b,
            c,
        }
    }
}

///  1D screen buffer array and iterate across it. If we find a white pixel (a true value), then
///  we calculate the 2D (x, y) of the screen and draw a rectangle
fn draw_screen(emulator: &Emulator, canvas: &mut Canvas<Window>) {
    // clear screen
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    let screen_buffer = emulator.get_display();
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for (i, pixel) in screen_buffer.iter().enumerate() {
        if *pixel {
            // convert index to x,y
            let x = (i % SCREEN_WIDTH) as u32;
            let y = (i / SCREEN_WIDTH) as u32;
            // draw pixel (scaled)
            let rect = Rect::new((x * SCALE) as i32, (y * SCALE) as i32, SCALE, SCALE);
            canvas.fill_rect(rect).unwrap();
        }
    }
    canvas.present();
}
