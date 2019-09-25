
use ggez::graphics::Rect;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

pub trait SpriteObject {
    fn get_frame(&self, sprite_system: &SpriteAnimationSystem) -> Option<String> { None }
    fn get_fractional_frame(&self, sprite_system: &SpriteAnimationSystem, sprite_data: &SpriteSheetData) -> Option<Rect> {
        if let Some(frame) = self.get_frame(sprite_system) {
            sprite_data.get_as_fractional_rect(frame)
        } else {
            None
        }
    }
    fn register_in_system(&mut self, sprite_system: &mut SpriteAnimationSystem, animation_registry: &SpriteAnimationRegistry) { }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpriteAnimationSystem {
    store: Vec<SpriteAnimationComponent>
}
impl SpriteAnimationSystem {
    pub fn new() -> SpriteAnimationSystem {
        SpriteAnimationSystem { store: vec![] }
    }
    pub fn add_anim(&mut self, anim: SpriteAnimationComponent) -> usize {
        self.store.push(anim);
        self.store.len()-1
    }
    pub fn add_registered_anim(&mut self, registry_key: String, registry: &SpriteAnimationRegistry) -> Option<usize> {
        if let Some(anim) = registry.get_anim(registry_key) {
            Some(self.add_anim(SpriteAnimationComponent::new(anim)))
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
    pub fn get_anim(&self, anim_handle: usize) -> Option<&SpriteAnimationComponent> {
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
pub struct SpriteAnimationComponent {
    animation: SpriteAnimation,
    current_time: f32,
    current_frame: usize,
    pub finished: bool
}
impl SpriteAnimationComponent {
    pub fn new(animation: &SpriteAnimation) -> SpriteAnimationComponent {
        SpriteAnimationComponent {
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
impl SpriteSheetData {
    pub fn get_as_fractional_rect(&self, sprite_sheet_key: String) -> Option<Rect> {
        if let Some(sprite_data) = self.frames.get(&sprite_sheet_key) {
            let spritesheet_rect = self.meta.size.to_rect_f32();
            let spr_rect = sprite_data.frame.to_rect_f32();
            Some(Rect::fraction(spr_rect.x, spr_rect.y, spr_rect.w, spr_rect.h, &spritesheet_rect))
        } else {
            None
        }
    }
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
