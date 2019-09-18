
use ggez::graphics::Rect;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use crate::shooter::Vector2;

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
    pub fn get_size(&self) -> SizeU32 {
        SizeU32 {
            w: self.w,
            h: self.h
        }
    }
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SizeU32 {
    pub w: u32,
    pub h: u32
}
impl SizeU32 {
    pub fn to_vector2(&self) -> Vector2 {
        Vector2::new(self.w as f32, self.h as f32)
    }
    pub fn to_rect_f32(&self) -> Rect {
        Rect::new(0.0, 0.0, self.w as f32, self.h as f32)
    }
}
