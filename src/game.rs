use std::collections::VecDeque;

#[derive(Debug)]
pub struct Point {
    x: u32,
    y: u32
}

#[derive(Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right
}

#[derive(Debug)]
struct Snake {
    points: VecDeque<Point>,
    length: usize,
    direction: Direction
}

impl Snake {
    pub fn new(length: usize, starting_point: Point) -> Snake {
        let mut new_queue: VecDeque<Point> = VecDeque::new();
        new_queue.push_front(starting_point);
        Snake {
            points: new_queue,
            length: length,
            direction: Direction::Right
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
        self.direction = direction;
    }

    pub fn tick(&mut self) {
        if let Some(front) = self.points.front() {
            let new_point = match  self.direction {
                Direction::Up => {
                    Point{
                        x: front.x,
                        y: front.y + 1
                    }
                }
                Direction::Down => {
                    Point{
                        x: front.x,
                        y: front.y - 1
                    }
                }
                Direction::Right => {
                    Point{
                        x: front.x + 1,
                        y: front.y
                    }
                }
                Direction::Left => {
                    Point{
                        x: front.x - 1,
                        y: front.y
                    }
                }
            };
            self.add_point(new_point);
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::game::*;

    #[test]
    fn test_snake_growth() {
        let start_point = Point{
            x: 10,
            y: 10
        };

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
