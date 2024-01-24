use std::collections::VecDeque;

use rand::{rngs::ThreadRng, Rng};

use crate::{renderer::SCALE, util::Point, WINDOW_HEIGHT, WINDOW_WIDTH};

const WIDTH: i32 = (WINDOW_WIDTH / SCALE) as i32;
const HEIGHT: i32 = (WINDOW_HEIGHT / SCALE) as i32;

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
    direction: Direction,
}

impl Snake {
    pub fn default() -> Self {
        let mut body = VecDeque::new();
        body.push_back(Point(2, 3));

        Snake {
            body,
            head: Point(3, 3),
            direction: Direction::Right,
        }
    }

    pub fn update_head(&mut self) {
        let head = self.head;

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

        self.body.push_back(self.head);
        self.head = next_head;
    }

    pub fn change_direction(&mut self, direction: Direction) {
        if self.direction == direction {
            return;
        }

        let old_dir = self.direction;

        if (old_dir == Direction::Up && direction != Direction::Down)
            || (old_dir == Direction::Down && direction != Direction::Up)
            || (old_dir == Direction::Left && direction != Direction::Right)
            || (old_dir == Direction::Right && direction != Direction::Left)
        {
            self.direction = direction;
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

    pub fn update(&mut self, direction: Direction) {
        self.snake.change_direction(direction);
        self.snake.update_head();

        let snake_head = self.snake.head;

        if snake_head == self.food {
            self.spawn_food();
        } else {
            self.snake.body.pop_front();
        }
    }

    fn spawn_food(&mut self) {
        let x = self.rng.gen_range(0..WIDTH);
        let y = self.rng.gen_range(0..HEIGHT);

        let new_food = Point(x, y);

        if self.snake.body.contains(&new_food) {
            self.spawn_food();
            return;
        }

        self.food = new_food;
    }
}
