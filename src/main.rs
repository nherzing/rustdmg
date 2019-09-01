use std::fs;
use structopt::StructOpt;


mod memory_bus;
mod cpu;
mod gameboy;

#[derive(StructOpt)]
struct Cli {
    #[structopt(short, long)]
    debug: bool,
    #[structopt(parse(from_os_str))]
    cartridge_path: std::path::PathBuf,
}

fn main() {
    let args = Cli::from_args();
    let cartridge = fs::read(args.cartridge_path).unwrap();

    if args.debug {
        println!("Debug enabled!");
    }
    gameboy::Gameboy::boot(&cartridge, args.debug);
}
