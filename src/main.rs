use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::{WindowCanvas, TextureCreator, Texture};
use sdl2::video::WindowContext;
use sdl2::pixels::Color;
use sdl2::rect::{Rect, Point};

use specs::{World, WorldExt, Join, DispatcherBuilder, System};

use std::time::Instant;
use std::path::Path;
use std::collections::HashMap;
use std::vec::Vec;

pub mod texture_manager;
pub mod utils;
pub mod components;
pub mod game;
pub mod asteroid;
pub mod missile;

const GAME_WIDTH: u32 = 1280;
const GAME_HEIGHT: u32 = 640;

pub struct UIElement<'a>{
    texture : Texture<'a>,
    position : Rect
}


fn render(canvas: &mut WindowCanvas, texture_manager: &mut texture_manager::TextureManager<WindowContext>, ecs: &World, current_score : u32, ui_elements : &Vec<UIElement>) -> Result<(), String> {

    let color = Color::RGBA(0, 10, 100, 255);

    canvas.set_draw_color(color);
    canvas.clear();

    let positions = ecs.read_storage::<components::Position>();
    let renderables = ecs.read_storage::<components::Renderable>();

    for(renderable, pos) in (&renderables, &positions).join(){
        let src = Rect::new(0, 0, renderable.i_w, renderable.i_h);
        let x: i32 = pos.x as i32;
        let y: i32 = pos.y as i32;
        let dest = Rect::new(x - ((renderable.o_w/2) as i32), y - ((renderable.o_h/2) as i32), renderable.o_w, renderable.o_h);

        let center = Point::new((renderable.o_w/2) as i32, (renderable.o_h/2) as i32);
        let texture = texture_manager.load(&renderable.tex_name)?;
        canvas.copy_ex(
            &texture,
            src,
            dest,
            renderable.rot,
            center,
            false,
            false
        )?;
    }

    for ui_element in ui_elements  {
        canvas.copy(&ui_element.texture, None, Some(ui_element.position))?;
    }

    canvas.present();
    Ok(())
}

