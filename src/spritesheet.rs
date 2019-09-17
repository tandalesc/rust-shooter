
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpriteSheetData {
    pub frames: HashMap<String, SpriteData>,
    meta: SpriteSheetMeta
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
struct SpriteSheetMeta {
    app: String,
    version: String,
    image: String,
    format: String,
    size: SizeU32,
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
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SizeU32 {
    pub w: u32,
    pub h: u32
}
