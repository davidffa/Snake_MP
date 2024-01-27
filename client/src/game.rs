use std::collections::{HashMap, VecDeque};

use crate::util::Point;

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
    pub old_tail: Point,
}

impl Snake {
    pub fn new(body: VecDeque<Point>, head: Point) -> Self {
        let old_tail = *body.front().unwrap();

        Self {
            body,
            head,
            old_tail,
        }
    }
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
