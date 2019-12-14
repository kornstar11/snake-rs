extern crate simple_logger;
extern crate log;
extern crate snake_rs;
extern crate tokio;
use warp::{self, path, Filter};
use std::sync::{Arc, Mutex};
use snake_rs::game::{Snake, Point};
use futures::stream::Stream;
use futures::future::Future;
//use futures::{FutureExt, StreamExt};

fn main() {
    simple_logger::init().unwrap();

    let game_state: Vec<Snake> = vec![];
    let game_state_arc: Arc<Mutex<Vec<Snake>>> = Arc::new(Mutex::new(game_state));

    let assets = warp::fs::dir("assets");
    let index = warp::path::end().and(assets.clone());

    let v1_route = path("v1");

    let connect_route = v1_route
        .and(path!("connect"))
        .and(warp::ws2())
        .map(|ws: warp::ws::Ws2| {
            // And then our closure will be called when it completes...
            ws.on_upgrade(|websocket| {
                //let websocket_stream: Stream<Item = warp::filters::ws::Message> = websocket;
                // Just echo all messages back...
                let (tx, rx) = websocket.split();//websocket_stream.split();
                rx.forward(tx).map(|_| ()).map_err(|e| {
                    eprintln!("websocket error: {:?}", e);
                })
//                rx.forward(tx).map(|result| {
//                    if let Err(e) = result {
//                        eprintln!("websocket error: {:?}", e);
//                    }
//                })
            })
        });

    let sum = v1_route
        .and(path!("sum" / u32 / u32)
            .map(move|a, b| {
                let a = Arc::clone(&game_state_arc);
                let mut l = a.lock().unwrap();
                l.push(Snake::new(1, Point::new(1,2)));
                std::format!("{:?}", l)
            }));

    let routes = warp::get2().and(connect_route.or(sum.or(index)));
    warp::serve(routes).run(([127, 0, 0, 1], 3030));

}
