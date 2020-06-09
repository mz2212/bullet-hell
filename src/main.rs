extern crate sdl2;

use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::KeyboardState;
use sdl2::keyboard::Scancode;
use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::ttf;

use rand::{thread_rng, Rng};

const PIXEL_MUL: u32 = 4; // The pixel multiplier, basically the scale. The canvas logical size gets divided by this.

type Location = (i32, i32);
type Size = (u32, u32);

struct Player {
    loc: Location,
    size: Size, // The size of the sprite. It could feasibly be changed to strech the sprite.
    shoot_timer: u32,
}

struct Enemy {
    loc: Location,
    size: Size,
    vel: Location,
    shoot_timer: u32,
    rot_angle: f64,
}

#[derive(Debug, Copy, Clone)]
struct Projectile {
    loc: Location,
    size: Size,
    vel: Location, // Techincally the velocity. Just didn't want to make another type.
    rot_angle: f64,
    p_type: ProjectileT,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum ProjectileT {
    Player,
    Enemy,
}

//TODO: Add the various screens (lose, start), Make it so the player can lose.

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Bullet Hell", 1280, 720)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let (mut width, mut height) = canvas.window().size();
    width /= PIXEL_MUL;
    height /= PIXEL_MUL;
    canvas.set_logical_size(width, height).unwrap();
    let ttf_handler = ttf::init().unwrap();
    let font = ttf_handler.load_font("HABESHAPIXELS.ttf", 14).unwrap();
    let texture_creator = canvas.texture_creator();
    
    // Thanks to: https://gigi.nullneuron.net/gigilabs/loading-images-in-sdl2-with-sdl_image/
    // for the general idea of how to load a texture. sdl2 crate docs helped tons too
    let player_image = texture_creator.load_texture("player.png").unwrap();
    let enemy_image = texture_creator.load_texture("enemy.png").unwrap();
    let projectile_image = texture_creator.load_texture("projectile.png").unwrap();
    let eprojectile_image = texture_creator.load_texture("enemy_projectile.png").unwrap();
    
    let mut rng = thread_rng();
    
    let mut player = Player {
        loc: (width as i32/2-12, height as i32 - 12),
        size: (24, 12),
        shoot_timer: 0,
    };
    
    let mut projectiles: Vec<Projectile> = Vec::new();
    let mut enemies: Vec<Enemy> = Vec::new();
    let mut spawn_timer = 40;
    let mut score = 0;
    let mut sdl_quit = false;
    
    while !sdl_quit {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => sdl_quit = true,
                _ => {}
            }
        }

        let keyboard_state = KeyboardState::new(&event_pump);
        let mut speed_divisor = 1;
        if keyboard_state.is_scancode_pressed(Scancode::LShift) {
            speed_divisor = 2;
        }
        if keyboard_state.is_scancode_pressed(Scancode::A) {
            player.loc.0 -= 2/speed_divisor;
        }
        if keyboard_state.is_scancode_pressed(Scancode::D) {
            player.loc.0 += 2/speed_divisor;
        }
        if keyboard_state.is_scancode_pressed(Scancode::W) {
            player.loc.1 -= 2/speed_divisor;
        }
        if keyboard_state.is_scancode_pressed(Scancode::S) {
            player.loc.1 += 2/speed_divisor;
        }
        if keyboard_state.is_scancode_pressed(Scancode::Space) {
            if player.shoot_timer == 0 {
                projectiles.push(Projectile {
                    loc: (player.loc.0 + player.size.0 as i32/2 - 4, player.loc.1),
                    size: (8, 11),
                    vel: (0, -3),
                    rot_angle: 0.0,
                    p_type: ProjectileT::Player,
                });
                player.shoot_timer = 30;
            } else {
                player.shoot_timer -= 1;
            }
        }

        if spawn_timer == 0 {
            enemies.push(Enemy {
                loc: (rng.gen_range(0, width) as i32, 0),
                size: (10, 10),
                vel: (0, 1),
                rot_angle: 0.0,
                shoot_timer: 20,
            });
            spawn_timer = rng.gen_range(20, 180);
        } else { spawn_timer -= 1; }


        
        // update logic
        let mut e = 0;
        while e < enemies.len() {
            enemies[e].loc.0 += enemies[e].vel.0;
            enemies[e].loc.1 += enemies[e].vel.1;
            if enemies[e].shoot_timer == 0 {
                projectiles.push(Projectile {
                    loc: (enemies[e].loc.0 + enemies[e].size.0 as i32/2-2, enemies[e].loc.1),
                    size: (5, 8),
                    vel: (0, 3),
                    rot_angle: 180.0,
                    p_type: ProjectileT::Enemy,
                });
                enemies[e].shoot_timer = 60;
            } else { enemies[e].shoot_timer -= 1; }
            if enemies[e].loc.1 > height as i32 {
                enemies.remove(e);
            }
            e += 1;
        }

        let mut p = 0;
        while p < projectiles.len() {
            projectiles[p].loc.0 += projectiles[p].vel.0;
            projectiles[p].loc.1 += projectiles[p].vel.1;
            let mut to_be_removed = false;
            e = 0;
            
            if (projectiles[p].loc.1 + projectiles[p].size.1 as i32) < 0 || projectiles[p].loc.1 > height as i32 {
                to_be_removed = true;
            }

            while e < enemies.len() {
                let enemy_rect = Rect::new(enemies[e].loc.0, enemies[e].loc.1, enemies[e].size.0, enemies[e].size.1);
                let projectile_rect = Rect::new(projectiles[p].loc.0, projectiles[p].loc.1, projectiles[p].size.0, projectiles[p].size.1);
                if enemy_rect.has_intersection(projectile_rect) && projectiles[p].p_type == ProjectileT::Player {
                    enemies.remove(e);
                    to_be_removed = true;
                    score += 1;
                }
                e += 1;
            }

            if to_be_removed {
                projectiles.remove(p);
            }
            p += 1;
        }
        
        

        // drawing logic
        canvas.set_draw_color(Color::BLACK);
        canvas.clear();

        for p in &projectiles {
            let projectile_rect = Rect::new(p.loc.0, p.loc.1, p.size.0, p.size.1);
            match p.p_type {
                ProjectileT::Player => canvas.copy_ex(&projectile_image, None, projectile_rect, p.rot_angle, None, false, false).unwrap(),
                ProjectileT::Enemy => canvas.copy_ex(&eprojectile_image, None, projectile_rect, p.rot_angle, None, false, false).unwrap()
            }
        }

        for e in &enemies {
            let enemy_rect = Rect::new(e.loc.0, e.loc.1, e.size.0, e.size.1);
            canvas.copy_ex(&enemy_image, None, enemy_rect, e.rot_angle, None, false, false).unwrap();
        }
        // Set up the rectangle target for the texture
        let player_rect = Rect::new(player.loc.0, player.loc.1, player.size.0, player.size.1);
        canvas.copy(&player_image, None, player_rect).unwrap();

        let text = font.render(format!("Score: {}", score).as_str()).solid(Color::WHITE).unwrap();
        let text_texture = text.as_texture(&texture_creator).unwrap();
        let text_rect = Rect::new(4, 4, text_texture.query().width, text_texture.query().height);
        canvas.copy(&text_texture, None, text_rect).unwrap();
        canvas.present();
    }
}
