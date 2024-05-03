use chip_eight_emu::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::env;
use std::fs::File;
use std::io::Read;

const SCALE: u32 = 15;
const WINDOW_WIDTH: u32 = SCREEN_WIDTH as u32 * SCALE;
const WINDOW_HEIGHT: u32 = SCREEN_HEIGHT as u32 * SCALE;
const TICKS_PER_FRAME: usize = 10;
fn main() {
    // parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <file>", args[0]);
        return;
    }

    // Load the ROM
    let mut file = File::open(&args[1]).expect("Unable to open file, make sure it exists");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let rom = buffer;
    let mut chip_eight = Emulator::new();
    chip_eight.load(&rom);

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

    let audio_context = sdl_context.audio().unwrap();
    let buzzer = chip_eight_emu::sound::Buzzer::new(&audio_context);

    // Run the emulator
    let mut event_pump = sdl_context.event_pump().unwrap();

    // Game loop
    'gameloop: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'gameloop;
                }
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    if let Some(button) = keypress_to_button_code(key) {
                        chip_eight.keypress(button, true);
                    }
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    if let Some(button) = keypress_to_button_code(key) {
                        chip_eight.keypress(button, false);
                    }
                }
                _ => {}
            }
        }
        for _ in 0..TICKS_PER_FRAME {
            // 10 instructions per frame
            chip_eight.tick(); //execute next instruction (move program counter)
        }
        chip_eight.tick_timers(&buzzer);

        draw_screen(&chip_eight, &mut canvas);
    }
}

///  1D screen buffer array and iterate across it. If we find a white pixel (a true value), then
///  we calculate the 2D (x, y) of the screen and draw a rectangle
///  TODO: vsync (wait for vblank)
fn draw_screen(emulator: &Emulator, canvas: &mut Canvas<Window>) {
    // clear screen
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    let screen_buffer = emulator.get_display();
    canvas.set_draw_color(Color::RGB(0x0, 0xff, 0x0));
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

fn keypress_to_button_code(key: Keycode) -> Option<usize> {
    // original chip 8 expects a grid of 4x4 buttons
    // 1 2 3 C
    // 4 5 6 D
    // 7 8 9 E
    // A 0 B F
    // this function maps the sdl2 keycode to the chip8 button code
    match key {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),
        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),
        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None,
    }
}
