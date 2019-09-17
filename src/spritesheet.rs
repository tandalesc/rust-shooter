
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpriteSheetData {
    pub frames: HashMap<String, Sprite>,
    meta: SpriteSheetMeta
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sprite {
    pub frame: Rect,
    rotated: bool,
    trimmed: bool,
    #[serde(rename = "spriteSourceSize")]
    pub sprite_source_size: Rect,
    #[serde(rename = "sourceSize")]
    pub source_size: Size
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct SpriteSheetMeta {
    app: String,
    version: String,
    image: String,
    format: String,
    size: Size,
    scale: String,
    #[serde(rename = "smartupdate")]
    smart_update: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Size {
    pub w: u32,
    pub h: u32
}
