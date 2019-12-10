extern crate hyper;
extern crate simple_logger;
extern crate log;
extern crate snake_rs;

use hyper::{Body, Request, Response, Server};
use hyper::service::service_fn_ok;
use hyper::rt::{self, Future};
use std::sync::{Arc, Mutex};
use snake_rs::game::{Snake, Point};

fn main() {
    simple_logger::init().unwrap();
    let addr = ([127, 0, 0, 1], 3000).into();
    let my_v:Arc<Mutex<Vec<Snake>>>= Arc::new(Mutex::new(vec![]));

    let server = Server::bind(&addr)
        .serve( move || {
            let aa = Arc::clone(&my_v);
            // This is the `Service` that will handle the connection.
            // `service_fn_ok` is a helper to convert a function that
            // returns a Response into a `Service`.
            service_fn_ok(move |_: Request<Body>| {
                let a =  Arc::clone(&aa);
                let mut m = a.lock().unwrap();
                let new_snake = Snake::new(3, Point::new(10,10));
                m.push(new_snake);
                for snake in m.iter_mut() {
                    snake.tick();
                }
                Response::new(Body::from(format!("vec {:?}", m)))
            })
        })
        .map_err(|e| eprintln!("server error: {}", e));

    println!("Listening on http://{}", addr);

    rt::run(server);
}
