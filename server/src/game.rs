use std::{
    collections::{HashMap, VecDeque},
    time::{SystemTime, UNIX_EPOCH},
};

use rand::{rngs::StdRng, Rng, SeedableRng};

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

    pub fn spawn_snake(&mut self, snake_id: u8) {
        let mut occupied_points = Vec::new();

        occupied_points.push(self.food);

        for snake in self.snakes.values() {
            for pos in snake.body.iter() {
                occupied_points.push(*pos);
            }

            occupied_points.push(snake.head);
        }

        let mut p1 = Point(self.rng.gen_range(0..WIDTH), self.rng.gen_range(0..HEIGHT));
        let mut p2 = Point(p1.0 + 1, p1.1);

        while occupied_points.contains(&p1) || occupied_points.contains(&p2) {
            p1 = Point(self.rng.gen_range(0..WIDTH), self.rng.gen_range(0..HEIGHT));
            p2 = Point(p1.0 + 1, p1.1);
        }

        let mut body = VecDeque::new();
        body.push_back(p2);

        let snake = Snake {
            head: p1,
            body,
            direction: Direction::Right,
        };

        self.snakes.insert(snake_id, snake);
    }

    pub fn kill_snake(&mut self, snake_id: u8) {
        self.snakes.remove(&snake_id);
    }

    pub fn update(&mut self) -> (Option<u8>, Option<Vec<u8>>) {
        if self.snakes.is_empty() {
            return (None, None);
        }

        self.snakes.values_mut().for_each(Snake::update_head);

        let mut snake_eat = None;

        for (snake_id, snake) in self.snakes.iter_mut() {
            if snake.head == self.food {
                snake_eat = Some(*snake_id);
            } else {
                snake.body.pop_front();
            }
        }

        let mut points = Vec::new();
        let mut killed_snakes = Vec::new();

        for snake in self.snakes.values() {
            points.extend(snake.body.iter().cloned());
            points.push(snake.head);
        }

        for (snake_id, snake) in self.snakes.iter_mut() {
            if points.iter().filter(|p| **p == snake.head).count() > 1 {
                killed_snakes.push(*snake_id);
            }
        }

        if snake_eat.is_some() {
            self.spawn_food();
        }

        if killed_snakes.is_empty() {
            return (snake_eat, None);
        }

        (snake_eat, Some(killed_snakes))
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
