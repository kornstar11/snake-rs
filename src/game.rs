extern crate log;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Clone, Serialize, Hash, PartialEq, Eq, Copy)]
pub struct Point {
    x: usize,
    y: usize,
}

impl Point {
    pub fn new(x: usize, y: usize) -> Point {
        Point { x: x, y: x }
    }
}

pub struct Box {
    start_point: Point,
    width: usize,
    height: usize
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Serialize)]
pub struct Snake {
    points: VecDeque<Point>,
    length: usize,
    direction: Direction,
    is_alive: bool,
}

impl Snake {
    pub fn new(length: usize, starting_point: Point) -> Snake {
        let mut new_queue: VecDeque<Point> = VecDeque::new();
        new_queue.push_front(starting_point);
        Snake {
            points: new_queue,
            length: length,
            direction: Direction::Right,
            is_alive: true
        }
    }

    pub fn grow(&mut self, grow_by: usize) {
        self.length = self.length + 1;
    }

    fn add_point(&mut self, point: Point) {
        self.points.push_front(point);
        while self.points.len() > self.length {
            self.points.pop_back();
        }
    }


    pub fn set_direction(&mut self, direction: Direction) {
        let is_valid_change = match (&direction, &self.direction) {
            (Direction::Left, Direction::Right) => {
                false
            }
            (Direction::Right, Direction::Left) => {
                false
            }
            (Direction::Up, Direction::Down) => {
                false
            }
            (Direction::Down, Direction::Up) => {
                false
            }
            _ => true
        };
        if is_valid_change {
            self.direction = direction;
        }
    }

    pub fn tick(&mut self) {
        if let Some(front) = self.points.front() {
            let new_point = match self.direction {
                Direction::Up => Point {
                    x: front.x,
                    y: front.y - 1,
                },
                Direction::Down => Point {
                    x: front.x,
                    y: front.y + 1,
                },
                Direction::Right => Point {
                    x: front.x + 1,
                    y: front.y,
                },
                Direction::Left => Point {
                    x: front.x - 1,
                    y: front.y,
                },
            };
            self.add_point(new_point);
        }
    }
}

#[derive(Debug)]
pub enum StateUpdate {
    Tick,
    ChangeDirection(usize, Direction),
    DropSnake(usize),
}

#[derive(Debug)]
pub struct GameState {
    snakes: HashMap<usize, Snake>,
    id_gen: AtomicUsize,
    points_set: HashSet<Point>,
    food_set: HashSet<Point>,
    x_size: usize,
    y_size: usize

}

impl GameState {
    pub fn new() -> GameState {
        let snakes: HashMap<usize, Snake> = HashMap::new();
        let points_set: HashSet<Point> = HashSet::new();
        let food_set: HashSet<Point> = HashSet::new();
        let id_gen = AtomicUsize::new(0);

        GameState { snakes, id_gen, points_set, food_set,x_size: 768, y_size: 512 }
    }

    fn tick(&mut self) {
        for (_, snake) in self.snakes.iter_mut() {
            snake.tick();
        }

        self.points_set.clear();
        for (_, snake) in self.snakes.iter_mut() {
            let is_dead = if let Some(head_point) = snake.points.get(0) {
                self.points_set.contains(head_point)
            } else {
                false
            };
            snake.is_alive = !is_dead;

            for &point in snake.points.iter() {
                self.points_set.insert(point);
            }

        }
        //TODO detect collisions
    }

    pub fn get_snakes_ref(&self) -> &HashMap<usize, Snake> {
        &self.snakes
    }

    pub fn get_snakes(&self) -> HashMap<usize, Snake> {
        self.snakes.clone()
    }

    pub fn handle(&mut self, update: StateUpdate) {
        log::debug!("Handling {:?}", update);
        match update {
            StateUpdate::Tick => {
                self.tick();
            }
            StateUpdate::ChangeDirection(id, direction) => {
                if let Some(snake) = self.snakes.get_mut(&id) {
                    snake.set_direction(direction);
                } else {
                    log::warn!("Missing id {}", id);
                }
            }
            StateUpdate::DropSnake(id) => {
                self.snakes.remove(&id);
            }
        }
    }

    pub fn create_snake(&mut self) -> usize {
        let new_id = self.id_gen.fetch_add(1, Ordering::SeqCst);
        let starting_point = Point { x: 10, y: 10 };
        self.snakes.insert(new_id, Snake::new(30, starting_point));
        new_id
    }
}

#[cfg(test)]
mod tests {
    use crate::game::*;

    #[test]
    fn test_snake_growth() {
        let start_point = Point { x: 10, y: 10 };

        let mut snake = Snake::new(3, start_point);

        for i in 0..10 {
            snake.tick();
            println!("Iter {} ${:?}", i, snake);
        }
        assert!(snake.length == 3);
        assert!(snake.points.len() == snake.length);
        snake.grow(1);
        for i in 0..10 {
            snake.tick();
            println!("Iter {} ${:?}", i, snake);
        }
        assert!(snake.length == 4);
        assert!(snake.points.len() == snake.length);
    }
    //todo test direction changes
}
