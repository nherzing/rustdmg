use std::iter::{ Iterator };
use structopt::StructOpt;
use sdl2::event::Event;
use sdl2::keyboard::{KeyboardState, Scancode, Keycode};
use crate::joypad_controller::JoypadInput;
use crate::cartridge::Cartridge;

#[cfg(feature = "debug")]
macro_rules! debug {
    ($( $args:expr ),*) => { println!( $( $args ),* ); }
}

#[cfg(not(feature = "debug"))]
macro_rules! debug {
    ($( $args:expr ),*) => {}
}


mod bitops;
mod cartridge;
mod clocks;
mod memory;
mod cpu;
mod ram_device;
mod rom_device;
mod gameboy;
mod interrupt_controller;
mod timer_controller;
mod joypad_controller;
mod lcd;
mod renderer;

#[derive(StructOpt)]
struct Cli {
    #[structopt(short, long)]
    debug: bool,
    #[structopt(short, long)]
    skip_boot_rom: bool,

    #[structopt(parse(from_os_str))]
    cartridge_path: std::path::PathBuf,
}

const SCANCODES: [Scancode; 8] = [
    Scancode::W, Scancode::S, Scancode::A, Scancode::D,
    Scancode::X, Scancode::Z, Scancode::L, Scancode::K
];

fn scancode_to_joypad_input(scancode: &Scancode) -> JoypadInput {
    match *scancode {
        Scancode::W => JoypadInput::Up,
        Scancode::S => JoypadInput::Down,
        Scancode::A => JoypadInput::Left,
        Scancode::D => JoypadInput::Right,
        Scancode::X => JoypadInput::Start,
        Scancode::Z => JoypadInput::Select,
        Scancode::L => JoypadInput::A,
        Scancode::K => JoypadInput::B,
        _ => panic!("No mapping for key {:?}", scancode)
    }
}

fn collect_pressed(keyboard_state: &KeyboardState) -> Vec<JoypadInput> {
    SCANCODES.iter().filter(|sc| keyboard_state.is_scancode_pressed(**sc)).
        map(|sc| scancode_to_joypad_input(sc)).
        collect()
}

fn main() {
    let args = Cli::from_args();
    let cartridge = Cartridge::new(args.cartridge_path);

    if args.debug {
        debug!("Debug enabled!");
    }

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Gameboy", 160*2 + 50 + 128*2, 144*2)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window
        .into_canvas()
        .build()
        .unwrap();
    let texture_creator = canvas.texture_creator();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let renderer = renderer::Renderer::new(&mut canvas, &texture_creator);
    let mut gameboy = gameboy::Gameboy::new(renderer, args.debug);

    gameboy.boot(cartridge, args.skip_boot_rom);



    'running: loop {
        let pressed = collect_pressed(&event_pump.keyboard_state());

        gameboy.tick(&pressed);

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
    }
}
