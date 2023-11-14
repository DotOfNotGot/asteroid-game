use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::{WindowCanvas, TextureCreator};
use sdl2::video::WindowContext;
use sdl2::pixels::Color;
use sdl2::rect::{Rect, Point};

use specs::{World, WorldExt, Join, DispatcherBuilder, System};

use std::time::Duration;
use std::time::Instant;
use std::path::Path;
use std::collections::HashMap;

pub mod texture_manager;
pub mod utils;
pub mod components;
pub mod game;
pub mod asteroid;
pub mod missile;

const GAME_WIDTH: u32 = 1280;
const GAME_HEIGHT: u32 = 640;

fn render(canvas: &mut WindowCanvas, texture_manager: &mut texture_manager::TextureManager<WindowContext>, texture_creator: &TextureCreator<WindowContext>, font: &sdl2::ttf::Font, ecs: &World) -> Result<(), String> {

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

    let gamedatas = ecs.read_storage::<components::GameData>();
    for gamedata in (gamedatas).join() {
        let score: String = "Score: ".to_string() + &gamedata.score.to_string();
        let surface = font
                .render(&score)
                .blended(Color::RGBA(15, 180, 75, 255))
                .map_err(|e| e.to_string())?;
        
        let texture = texture_creator
                .create_texture_from_surface(&surface)
                .map_err(|e| e.to_string())?;
        let target = Rect::new(10 as i32, 0 as i32, 100 as u32, 50 as u32);

        canvas.copy(&texture, None, Some(target))?;
    }

    canvas.present();
    Ok(())
}

struct State { ecs: World }

fn main() -> Result<(), String> {
    println!("Game go vroom");
    
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem.window("Dumbass", GAME_WIDTH as u32, GAME_HEIGHT as u32)
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
    let mut fps = 61.0;

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
        game::update(&mut gs.ecs, &mut key_manager);
        dispatcher.dispatch(&gs.ecs);
        gs.ecs.maintain();
        render(&mut canvas, &mut texture_manager, &texture_creator, &font, &gs.ecs)?;
        
        frame_count += 1;
        let elapsed_time = start_time.elapsed().as_secs_f64();


        if elapsed_time >= 1.0{
            fps = frame_count as f64 / elapsed_time;
            frame_count = 0;
            println!("fps: {0}", fps);
            start_time = Instant::now();
        }

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32/60))
    }

    Ok(())

}

