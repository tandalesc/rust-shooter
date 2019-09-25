
use ggez::graphics::Rect;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpriteAnimationSystem {
    store: Vec<SpriteSheetAnimation>
}
impl SpriteAnimationSystem {
    pub fn new() -> SpriteAnimationSystem {
        SpriteAnimationSystem { store: vec![] }
    }
    pub fn add_anim(&mut self, anim: SpriteSheetAnimation) -> usize {
        self.store.push(anim);
        self.store.len()-1
    }
    pub fn add_registered_anim(&mut self, registry_key: String, registry: &SpriteAnimationRegistry) -> Option<usize> {
        if let Some(anim) = registry.get_anim(registry_key) {
            Some(self.add_anim(SpriteSheetAnimation::new(anim)))
        } else {
            None
        }
    }
    pub fn remove_anim(&mut self, anim_handle: usize) {
        self.store.remove(anim_handle);
    }
    pub fn time_tick(&mut self, tick: f32) {
        for anim in &mut self.store {
            anim.time_tick(tick);
        }
    }
    pub fn get_anim(&self, anim_handle: usize) -> Option<&SpriteSheetAnimation> {
        if let Some(anim) = self.store.get(anim_handle) {
            Some(anim)
        } else {
            None
        }
    }
    pub fn get_frame(&self, anim_handle: usize) -> Option<&String> {
        if let Some(anim) = self.store.get(anim_handle) {
            Some(anim.get_frame())
        } else {
            None
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpriteAnimationRegistry {
    store: HashMap<String, SpriteAnimation>
}
impl SpriteAnimationRegistry {
    pub fn new() -> SpriteAnimationRegistry {
        SpriteAnimationRegistry { store: HashMap::new() }
    }
    pub fn add_anim(&mut self, anim_key: String, anim: SpriteAnimation) {
        self.store.insert(anim_key, anim);
    }
    pub fn get_anim(&self, anim_key: String) -> Option<&SpriteAnimation> {
        if let Some(anim) = self.store.get(&anim_key) {
            Some(&anim)
        } else {
            None
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpriteAnimation {
    pub frames: Vec<String>,
    pub time_per_frame: f32,
    pub loop_anim: bool
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpriteSheetAnimation {
    animation: SpriteAnimation,
    current_time: f32,
    current_frame: usize,
    pub finished: bool
}
impl SpriteSheetAnimation {
    pub fn new(animation: &SpriteAnimation) -> SpriteSheetAnimation {
        SpriteSheetAnimation {
            animation: animation.clone(),
            current_time: 0.0,
            current_frame: 0,
            finished: false
        }
    }
    pub fn time_tick(&mut self, tick: f32) {
        if !self.finished {
            if self.current_time+tick > self.animation.time_per_frame {
                self.current_time = 0.0;
                self.current_frame = if self.current_frame+1 < self.animation.frames.len() {
                    self.current_frame+1
                } else if self.animation.loop_anim {
                    0
                } else {
                    self.finished = true;
                    self.current_frame
                };
            } else {
                self.current_time += tick;
            }
        }
    }
    pub fn get_frame(&self) -> &String {
        &self.animation.frames[self.current_frame]
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