struct State { ecs: World }

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem.window("Asteroid game", GAME_WIDTH as u32, GAME_HEIGHT as u32)
        .position_centered()
        .build()
        .expect("video subsystem fucked up");
    let mut canvas = window.into_canvas().build()
                                                    .expect("Canvas go oof");
    let texture_creator = canvas.texture_creator();
    let mut texture_manager = texture_manager::TextureManager::new(&texture_creator);

    // load images
    texture_manager.load("img/ship.png")?;
    texture_manager.load("img/asteroid.png")?;
    texture_manager.load("img/bullet.png")?;

    // prepare fonts
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
    let font_path: &Path = Path::new(&"fonts/rainyhearts.ttf");
    let mut font = ttf_context.load_font(font_path, 128)?;


    let mut event_pump = sdl_context.event_pump()?;
    let mut key_manager: HashMap<String, bool> = HashMap::new();

    let mut gs = State {
        ecs: World::new()
    };
    gs.ecs.register::<components::Position>();
    gs.ecs.register::<components::Renderable>();
    gs.ecs.register::<components::Player>();
    gs.ecs.register::<components::Asteroid>();
    gs.ecs.register::<components::Missile>();
    gs.ecs.register::<components::GameData>();

    let mut dispatcher = DispatcherBuilder::new()
                                                    .with(asteroid::AsteroidMover, "asteroid_mover", &[])
                                                    .with(asteroid::AsteroidCollider, "asteroid_collider", &[])
                                                    .with(missile::MissileMover, "missile_mover", &[])
                                                    .with(missile::MissileStriker, "missile_striker", &[])
                                                    .build();

    game::load_world(&mut gs.ecs);
    let mut frame_count: u32 = 0;
    let mut start_time = Instant::now();
    let mut fixed_step_time = Instant::now();
    let mut fps: f64 = 0.0;

    let mut current_score : u32 = 9999;
    let mut ui_elements : Vec<UIElement> = Vec::new();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {
                    break 'running;
                },
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                },
                Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
                    utils::key_down(&mut key_manager, " ".to_string());
                },
                Event::KeyUp { keycode: Some(Keycode::Space), .. } => {
                    utils::key_up(&mut key_manager, " ".to_string());
                },
                Event::KeyDown { keycode: Some(Keycode::J), .. } => {
                    utils::key_down(&mut key_manager, "J".to_string());
                },
                Event::KeyUp { keycode: Some(Keycode::J), .. } => {
                    utils::key_up(&mut key_manager, "J".to_string());
                },
                Event::KeyDown { keycode, .. } =>{
                    match keycode {
                        None => {},
                        Some(key) => {
                            utils::key_down(&mut key_manager, key.to_string());
                        }
                    }
                },
                Event::KeyUp{ keycode, .. } =>{
                    match keycode {
                        None => {},
                        Some(key) => {
                            utils::key_up(&mut key_manager, key.to_string());
                        }
                    }
                },
                _ => {}
            }
        }
        frame_count += 1;
        let elapsed_time = start_time.elapsed().as_secs_f64();
        let fixed_step_elapsed_time = fixed_step_time.elapsed().as_secs_f64();
        
        // Used to seperate the game logic into a fixed timestep so that it's not affected by the current render frame rate.
        if fixed_step_elapsed_time >= 1.0/60.0 {
            game::update(&mut gs.ecs, &mut key_manager);
            dispatcher.dispatch(&gs.ecs);
            gs.ecs.maintain();
            fixed_step_time = Instant::now();
        }

        // Fps counter only used 
        if elapsed_time >= 1.0{
            fps = frame_count as f64 / elapsed_time;
            frame_count = 0;
            start_time = Instant::now();
        }
        
        {
        let gamedatas = gs.ecs.read_storage::<components::GameData>();

        for gamedata in (gamedatas).join() {
            let new_score = gamedata.score;
    
            // If score hasn't changed then we don't need to create a new UI Element.
            if new_score == current_score && frame_count != 0{
                continue;
            }

            current_score = new_score;

            // Clears the UI Element so that we can replace it with one reflecting the new score.

            ui_elements.clear();

            let color = Color::RGBA(15, 180, 75, 255);

            let score: String = "Score: ".to_string() + &current_score.to_string();
            let surface = font
                    .render(&score)
                    .blended(color)
                    .map_err(|e| e.to_string())?;
            
            let texture : Texture = texture_creator
                    .create_texture_from_surface(&surface)
                    .map_err(|e| e.to_string())?;
            let target = Rect::new(10 as i32, 0 as i32, 100 as u32, 50 as u32);
    
            let score_ui : UIElement = UIElement{texture : texture, position: target};
            
            ui_elements.push(score_ui);
            
            let asteroid_count = "Asteroid count: ".to_string() + &gs.ecs.read_storage::<components::Asteroid>().count().to_string();

            let asteroid_surface = font
                    .render(&asteroid_count)
                    .blended(color)
                    .map_err(|e| e.to_string())?;
            
            let asteroid_count_texture : Texture = texture_creator
                    .create_texture_from_surface(asteroid_surface)
                    .map_err(|e| e.to_string())?;
            let asteroid_count_target = Rect::new(10 as i32, 40 as i32, 300 as u32, 50 as u32);

            let asteroid_count_ui : UIElement = UIElement { texture: asteroid_count_texture, position: asteroid_count_target };

            ui_elements.push(asteroid_count_ui);

            let fps_display = "FPS: ".to_string() + &(fps as u32).to_string();

            let fps_surface = font
                    .render(&fps_display)
                    .blended(color)
                    .map_err(|e| e.to_string())?;
            
            let fps_texture : Texture = texture_creator
                    .create_texture_from_surface(fps_surface)
                    .map_err(|e| e.to_string())?;
            let fps_target = Rect::new(10 as i32, 80 as i32, 150 as u32, 50 as u32);

            let asteroid_count_ui : UIElement = UIElement { texture: fps_texture, position: fps_target };

            ui_elements.push(asteroid_count_ui);
        }



        }

        // Renders all the textures to the window.
        render(&mut canvas, &mut texture_manager, &gs.ecs, current_score, &ui_elements)?;
    }

    Ok(())

}

