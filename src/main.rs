use structopt::StructOpt;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use crate::cartridge::Cartridge;

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

fn main() {
    let args = Cli::from_args();
    let cartridge = Cartridge::new(args.cartridge_path);

    if args.debug {
        println!("Debug enabled!");
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
        gameboy.tick();

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
