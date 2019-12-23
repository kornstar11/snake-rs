extern crate log;
extern crate rand;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use rand::Rng;

#[derive(Debug, Clone, Serialize, Hash, PartialEq, Eq, Copy)]
pub struct Point {
    x: usize,
    y: usize,
}

impl Point {
    pub fn new(x: usize, y: usize) -> Point {
        Point { x: x, y: y }
    }
}

#[derive(Debug, Clone, Serialize, Hash, PartialEq, Eq, Copy)]
pub struct BoxShape {
    start_point: Point,
    width: isize,
    height: isize
}

impl BoxShape {
    ///
    /// Makes a square box at a given starting point
    pub fn new(start_point: Point, size: isize) -> BoxShape {
        BoxShape {
            start_point: start_point,
            width: size,
            height: size
        }
    }

    pub fn intersects(&self, point: &Point) -> bool {
        let end_x = self.start_point.x as isize + self.width;
        let end_y = self.start_point.y as isize + self.height;

        let (start_x, end_x) = ((self.start_point.x as isize).min(end_x), (self.start_point.x as isize).max(end_x));
        let (start_y, end_y) = ((self.start_point.y as isize).min(end_y), (self.start_point.y as isize).max(end_y));

        point.x as isize >= start_x &&
            point.x as isize <= end_x &&
            point.y as isize >= start_y &&
            point.y as isize <= end_y
    }
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
        self.length = self.length + grow_by;
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
pub enum InputStateUpdate {
    Tick,
    ChangeDirection(usize, Direction),
    DropSnake(usize),
}

#[derive(Debug, Clone)]
pub struct OutputStateUpdate {
    snakes: HashMap<usize, Snake>,
    foods: HashSet<BoxShape>,
}

impl OutputStateUpdate {
    pub fn get_snakes(&self) -> &HashMap<usize, Snake> {
        &self.snakes
    }

    pub fn get_foods(&self) -> &HashSet<BoxShape> {
        &self.foods
    }
}



#[derive(Debug)]
pub struct GameState {
    snakes: HashMap<usize, Snake>,
    id_gen: AtomicUsize,
    food_set: HashSet<BoxShape>,
    food_count: usize,
    x_size: usize,
    y_size: usize,
    food_size: usize
}

impl GameState {
    pub fn new() -> GameState {
        let snakes: HashMap<usize, Snake> = HashMap::new();
        let food_set: HashSet<BoxShape> = HashSet::new();
        let id_gen = AtomicUsize::new(0);

        GameState { snakes, id_gen, food_set, food_count: 3, x_size: 768, y_size: 512, food_size:5 }
    }

    fn tick(&mut self) -> () {
        let mut snake_point_set: HashMap<&Point, usize> = HashMap::new();
        let mut dead_snakes: Vec<usize> = vec![];
        for (snake_id, snake) in self.snakes.iter_mut() {
            snake.tick();
            let mut first = true;
            for pt in snake.points.iter() {
                if let Some(collide_id) = snake_point_set.insert(pt, *snake_id) {
                    if first { //if our head hit another we are dead
                        dead_snakes.push(snake_id.clone());
                    } else { //the other snake must have hit us
                        dead_snakes.push(collide_id)
                    }
                    first = false;
                }
            }
        }

        self.snakes.retain(|id, _| !dead_snakes.contains(id));

        for (snake_id, snake) in self.snakes.iter_mut() {
            // check if this snake has hit another
            if let Some(head_point) = snake.points.get(0) {
                //check if we ate food
                //snake.grow(10);
                let mut ate_food = false;
                self.food_set.retain(|&f| {
                    let eaten = f.intersects(head_point);
                    if eaten {
                        println!("Snake ate: {}, {:?} {:?}", eaten, head_point, f);
                        ate_food = true;
                    }
                    !eaten
                });

                if ate_food {
                    snake.grow(10);
                }

            }
            snake.is_alive = true;

        }

        self.generate_food();
        //TODO detect collisions
    }

    fn generate_food(&mut self) {
        if self.food_set.len() >= self.food_count {
            return;
        }
        let to_create = self.food_count - self.food_set.len();
        let mut rng = rand::thread_rng();
        for _ in 0..to_create {
            let x =  rng.gen_range(0, 400);//self.x_size - self.food_size);
            let y =  rng.gen_range(0, 400);//self.y_size - self.food_size);

            self.food_set.insert(BoxShape::new(Point{x, y}, self.food_size as isize));
        }
    }

    pub fn get_snakes_ref(&self) -> &HashMap<usize, Snake> {
        &self.snakes
    }

    pub fn get_snakes(&self) -> HashMap<usize, Snake> {
        self.snakes.clone()
    }

    pub fn get_foods(&self) -> HashSet<BoxShape> {
        self.food_set.clone()
    }

    pub fn get_state(&self) -> OutputStateUpdate {
        OutputStateUpdate{
            snakes: self.get_snakes(),
            foods: self.get_foods()
        }
    }

    pub fn handle(&mut self, update: InputStateUpdate) {
        log::debug!("Handling {:?}", update);
        match update {
            InputStateUpdate::Tick => {
                self.tick();
            }
            InputStateUpdate::ChangeDirection(id, direction) => {
                if let Some(snake) = self.snakes.get_mut(&id) {
                    snake.set_direction(direction);
                } else {
                    log::warn!("Missing id {}", id);
                }
            }
            InputStateUpdate::DropSnake(id) => {
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
