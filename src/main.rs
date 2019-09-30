use structopt::StructOpt;
use sdl2::audio::AudioSpecDesired;
use crate::clocks::AUDIO_SAMPLE_RATE;

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
mod gameboy;
mod interrupt_controller;
mod timer_controller;
mod joypad_controller;
mod lcd;
mod sound;
mod serial;
mod renderer;

#[derive(StructOpt)]
struct Cli {
    #[structopt(short, long)]
    debug: bool,
    #[structopt(long)]
    dmg: bool,
    #[structopt(short, long)]
    skip_boot_rom: bool,
    #[structopt(parse(from_os_str))]
    cartridge_path: std::path::PathBuf,
}

fn main() {
    let args = Cli::from_args();
    let cartridge = cartridge::Cartridge::new(args.cartridge_path);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

    let window = video_subsystem
        .window("Gameboy", 160*2, 144*2)
        .position_centered()
        .resizable()
        .build()
        .unwrap();
    let canvas = window
        .into_canvas()
        .build()
        .unwrap();

    let desired_spec = AudioSpecDesired {
        freq: Some(AUDIO_SAMPLE_RATE as i32),
        channels: Some(2),
        samples: Some(1024)
    };
    let audio_queue = audio_subsystem.open_queue(None, &desired_spec).unwrap();
    audio_queue.resume();
    debug!("AUDIO SPEC: {:?}", audio_queue.spec());

    let event_pump = sdl_context.event_pump().unwrap();

    let mut renderer = renderer::Renderer::new(canvas, audio_queue, event_pump);

    renderer.run(cartridge, args.debug, args.skip_boot_rom, args.dmg);
}
