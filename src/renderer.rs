use std::{thread, time};
use sdl2::pixels::{PixelFormatEnum, Color as PColor};
use sdl2::render::{WindowCanvas, Texture};
use sdl2::audio::AudioQueue;
use sdl2::EventPump;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::{KeyboardState, Scancode, Keycode};
use sdl2::rect::Rect;
use samplerate::{Samplerate, ConverterType};
use crate::clocks::{CLOCK_FREQ, AUDIO_SAMPLE_RATE, NS_PER_SCREEN_REFRESH, NS_PER_SAMPLE};
use crate::cartridge::Cartridge;
use crate::gameboy::{Gameboy, Color, JoypadInput, GAME_WIDTH, GAME_HEIGHT};

// const UGLY_GREENS_PALETTE: [PColor; 5] = [
//     PColor { r: 114, g: 129, b: 77, a: 255 },
//     PColor { r: 86, g: 107, b: 86, a: 255 },
//     PColor { r: 66, g: 92, b: 83, a: 255 },
//     PColor { r: 63, g: 82, b: 80, a: 255 },
//     PColor { r: 140, g: 128, b: 47, a: 255 }
// ];
const BW_PALETTE: [PColor; 5] = [
    PColor { r: 255, g: 255, b: 255, a: 255 },
    PColor { r: 170, g: 170, b: 170, a: 255 },
    PColor { r: 85, g: 85, b: 85, a: 255 },
    PColor { r: 0, g: 0, b: 0, a: 255 },
    PColor { r: 0, g: 0, b: 0, a: 255 }
];

const SCANCODES: [Scancode; 8] = [
    Scancode::W, Scancode::S, Scancode::A, Scancode::D,
    Scancode::X, Scancode::Z, Scancode::L, Scancode::K
];

fn to_pcolor(color: Color) -> PColor {
    BW_PALETTE[match color {
        Color::White => 0,
        Color::LightGray => 1,
        Color::DarkGray => 2,
        Color::Black => 3,
        Color::Off => 4
    }]
}

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


pub struct Renderer {
    canvas: WindowCanvas,
    audio_converter: Samplerate,
    audio_queue: AudioQueue<f32>,
    event_pump: EventPump,
    frame_buffer: [Color; GAME_WIDTH * GAME_HEIGHT],
    bg_tile_map_frame_buffer: [Color; 128 * 192]
}

impl Renderer {
    pub fn new(canvas: WindowCanvas, audio_queue: AudioQueue<f32>, event_pump: EventPump) -> Self {
        let audio_converter = Samplerate::new(ConverterType::SincFastest, CLOCK_FREQ, AUDIO_SAMPLE_RATE, 2).unwrap();
        Renderer {
            canvas,
            audio_converter,
            audio_queue,
            event_pump,
            frame_buffer: [Color::Off; GAME_WIDTH * GAME_HEIGHT],
            bg_tile_map_frame_buffer: [Color::Off; 128 * 192]
        }
    }

