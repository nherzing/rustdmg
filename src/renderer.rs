use std::{thread, time};
use std::collections::VecDeque;
use sdl2::pixels::{PixelFormatEnum, Color as PColor};
use sdl2::render::{WindowCanvas, TextureCreator, Texture};
use sdl2::audio::AudioQueue;
use sdl2::rect::Rect;
use samplerate::{Samplerate, ConverterType};
use crate::clocks::{CLOCK_FREQ, AUDIO_SAMPLE_RATE, NS_PER_SCREEN_REFRESH, SAMPLES_PER_FRAME, NS_PER_SAMPLE};

const WHITE: PColor = PColor { r: 114, g: 129, b: 77, a: 255 };
const LIGHT_GRAY: PColor = PColor { r: 86, g: 107, b: 86, a: 255 };
const DARK_GRAY: PColor = PColor { r: 66, g: 92, b: 83, a: 255 };
const BLACK: PColor = PColor { r: 63, g: 82, b: 80, a: 255 };
const OFF: PColor = PColor { r: 140, g: 128, b: 47, a: 255 };
pub const GAME_WIDTH: usize = 160;
pub const GAME_HEIGHT: usize = 144;

#[derive(Copy, Clone, Debug)]
pub enum Color {
    White,
    LightGray,
    DarkGray,
    Black,
    Off
}

impl Color {
    fn to_pcolor(&self) -> PColor {
        match self {
            Color::White => WHITE,
            Color::LightGray => LIGHT_GRAY,
            Color::DarkGray => DARK_GRAY,
            Color::Black => BLACK,
            Color::Off => OFF
        }
    }
}

pub struct Renderer<'a> {
    canvas: &'a mut WindowCanvas,
    game_texture: Texture<'a>,
    bg_tile_texture: Texture<'a>,
    audio_data: Vec<f32>,
    audio_converter: Samplerate,
    audio_queue: &'a mut AudioQueue<f32>,
    frame_buffer_queue: VecDeque<Vec<Color>>
}

impl<'a> Renderer<'a> {
    pub fn new(canvas: &'a mut WindowCanvas, texture_creator: &'a TextureCreator<sdl2::video::WindowContext>, audio_queue: &'a mut AudioQueue<f32>) -> Self {
        let game_texture = texture_creator.create_texture_streaming(
            PixelFormatEnum::RGB24, GAME_WIDTH as u32, GAME_HEIGHT as u32
        ).unwrap();
        let bg_tile_texture = texture_creator.create_texture_streaming(
            PixelFormatEnum::RGB24, 128, 128
        ).unwrap();

        let converter = Samplerate::new(ConverterType::SincMediumQuality, CLOCK_FREQ, AUDIO_SAMPLE_RATE, 2).unwrap();
        Renderer {
            canvas, game_texture, bg_tile_texture,
            audio_converter: converter,
            audio_data: Vec::new(),
            audio_queue,
            frame_buffer_queue: VecDeque::new()
        }
    }

    pub fn push_frame_buffer(&mut self, frame_buffer: &[Color]) {
        self.flush_audio();
        self.frame_buffer_queue.push_back(frame_buffer.to_owned());
        if self.frame_buffer_queue.len() >= 3|| self.until_draw().is_none() {
            self.wait_for_frame();
            self.draw_frame();
        }
    }

    fn until_draw(&self) -> Option<time::Duration> {
        let buffered_samples = (self.audio_queue.size() / 2 / 4) as u64;
        let ns_buffered = buffered_samples * NS_PER_SAMPLE;
        let ns_want = NS_PER_SCREEN_REFRESH * (self.frame_buffer_queue.len() as u64 - 1);
        time::Duration::from_nanos(ns_buffered).checked_sub(time::Duration::from_nanos(ns_want))
    }

    fn wait_for_frame(&mut self) {
        match self.until_draw() {
            None => {
                debug!("Dropping frame...");
                self.frame_buffer_queue.pop_front();
                if !self.frame_buffer_queue.is_empty() {
                    self.wait_for_frame();
                }
            }
            Some(d) => {
                if d.as_nanos() > NS_PER_SCREEN_REFRESH as u128 {
                    thread::sleep(d.checked_sub(time::Duration::from_nanos(NS_PER_SCREEN_REFRESH)).unwrap());
                }
            }
        }
    }

    pub fn draw_frame(&mut self) {
        match &self.frame_buffer_queue.pop_front() {
            None => {
                debug!("No framebuffer ready!!!");
            }
            Some(frame_buffer) => {
                self.game_texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
                    for y in 0..GAME_HEIGHT {
                        for x in 0..GAME_WIDTH {
                            let offset = y*pitch + x*3;
                            let color = frame_buffer[y * GAME_WIDTH + x].to_pcolor();
                            buffer[offset] = color.r;
                            buffer[offset + 1] = color.g;
                            buffer[offset + 2] = color.b;
                        }
                    }
                }).unwrap();

                let game_rect = Rect::new(0, 0, 2 * GAME_WIDTH as u32, 2 * GAME_HEIGHT as u32);
                let bg_tile_rect = Rect::new(game_rect.width() as i32 + 50, 0, 128 * 2, 128 * 2);

                self.canvas.clear();
                self.canvas.copy(&self.game_texture, None, Some(game_rect)).unwrap();
                self.canvas.copy(&self.bg_tile_texture, None, Some(bg_tile_rect)).unwrap();
                self.canvas.present();
            }
        }
    }

    pub fn queue_audio(&mut self, data: &[f32]) {
        self.audio_data.extend_from_slice(data);
    }

    fn flush_audio(&mut self) {
        if self.audio_queue.size() < 100 {
            debug!("AUDIO QUEUE SIZE: {}", self.audio_queue.size());
        }
        let samples_per_frame = SAMPLES_PER_FRAME as usize;
        if self.audio_data.len() < samples_per_frame {
            let v = *self.audio_data.last().unwrap();
            for _ in 0..(samples_per_frame - self.audio_data.len()) {
                self.audio_data.push(v);
            }
        }
        let data = self.audio_converter.process(&self.audio_data[0..samples_per_frame]).unwrap();
        self.audio_queue.queue(&data);
        self.audio_data.clear();
    }

    pub fn update_bg_tile_texture(&mut self, frame_buffer: &[Color]) {
        self.bg_tile_texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for y in 0..128 {
                for x in 0..128 {
                    let offset = y*pitch + x*3;
                    let color = frame_buffer[y * 128 + x].to_pcolor();
                    buffer[offset] = color.r;
                    buffer[offset + 1] = color.g;
                    buffer[offset + 2] = color.b;
                }
            }
        }).unwrap();
    }
}
