#![feature(async_await, await_macro)]

use hyper::Client;
use hyper::rt::Stream;
use futures_playground::{Compat01, Compat03, ExecCompat};

use tokio::runtime::Runtime;
use std::io::{self, prelude::*};

fn main() {
    let client = Client::new();
    let fut = client.get("http://httpbin.org/ip".parse().unwrap()).compat();

    let fut = async {
        let res = await!(fut).unwrap();
        println!("{}", res.status());
        // println!("uncomment me!");

        await!(res.into_body().for_each(|chunk| {
            io::stdout().write_all(&chunk)
                .map_err(|e| panic!("example expects stdout is open, error={}", e))
        }).compat());
    };

    let mut rt = Runtime::new().unwrap();
    let fut = fut.compat(rt.executor().compat());

    let _ = rt.block_on(fut);   
}