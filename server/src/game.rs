use std::{
    collections::{HashMap, VecDeque},
    time::{SystemTime, UNIX_EPOCH},
};

use rand::{
    rngs::{OsRng, StdRng},
    Rng, SeedableRng,
};

use crate::util::Point;

const WIDTH: i32 = 80;
const HEIGHT: i32 = 60;

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
        self.direction = direction;
    }
}

pub struct GameContext {
    pub snakes: HashMap<u8, Snake>,
    pub food: Point,
    rng: StdRng,
}

impl GameContext {
    pub fn new() -> Self {
        GameContext {
            snakes: HashMap::new(),
            food: Point(10, 4),
            rng: StdRng::seed_from_u64(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            ),
        }
    }

    pub fn update(&mut self) {
        self.snakes.values_mut().for_each(Snake::update_head);

        // TODO: Handle snake updating

        // let snake_head = self.snake.head;

        // if snake_head == self.food {
        //     self.spawn_food();
        // } else {
        //     self.snake.body.pop_front();
        // }
    }

    fn spawn_food(&mut self) {
        let x = self.rng.gen_range(0..WIDTH);
        let y = self.rng.gen_range(0..HEIGHT);

        let new_food = Point(x, y);

        if self
            .snakes
            .values()
            .any(|snake| snake.body.contains(&new_food))
        {
            self.spawn_food();
            return;
        }

        self.food = new_food;
    }
}
