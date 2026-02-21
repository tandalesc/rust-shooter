// Display and world dimensions
pub const DISPLAY_WIDTH: f32 = 920.0;
pub const DISPLAY_HEIGHT: f32 = 690.0;
pub const WORLD_WIDTH: f32 = 920.0;
pub const WORLD_HEIGHT: f32 = 690.0;

// Physics
pub const FRICTION: f32 = 0.1;

// Enemy wave layout
pub const ENEMIES_PER_ROW: u8 = 7;
pub const ENEMY_ROWS: u8 = 3;

// Gameplay tuning
pub const ENEMY_SHOOT_CHANCE: usize = 10;
pub const NUM_STARS: usize = 300;
pub const PLAYER_EXP_PER_KILL: f32 = 40.0;
pub const PLAYER_INVINCIBILITY_FRAMES: u32 = 60;
pub const PLAYER_CONTACT_DAMAGE: f32 = 20.0;
pub const WAVE_GRACE_PERIOD: usize = 200;
pub const PLAYER_MAX_HEALTH: f32 = 100.0;
pub const PLAYER_MIN_HEALTH_RESTORE: f32 = 25.0;
pub const EXP_TO_LEVEL: f32 = 100.0;

// Debug
pub const SHOW_FRAMERATE: bool = false;
pub const SHOW_HITBOXES: bool = false;
pub const HITBOX_COLOR: [f32; 4] = [1.0, 0.1, 0.1, 0.4];
