#![feature(pin, async_await, await_macro)]

use hyper::Client;
use futures::compat::*;
use futures::stream::{StreamExt};
use futures::future::{TryFutureExt, FutureExt};

use pin_utils::pin_mut;

use tokio::runtime::Runtime;
use std::io::{self, Write};

fn main() {
    let client = Client::new();
    let fut = client.get("http://httpbin.org/ip".parse().unwrap()).compat();

    let fut = async {
        let res = await!(fut).unwrap();
        println!("{}", res.status());

        let body = res.into_body().compat();

        pin_mut!(body);
        while let Some(Ok(chunk)) = await!(body.next()) {
            let _ = io::stdout()
            .write_all(&chunk)
            .map_err(|e| {
                panic!("example expects stdout is open, error={}", e)
            });
        }
    };

    let mut rt = Runtime::new().unwrap();
    let fut = fut.unit_error()
                 .boxed()
                 .compat(rt.executor().compat());

    let _ = rt.block_on(fut);
    
}