    pub fn run(&mut self, cartridge: Cartridge, debug: bool, skip_boot_rom: bool) {
        self.canvas.window_mut().set_size(GAME_WIDTH as u32 * 4, GAME_HEIGHT as u32 * 4).unwrap();

        let texture_creator = self.canvas.texture_creator();
        let mut game_texture = texture_creator.create_texture_streaming(
            PixelFormatEnum::RGB24, GAME_WIDTH as u32, GAME_HEIGHT as u32
        ).unwrap();
        let mut bg_tile_map_texture = texture_creator.create_texture_streaming(
            PixelFormatEnum::RGB24, 256, 256
        ).unwrap();

        let mut audio_data = Vec::with_capacity(40_000);

        let mut gameboy = Gameboy::new(debug);
        gameboy.boot(cartridge.clone(), skip_boot_rom);

        let mut paused = false;
        'running: loop {
            let pressed = collect_pressed(&self.event_pump.keyboard_state());

            if !paused {
                gameboy.tick(&pressed, &mut self.frame_buffer, &mut audio_data);
                self.flush_audio(&audio_data);
                audio_data.clear();
                gameboy.fill_tile_framebuffer(&mut self.bg_tile_map_frame_buffer);

                self.wait_for_frame();
                self.draw_frame(&mut game_texture, &mut bg_tile_map_texture);
            } else {
                thread::sleep(time::Duration::from_millis(10));
            }

            for event in self.event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    Event::KeyDown {
                        keycode: Some(Keycode::Space),
                        ..
                    } => {
                        paused = !paused;
                    },
                    Event::KeyDown {
                        keycode: Some(Keycode::R),
                        ..
                    } => {
                        paused = false;
                        gameboy = Gameboy::new(debug);
                        gameboy.boot(cartridge.clone(), skip_boot_rom);
                    }
                    Event::Window {
                        win_event: WindowEvent::Resized(_, h),
                        ..
                    } => {
                        let w = ((h as f32) * ((GAME_WIDTH as f32) / (GAME_HEIGHT as f32))) as u32;
                        debug!("Resized to: {}, {}", w, h);
                        self.canvas.window_mut().set_size(w, h as u32).unwrap();
                    }
                    _ => {}
                }
            }
        }
    }

    fn until_draw(&self) -> Option<time::Duration> {
        let buffered_samples = (self.audio_queue.size() / 2 / 4) as u64;
        let ns_buffered = buffered_samples * NS_PER_SAMPLE;
        let ns_want = NS_PER_SCREEN_REFRESH * 3 / 2;
        time::Duration::from_nanos(ns_buffered).checked_sub(time::Duration::from_nanos(ns_want))
    }

    fn wait_for_frame(&mut self) {
        match self.until_draw() {
            None => {
                debug!("Dropping frame...");
            }
            Some(d) => {
                if d.as_nanos() > NS_PER_SCREEN_REFRESH as u128 {
                    thread::sleep(d.checked_sub(time::Duration::from_nanos(NS_PER_SCREEN_REFRESH)).unwrap());
                }
            }
        }
    }

    fn draw_frame(&mut self, texture: &mut Texture, bg_tile_map_texture: &mut Texture) {
        texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for y in 0..GAME_HEIGHT {
                for x in 0..GAME_WIDTH {
                    let offset = y*pitch + x*3;
                    let color = to_pcolor(self.frame_buffer[y * GAME_WIDTH + x]);
                    buffer[offset] = color.r;
                    buffer[offset + 1] = color.g;
                    buffer[offset + 2] = color.b;
                }
            }
        }).unwrap();
        self.update_bg_tile_map_texture(bg_tile_map_texture);

        let game_rect = Rect::new(0, 0, 2 * GAME_WIDTH as u32, 2 * GAME_HEIGHT as u32);
        let bg_tile_rect = Rect::new(game_rect.width() as i32 + 50, 0, 128 * 4, 192 * 4);

        self.canvas.clear();
        self.canvas.copy(texture, None, game_rect).unwrap();
        self.canvas.copy(bg_tile_map_texture, None, bg_tile_rect).unwrap();
        self.canvas.present();
    }

    pub fn update_bg_tile_map_texture(&mut self, texture: &mut Texture) {
        texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for y in 0..192 {
                for x in 0..128 {
                    let offset = y*pitch + x*3;
                    let color = to_pcolor(self.bg_tile_map_frame_buffer[y * 128 + x]);
                    buffer[offset] = color.r;
                    buffer[offset + 1] = color.g;
                    buffer[offset + 2] = color.b;
                }
            }
        }).unwrap();
    }


    fn flush_audio(&mut self, audio_data: &[f32]) {
        if self.audio_queue.size() < 100 {
            debug!("AUDIO QUEUE SIZE: {}", self.audio_queue.size());
        }
        let data = self.audio_converter.process(&audio_data).unwrap();
        self.audio_queue.queue(&data);
    }
}
