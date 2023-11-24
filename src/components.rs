use specs::prelude::*;
use specs_derive::Component;
use vector2d::Vector2D;


#[derive(Component)]
pub struct Position{
    pub x: f64,
    pub y: f64,
    pub rot: f64
}

#[derive(Component)]
pub struct Renderable {
    // Texture name
    pub tex_name: String,
    // Width of src texture
    pub i_w: u32,
    // Height of src texture
    pub i_h: u32,
    // Width of output rect
    pub o_w: u32,
    // Height of output rect
    pub o_h: u32,
    // Offset number for sprite sheet
    pub frame: u32,
    // idk
    pub total_frames: u32,
    // Output rotation of texture
    pub rot: f64
}

#[derive(Component)]
pub struct Player {
    pub impulse: Vector2D<f64>,
    pub cur_speed: Vector2D<f64>
}

#[derive(Component)]
pub struct Asteroid {
    pub speed: f64,
    pub rot_speed: f64
}

#[derive(Component)]
pub struct Missile {
    pub speed: f64,
}

pub struct PendingAsteroid {
    pub x: f64,
    pub y: f64,
    pub rot: f64,
    pub size: u32
}

#[derive(Component)]
pub struct GameData {
    pub score: u32,
    pub level: u32,
    pub god_mode: bool
}