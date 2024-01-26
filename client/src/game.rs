use std::collections::{HashMap, VecDeque};

use crate::{renderer::SCALE, util::Point, WINDOW_HEIGHT, WINDOW_WIDTH};

// const WIDTH: i32 = (WINDOW_WIDTH / SCALE) as i32;
// const HEIGHT: i32 = (WINDOW_HEIGHT / SCALE) as i32;

#[derive(PartialEq)]
pub enum State {
    Joining,
    Playing,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub struct Snake {
    pub body: VecDeque<Point>,
    pub head: Point,
}

impl Snake {
    pub fn new(body: VecDeque<Point>, head: Point) -> Self {
        Self { body, head }
    }

    // pub fn change_direction(&mut self, direction: Direction) {
    //     if self.direction == direction {
    //         return;
    //     }

    //     let old_dir = self.direction;

    //     if (old_dir == Direction::Up && direction != Direction::Down)
    //         || (old_dir == Direction::Down && direction != Direction::Up)
    //         || (old_dir == Direction::Left && direction != Direction::Right)
    //         || (old_dir == Direction::Right && direction != Direction::Left)
    //     {
    //         self.direction = direction;
    //     }
    // }
}

pub struct GameContext {
    pub snakes: HashMap<u8, Snake>,
    pub food: Point,
    pub state: State,
}

impl GameContext {
    pub fn new() -> Self {
        Self {
            snakes: HashMap::new(),
            food: Point(0, 0),
            state: State::Joining,
        }
    }
}
