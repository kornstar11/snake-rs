extern crate log;
extern crate simple_logger;
extern crate snake_rs;
extern crate tokio;
use futures::future::Future;
use futures::stream::Stream;
use futures::sync::mpsc::channel;
use futures::sync::mpsc::Sender;
use snake_rs::game::{Direction, GameState, Snake, StateUpdate};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::timer::Interval;
use warp::filters::ws::Message;
use warp::{self, path, Filter};

fn main() {
    simple_logger::init().unwrap();

    let game_state_arc: Arc<Mutex<GameState>> = Arc::new(Mutex::new(GameState::new()));
    let game_tick: Arc<Mutex<Vec<Sender<HashMap<usize, Snake>>>>> = Arc::new(Mutex::new(vec![]));
    let ticker_fut = start_ticking(game_state_arc.clone(), game_tick.clone());


    let assets = warp::fs::dir("assets");
    let index = warp::path::end().and(assets.clone());

    let js = warp::path("js").and(warp::fs::dir("assets/js"));

    let v1_route = path("v1");

    let connect_state = game_state_arc.clone(); //needing to clone this for every closure seem waseful, can i just use static?
    let connect_route =
        v1_route
            .and(v1_route)
            .and(path!("connect"))
            .and(warp::ws2())
            .map(move |ws: warp::ws::Ws2| {
                let state = connect_state.clone();
                let game_tick = game_tick.clone();
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
                        .map(move |state| {
                            let my_snake = state.get(&snake_id).expect("missing id");
                            let as_json = serde_json::to_string(my_snake).expect("json failed");
                            Message::text(as_json)
                        });

                    let mapped_tx = update_rx.forward(ws_tx);

                    let rx_fut = rx.and_then(move |input: Message| {
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
                    let mapped_rx = rx_fut.map_err(|_| ()).for_each(|_| futures::future::ok(()));

                    let selected = futures::Future::select2(mapped_tx, mapped_rx)
                        .map_err(|_| ())
                        .map(|_| ());
                    selected
                })
            });
    let debug = v1_route.and(path!("debug")).map(move || {
        let state = game_state_arc.clone();
        let state = state.lock().unwrap();
        std::format!("Current state: {:?}", state)
    });

    let routes = warp::get2()
        .and(
            connect_route
                .or(debug)
                .or(index)
                .or(js)
        );
    let server_fut  = {
        let srv = warp::serve(routes);
        let (addr, fut) = srv.bind_ephemeral(([127, 0, 0, 1], 3031));
        log::info!("Listening on {:?}", addr);
        fut
    };//.run(([127, 0, 0, 1], 3031));

    let joined = server_fut.join(ticker_fut).map(|_| ());
    tokio::run(joined);

}

fn start_ticking(
    game_state_arc: Arc<Mutex<GameState>>,
    to_send: Arc<Mutex<Vec<Sender<HashMap<usize, Snake>>>>>,
) -> impl Future<Item = (), Error = ()> {
    let local_state = game_state_arc.clone();
    let ticker = Interval::new(Instant::now(), Duration::from_millis(500));
    ticker
        //.map_err(|e| panic!("timer failed; err={:?}", e))
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
        .map_err(|_| ())
}
