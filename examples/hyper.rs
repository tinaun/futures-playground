#![feature(async_await, await_macro)]


use hyper::Client;
use hyper::rt::Future;
use futures_playground::{Compat01, Compat03, ExecCompat};

use tokio::runtime::Runtime;

fn main() {
    let client = Client::new();
    let fut = client.get("http://httpbin.org/ip".parse().unwrap()).compat();

    let fut = async {
        let res = await!(fut).unwrap();
        println!("{}", res.status());
    };

    let mut rt = Runtime::new().unwrap();
    let fut = fut.compat(rt.executor().compat());

    rt.block_on(fut);
    
}