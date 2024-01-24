use std::collections::VecDeque;

use rand::{rngs::ThreadRng, Rng};

use crate::{renderer::SCALE, util::Point, WINDOW_HEIGHT, WINDOW_WIDTH};

const WIDTH: i32 = (WINDOW_WIDTH / SCALE) as i32;
const HEIGHT: i32 = (WINDOW_HEIGHT / SCALE) as i32;

#[derive(PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub struct Snake {
    pub position: VecDeque<Point>,
    direction: Direction,
}

impl Snake {
    pub fn default() -> Self {
        let mut position = VecDeque::new();
        position.push_back(Point(2, 3));
        position.push_back(Point(3, 3));

        Snake {
            position,
            direction: Direction::Right,
        }
    }

    pub fn update_pos(&mut self) {
        let head = *self.position.back().unwrap();

        let mut next_head = match self.direction {
            Direction::Up => head + Point(0, -1),
            Direction::Down => head + Point(0, 1),
            Direction::Right => head + Point(1, 0),
            Direction::Left => head + Point(-1, 0),
        };

        match next_head {
            Point(-1, _) => next_head.0 = WIDTH - 1,
            Point(WIDTH, _) => next_head.0 = 0,
            Point(_, -1) => next_head.1 = HEIGHT - 1,
            Point(_, HEIGHT) => next_head.1 = 0,
            _ => {}
        };

        self.position.push_back(next_head);
    }

    pub fn move_up(&mut self) {
        if self.direction != Direction::Down {
            self.direction = Direction::Up;
        }
    }

    pub fn move_down(&mut self) {
        if self.direction != Direction::Up {
            self.direction = Direction::Down;
        }
    }

    pub fn move_left(&mut self) {
        if self.direction != Direction::Right {
            self.direction = Direction::Left;
        }
    }

    pub fn move_right(&mut self) {
        if self.direction != Direction::Left {
            self.direction = Direction::Right;
        }
    }
}

pub struct GameContext {
    pub snake: Snake,
    pub food: Point,
    rng: ThreadRng,
}

impl GameContext {
    pub fn new() -> Self {
        GameContext {
            snake: Snake::default(),
            food: Point(10, 4),
            rng: rand::thread_rng(),
        }
    }

    pub fn update(&mut self) {
        self.snake.update_pos();

        let snake_head = *self.snake.position.back().unwrap();

        if snake_head == self.food {
            self.spawn_food();
        } else {
            self.snake.position.pop_front();
        }
    }

    fn spawn_food(&mut self) {
        let x = self.rng.gen_range(0..WIDTH);
        let y = self.rng.gen_range(0..HEIGHT);

        let new_food = Point(x, y);

        if self.snake.position.contains(&new_food) {
            self.spawn_food();
            return;
        }

        self.food = new_food;
    }
}
