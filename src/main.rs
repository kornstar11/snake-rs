extern crate log;
extern crate simple_logger;
extern crate snake_rs;
extern crate tokio;
#[macro_use]
extern crate crossbeam_channel;
use crossbeam_channel::{bounded, Sender};
use futures::future::Future;
use futures::stream::poll_fn;
use futures::stream::Stream;
use futures::{Async, Poll};
use snake_rs::game::{GameState, Point, Snake, StateUpdate};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::timer::Interval;
use warp::filters::path::full;
use warp::{self, path, Filter};

fn main() {
    simple_logger::init().unwrap();

    let game_state_arc: Arc<Mutex<GameState>> = Arc::new(Mutex::new(GameState::new()));
    let (tick_tx, tick_rx) = bounded(1);

    start_ticking(&game_state_arc, tick_tx);

    let assets = warp::fs::dir("assets");
    let index = warp::path::end().and(assets.clone());

    let v1_route = path("v1");

    let connect_state = game_state_arc.clone(); //needing to clone this for every closure seem waseful, can i just use static?
    let connect_route =
        v1_route
            .and(path!("connect"))
            .and(warp::ws2())
            .map(move |ws: warp::ws::Ws2| {
                let state = connect_state.clone();
                // And then our closure will be called when it completes...
                ws.on_upgrade(move |websocket| {
                    let state = state.clone();
                    //let websocket_stream: Stream<Item = warp::filters::ws::Message> = websocket;
                    // Just echo all messages back...
                    let (tx, rx) = websocket.split();
                    rx.and_then(move |input| {
                        let state = state.clone();
                        println!("Got input");
                        Ok(())
                    });
                    futures::future::ok(())
                    //                rx.forward(tx).map(|_| ()).map_err(|e| {
                    //                    eprintln!("websocket error: {:?}", e);
                    //                })
                })
            });
    let debug = v1_route.and(path!("debug")).map(move || {
        let state = game_state_arc.clone();
        let state = state.lock().unwrap();
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

fn start_ticking(
    game_state_arc: &Arc<Mutex<GameState>>,
    tick_tx: Sender<Arc<HashMap<usize, Snake>>>,
) {
    let game_state_arc = game_state_arc.clone();
    tokio::run({
        let local_state = game_state_arc.clone();
        let ticker = Interval::new(Instant::now(), Duration::from_millis(500));
        ticker
            .map_err(|e| panic!("timer failed; err={:?}", e))
            .for_each(move |_| {
                let local_state = local_state.clone();
                let mut local_state = local_state.try_lock().unwrap(); //.handle(StateUpdate::Tick);
                local_state.handle(StateUpdate::Tick);
                tick_tx.try_send(Arc::new(local_state.get_snakes()));
                //ticker.clone().lock().unwrap().reset(tick_duration);
                Ok(())
            })
    });
}
