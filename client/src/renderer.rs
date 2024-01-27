use sdl2::{pixels::Color, rect::Rect, render::WindowCanvas, video::Window};

use crate::{game::GameContext, util::Point};

const BG_COLOR: Color = Color::RGB(24, 24, 24);
const FOOD_COLOR: Color = Color::RED;
const SNAKE_COLOR: Color = Color::BLUE;
const SNAKE_HEAD_COLOR: Color = Color::CYAN;
const ENEMY_BODY_COLOR: Color = Color::RGB(255, 100, 0);
const ENEMY_HEAD_COLOR: Color = Color::YELLOW;

pub const WINDOW_WIDTH: u32 = 800;
pub const WINDOW_HEIGHT: u32 = 600;
pub const SCALE: u32 = 10;

pub struct Renderer {
    canvas: WindowCanvas,
}

impl Renderer {
    pub fn new(window: Window) -> Result<Renderer, String> {
        let canvas = window
            .into_canvas()
            .accelerated()
            .present_vsync()
            .build()
            .map_err(|e| e.to_string())?;

        Ok(Renderer { canvas })
    }

    pub fn draw_point(&mut self, point: &Point) -> Result<(), String> {
        let Point(x, y) = point;

        self.canvas
            .fill_rect(Rect::new(x * SCALE as i32, y * SCALE as i32, SCALE, SCALE))?;
        Ok(())
    }

    pub fn render(&mut self, context: &GameContext) -> Result<(), String> {
        // Background
        self.canvas.set_draw_color(BG_COLOR);
        self.canvas.clear();

        // Snake(s)
        for (id, snake) in context.snakes.iter() {
            if *id == context.snake_id {
                self.canvas.set_draw_color(SNAKE_COLOR);
            } else {
                self.canvas.set_draw_color(ENEMY_BODY_COLOR);
            }

            for point in snake.body.iter() {
                self.draw_point(point)?;
            }

            if *id == context.snake_id {
                self.canvas.set_draw_color(SNAKE_HEAD_COLOR);
            } else {
                self.canvas.set_draw_color(ENEMY_HEAD_COLOR);
            }
            self.draw_point(&snake.head)?;
        }

        // Food
        self.canvas.set_draw_color(FOOD_COLOR);
        self.draw_point(&context.food)?;

        self.canvas.present();

        Ok(())
    }
}
