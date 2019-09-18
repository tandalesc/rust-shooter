
use ggez::graphics::Rect;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

pub struct SpriteSheetAnimation {
    frames: Vec<String>,
    time_per_frame: f32,
    current_time: f32,
    current_frame: usize
}
impl SpriteSheetAnimation {
    pub fn new(frames: Vec<String>, time_per_frame: f32) -> SpriteSheetAnimation {
        SpriteSheetAnimation {
            frames: frames,
            time_per_frame: time_per_frame,
            current_time: 0.0,
            current_frame: 0
        }
    }
    pub fn time_tick(&mut self, tick: f32) {
        if self.current_time+tick > self.time_per_frame {
            self.current_time = 0.0;
            self.current_frame = if self.current_frame+1 < self.frames.len() {
                self.current_frame+1
            } else {
                0
            };
        } else {
            self.current_time += tick;
        }
    }
    pub fn get_frame(&self) -> &String {
        &self.frames[self.current_frame]
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpriteSheetData {
    pub frames: HashMap<String, SpriteData>,
    pub meta: SpriteSheetMeta
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpriteData {
    pub frame: RectU32,
    rotated: bool,
    trimmed: bool,
    #[serde(rename = "spriteSourceSize")]
    pub sprite_source_size: RectU32,
    #[serde(rename = "sourceSize")]
    pub source_size: SizeU32
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpriteSheetMeta {
    app: String,
    version: String,
    image: String,
    format: String,
    pub size: SizeU32,
    scale: String,
    #[serde(rename = "smartupdate")]
    smart_update: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RectU32 {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32
}
impl RectU32 {
    pub fn to_rect_f32(&self) -> Rect {
        Rect::new(self.x as f32, self.y as f32, self.w as f32, self.h as f32)
    }
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SizeU32 {
    pub w: u32,
    pub h: u32
}
impl SizeU32 {
    pub fn to_rect_f32(&self) -> Rect {
        Rect::new(0.0, 0.0, self.w as f32, self.h as f32)
    }
}
