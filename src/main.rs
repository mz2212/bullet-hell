extern crate sdl2;

use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::KeyboardState;
use sdl2::keyboard::Scancode;
use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::ttf;
use std::path::PathBuf;

use rand::{thread_rng, Rng};

const PIXEL_MUL: u32 = 4; // The pixel multiplier, basically the scale. The canvas logical size gets divided by this.

type Location = (i32, i32);
type Size = (u32, u32);

struct Star {
	loc: Location,
	size: Size,
	vel: Location,
}

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

#[derive(Debug, Clone)]
struct Text {
	loc: Location,
	text: String,
	color: Color,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum ProjectileT {
	Player,
	Enemy,
}

enum GameState {
	Title,
	Setup,
	Playing,
	Lose,
}

//TODO: Add the various screens (lose, start), Make it so the player can lose.

fn main() {
	let sdl_context = sdl2::init().unwrap();
	let mut event_pump = sdl_context.event_pump().unwrap();
	let video_subsystem = sdl_context.video().unwrap();
	let mut path = PathBuf::new();
	path.push(std::env::current_exe().unwrap());

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
	path.set_file_name("HABESHAPIXELS.ttf");
	let font = ttf_handler.load_font(path.as_path(), 14).unwrap();
	let texture_creator = canvas.texture_creator();
	
	// Thanks to: https://gigi.nullneuron.net/gigilabs/loading-images-in-sdl2-with-sdl_image/
	// for the general idea of how to load a texture. sdl2 crate docs helped tons too
	path.set_file_name("player.png");
	let player_image = texture_creator.load_texture(path.as_path()).unwrap();
	path.set_file_name("enemy.png");
	let enemy_image = texture_creator.load_texture(path.as_path()).unwrap();
	path.set_file_name("projectile.png");
	let projectile_image = texture_creator.load_texture(path.as_path()).unwrap();
	path.set_file_name("enemy_projectile.png");
	let eprojectile_image = texture_creator.load_texture(path.as_path()).unwrap();
	drop(path);
	
	let mut rng = thread_rng();
	
	let mut player = Player {
		loc: (width as i32/2-12, height as i32 - 12),
		size: (24, 12),
		shoot_timer: 0,
	};

	let score_text = Text {
		loc: (4, 4),
		text: String::from("Score: "),
		color: Color::WHITE,
	};

	let play_text = Text {
		loc: (width as i32/2, height as i32 - 25),
		text: String::from("Press Space to Play"),
		color: Color::WHITE,
	};

	let title_text = Text {
		loc: (width as i32/2, 10),
		text: String::from("Bullet Hell"),
		color: Color::WHITE,
	};

	let lose_text = Text {
		loc: (width as i32/2, height as i32 - 25),
		text: String::from("Press t to return to title"),
		color: Color::WHITE,
	};

	let menu_text = [title_text, play_text];
	
	let mut stars: Vec<Star> = Vec::new();
	let mut projectiles: Vec<Projectile> = Vec::new();
	let mut enemies: Vec<Enemy> = Vec::new();
	let mut spawn_timer = 40;
	let mut star_timer = true;
	let mut score = 0;
	let mut sdl_quit = false;
	let mut game_state = GameState::Title;
	
	while !sdl_quit {
		for event in event_pump.poll_iter() {
			match event {
				Event::Quit {..} => sdl_quit = true,
				_ => {}
			}
		}
		match game_state {
			GameState::Title => {
				let keyboard_state = KeyboardState::new(&event_pump);
				if keyboard_state.is_scancode_pressed(Scancode::Space) {
					game_state = GameState::Setup;
				}

				canvas.set_draw_color(Color::BLACK);
				canvas.clear();
				for text in menu_text.iter() {
					let pre_texture = font.render(text.text.as_str()).solid(text.color).unwrap();
					let texture = pre_texture.as_texture(&texture_creator).unwrap();
					let text_rect = Rect::new(text.loc.0-texture.query().width as i32/2, text.loc.1, texture.query().width, texture.query().height);
					canvas.copy(&texture, None, text_rect).unwrap();
				}
				canvas.present();
			},
			GameState::Setup => {
				stars = Vec::new();
				projectiles = Vec::new();
				enemies = Vec::new();
				spawn_timer = 40;
				star_timer = true;
				score = 0;
				player = Player {
					loc: (width as i32/2-12, height as i32 - 12),
					size: (24, 12),
					shoot_timer: 0,
				};
				game_state = GameState::Playing;
			},
			GameState::Playing => {
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

				if star_timer == true {
					stars.push(Star {
						loc: (rng.gen_range(0, width) as i32, 0),
						size: (1, 1),
						vel: (0, rng.gen_range(2, 5)),
					});
					star_timer = false;
				} else { star_timer = true; }

				// update logic
				let mut s = 0;
				while s < stars.len() {
					stars[s].loc.0 += stars[s].vel.0;
					stars[s].loc.1 += stars[s].vel.1;
					if stars[s].loc.1 > height as i32 {
						stars.remove(s);
					}
					s += 1;
				}

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

					let projectile_rect = Rect::new(projectiles[p].loc.0, projectiles[p].loc.1, projectiles[p].size.0, projectiles[p].size.1);

					while e < enemies.len() {
						let enemy_rect = Rect::new(enemies[e].loc.0, enemies[e].loc.1, enemies[e].size.0, enemies[e].size.1);
						if enemy_rect.has_intersection(projectile_rect) && projectiles[p].p_type == ProjectileT::Player {
							enemies.remove(e);
							to_be_removed = true;
							score += 1;
						}
						e += 1;
					}

					let player_rect = Rect::new(player.loc.0, player.loc.1, player.size.0, player.size.1);
					if player_rect.has_intersection(projectile_rect) && projectiles[p].p_type == ProjectileT::Enemy {
						game_state = GameState::Lose;
					}

					if to_be_removed {
						projectiles.remove(p);
					}
					p += 1;
				}



				// drawing logic
				canvas.set_draw_color(Color::BLACK);
				canvas.clear();

				canvas.set_draw_color(Color::WHITE);
				for s in &stars {
					let star_rect = Rect::new(s.loc.0, s.loc.1, s.size.0, s.size.1);
					canvas.draw_rect(star_rect).unwrap();
				}

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

				let text = font.render(format!("{} {}", score_text.text, score).as_str()).solid(score_text.color).unwrap();
				let text_texture = text.as_texture(&texture_creator).unwrap();
				let text_rect = Rect::new(score_text.loc.0, score_text.loc.1, text_texture.query().width, text_texture.query().height);
				canvas.copy(&text_texture, None, text_rect).unwrap();
				canvas.present();
			},
			GameState::Lose => {
				let keyboard_state = KeyboardState::new(&event_pump);
				if keyboard_state.is_scancode_pressed(Scancode::T) {
					game_state = GameState::Title;
				}

				canvas.set_draw_color(Color::BLACK);
				canvas.clear();
				let text = font.render(format!("{} {}", score_text.text, score).as_str()).solid(score_text.color).unwrap();
				let text_texture = text.as_texture(&texture_creator).unwrap();
				let text_rect = Rect::new(menu_text[0].loc.0-text_texture.query().width as i32/2, menu_text[0].loc.1, text_texture.query().width, text_texture.query().height);
				canvas.copy(&text_texture, None, text_rect).unwrap();
				let text = font.render(lose_text.text.as_str()).solid(lose_text.color).unwrap();
				let text_texture = text.as_texture(&texture_creator).unwrap();
				let text_rect = Rect::new(lose_text.loc.0-text_texture.query().width as i32/2, lose_text.loc.1, text_texture.query().width, text_texture.query().height);
				canvas.copy(&text_texture, None, text_rect).unwrap();
				canvas.present();
			}
		}
	}
}
