extern crate simple_logger;
extern crate log;
extern crate snake_rs;
extern crate tokio;
use warp::{self, path, Filter};
use std::sync::{Arc, Mutex};
use snake_rs::game::{Snake, Point};

#[tokio::main]
async fn main() {
    simple_logger::init().unwrap();

    let game_state: Vec<Snake> = vec![];
    let game_state_arc: Arc<Mutex<Vec<Snake>>> = Arc::new(Mutex::new(game_state));

    let assets = warp::fs::dir("assets");
    let index = warp::path::end().and(assets.clone());

    let v1_route = path("v1");

    let sum = v1_route
        .and(path!("sum" / u32 / u32)
            .map(move|a, b| {
                let a = Arc::clone(&game_state_arc);
                let mut l = a.lock().unwrap();
                l.push(Snake::new(1, Point::new(1,2)));
                std::format!("{:?}", l)
            }));

    let routes = warp::get2().and(sum.or(index));
    warp::serve(routes).run(([127, 0, 0, 1], 3030));

}
