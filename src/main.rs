#![feature(futures_api, async_await, await_macro, never_type, generators, pin)]

use nb::{block, await as nb_await};
use futures::executor::block_on;
use futures::join;

use std::time::{Instant, Duration};

/// a dummy led that blocks for 1s before
/// it changes state
struct SlowLed {
    start_toggle: Option<Instant>,
    state: bool,
}

impl SlowLed {
    fn new() -> Self {
        SlowLed {
            start_toggle: None,
            state: false,
        }
    }

    fn toggle(&mut self) -> nb::Result<(), !> {
        if self.start_toggle.is_none() {
            self.start_toggle = Some(Instant::now());
            Err(nb::Error::WouldBlock)
        } else {
            let elapsed = self.start_toggle.unwrap().elapsed();
            if elapsed < Duration::from_millis(1000) {
                Err(nb::Error::WouldBlock)
            } else {
                self.state = !self.state;
                Ok(())
            }
        }
    }
}

fn time_it<F: FnOnce()>(f: F) -> Duration {
    let start = Instant::now();
    f();
    start.elapsed()
}

fn toggle_sync() {
    let mut a = SlowLed::new();
    let mut b = SlowLed::new();
    block!(a.toggle());
    block!(b.toggle());
}

fn toggle_async() {
    let a = async {
        let mut a = SlowLed::new();
        nb_await!(a.toggle()) 
        // nb_await is not executor aware - it never calls the waker,
        // and thus, never is called again after yielding
    };

    let b = async {
        let mut b = SlowLed::new();
        nb_await!(b.toggle())
    };

    block_on(async {
        join!(a, b);
    });
}

fn main() {
    println!("{:?}", time_it(toggle_sync));
    println!("{:?}", time_it(toggle_async));
}