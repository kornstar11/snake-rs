extern crate simple_logger;
extern crate log;
extern crate snake_rs;
extern crate tokio;
#[macro_use]
extern crate crossbeam_channel;
use warp::{self, path, Filter};
use std::sync::{Arc, Mutex};
use snake_rs::game::{Point, GameState, StateUpdate};
use futures::stream::Stream;
use futures::future::Future;
use futures::stream::poll_fn;
use futures::{Async, Poll};
use std::ops::Deref;
use std::time::{Instant, Duration};
use tokio::timer::Interval;
use crossbeam_channel::{bounded, Sender};

//use futures::{FutureExt, StreamExt};

fn main() {
    simple_logger::init().unwrap();

    let game_state_arc: Arc<Mutex<GameState>> = Arc::new(Mutex::new(GameState::new()));
    let tick_duration = Instant::now() + Duration::from_millis(100);
    let (tick_tx, tick_rx) = bounded(1);
    tokio::run({
        let local_state = game_state_arc.clone();
        let ticker = Interval::new(Instant::now(), Duration::from_millis(500));
        ticker
            .map_err(|e| panic!("timer failed; err={:?}", e))
            .for_each(move |_| {
                local_state.clone().try_lock().unwrap().handle(StateUpdate::Tick);
                tick_tx.try_send(0);
                //ticker.clone().lock().unwrap().reset(tick_duration);
                Ok(())
            })
    });

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
    let debug = v1_route.and(path!("debug")).map(move || {
        let state_arc = game_state_arc
            .clone();
         let state = state_arc
            .lock()
            .unwrap();
        std::format!("Current state: {:?}", state)
    });

//    let sum = v1_route
//        .and(path!("sum" / u32 / u32)
//            .map(move|a, b| {
////                let a = Arc::clone(&game_state_arc);
////                let mut l = a.lock().unwrap();
////                l.push(Snake::new(1, Point::new(1,2)));
////                std::format!("{:?}", l)
//            }));

    let routes = warp::get2().and(connect_route.or(debug.or(index)));
    warp::serve(routes).run(([127, 0, 0, 1], 3030));

}
