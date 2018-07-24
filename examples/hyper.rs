#![feature(async_await, await_macro)]


use hyper::Client;
use hyper::rt::{self, Future};
use futures_playground::{Compat01, Compat03};
fn main() {
    let client = Client::new();
    let fut = client.get("http://httpbin.org/ip".parse().unwrap()).compat();

    let fut = async {
        let res = await!(fut).unwrap();
        println!("{}", res.status());
    };
    
}