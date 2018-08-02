#![feature(async_await, await_macro)]

use hyper::Client;
use hyper::rt::Stream;
use futures::compat::*;
use futures::future::{TryFutureExt, FutureExt};

use tokio::runtime::Runtime;
use std::io::{self, Write};

fn main() {
    let client = Client::new();
    let fut = client.get("http://httpbin.org/ip".parse().unwrap()).compat();

    let fut = async {
        let res = await!(fut).unwrap();
        println!("{}", res.status());

        let body = res.into_body().concat2().compat();
        let body = await!(body).unwrap();
        
        io::stdout()
            .write_all(&body)
            .map_err(|e| {
                panic!("example expects stdout is open, error={}", e)
            })
    };

    let mut rt = Runtime::new().unwrap();
    let fut = fut.unit_error()
                 .boxed()
                 .compat(rt.executor().compat());

    let _ = rt.block_on(fut);
    
}