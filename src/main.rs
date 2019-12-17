extern crate log;
extern crate simple_logger;
extern crate snake_rs;
extern crate tokio;
#[macro_use]
use futures::future::Future;
use futures::stream;
use futures::stream::poll_fn;
use futures::stream::Stream;
use futures::sync::mpsc::channel;
use futures::sync::mpsc::{Receiver, Sender};
//use stream::channel::{Sender, Receiver};
use futures::future::Err;
use futures::sink::Sink;
use futures::{Async, Poll};
use snake_rs::game::{Direction, GameState, Point, Snake, StateUpdate};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::timer::Interval;
use warp::filters::fs::dir;
use warp::filters::path::full;
use warp::filters::ws::{Message, WebSocket};
use warp::{self, path, Filter};

fn main() {
    simple_logger::init().unwrap();

    let game_state_arc: Arc<Mutex<GameState>> = Arc::new(Mutex::new(GameState::new()));
    let game_tick: Arc<Mutex<Vec<Sender<HashMap<usize, Snake>>>>> = Arc::new(Mutex::new(vec![]));

    start_ticking(&game_state_arc, game_tick.clone());

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
                let game_tick = game_tick.clone();
                // And then our closure will be called when it completes...
                ws.on_upgrade(move |websocket| {
                    let (ws_tx, rx) = websocket.split();
                    let (update_tx, update_rx) = channel(1);

                    let state_l = state.clone();
                    let mut state_l = state_l.lock().unwrap();
                    let snake_id = state_l.create_snake();

                    let game_tick = game_tick.clone();
                    let mut game_tick = game_tick.lock().unwrap();
                    //add our channel
                    game_tick.push(update_tx);

                    let update_rx = update_rx
                        .map_err(|()| -> warp::Error { unreachable!("whoa") })
                        .map(|state| {
                            let my_snake = state.get(&snake_id).expect("missing id");
                            let as_json = serde_json::to_string(my_snake).expect("json failed");
                            Message::text(as_json)
                        });
                    let ford = update_rx.forward(ws_tx);

                    let r_fut = rx.and_then(move |input: Message| {
                        let message_string = input.to_str().unwrap();
                        let direction: Direction = serde_json::from_str(message_string).unwrap();
                        log::debug!(
                            "Got input {} {:?} {:?}",
                            snake_id,
                            message_string,
                            direction
                        );
                        let state = state.clone();
                        let mut state = state.lock().unwrap();
                        state.handle(StateUpdate::ChangeDirection(snake_id, direction));
                        Ok(())
                    });
                    //r_fut.map(|()| ())
                    //    .map_err(|e| -> warp::Error { unreachable!("whoa") })
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

    let routes = warp::get2().and(connect_route.or(debug.or(index)));
    warp::serve(routes).run(([127, 0, 0, 1], 3030));
}

fn start_ticking(
    game_state_arc: &Arc<Mutex<GameState>>,
    to_send: Arc<Mutex<Vec<Sender<HashMap<usize, Snake>>>>>,
) {
    tokio::run({
        let local_state = game_state_arc.clone();
        let ticker = Interval::new(Instant::now(), Duration::from_millis(500));
        ticker
            .map_err(|e| panic!("timer failed; err={:?}", e))
            .for_each(move |_| {
                let local_state = local_state.clone();
                let mut local_state = local_state.try_lock().unwrap(); //.handle(StateUpdate::Tick);
                let to_send = to_send.clone();
                let mut to_send = to_send.try_lock().unwrap(); //.handle(StateUpdate::Tick);
                local_state.handle(StateUpdate::Tick);

                for send_to in to_send.iter_mut() {
                    send_to
                        .try_send(local_state.get_snakes())
                        .expect("to_send failed");
                }

                //tick_tx.try_send(Arc::new(local_state.get_snakes()));
                Ok(())
            })
    });
}
