#![feature(arbitrary_self_types, futures_api, async_await, await_macro, never_type, generators, pin)]

use nb::{block, await as nb_await};
use futures::executor::block_on;
use futures::{join, unsafe_unpinned, Future};
use futures::task::{Context, Poll};

use std::time::{Instant, Duration};
use std::mem::PinMut;

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

fn _toggle_async() {
    let a = async {
        let mut a = SlowLed::new();
        nb_await!(a.toggle()) 
        // nb_await is not executor aware - it never calls the waker,
        // and thus, never is called again after yielding - running this function calls each 
        // sub future once and then never again.
    };

    let b = async {
        let mut b = SlowLed::new();
        nb_await!(b.toggle())
    };

    block_on(async {
        join!(a, b);
    });
}

// you could write a custom executor that calls all of its futures round robin without using a waker,
// but i decided to write a wrapper type for the built in executor instead. - i imagine that the former
// approach would make more sense on an embedded system, but i don't have much experience there.

/// a future that wraps a nonblocking function
struct NbFuture<F, T, E> 
    where F: FnMut() -> nb::Result<T, E>
{
    f: F,
}

impl<F, T, E> NbFuture<F, T, E> 
    where F: FnMut() -> nb::Result<T, E>
{
    unsafe_unpinned!(f: F); 
    //convenence macro for getting a mutable reference to a field out of a pinned struct
}

impl<F, T, E> Future for NbFuture<F, T, E> 
    where F: FnMut() -> nb::Result<T, E>
{
    type Output = Result<T, E>;
    fn poll(mut self: PinMut<Self>, ctx: &mut Context) -> Poll<Result<T, E>> {
        match (self.f())() {
            Ok(t) => Poll::Ready(Ok(t)),
            Err(nb::Error::WouldBlock) => {
                ctx.waker().wake();
                Poll::Pending
            },
            Err(nb::Error::Other(e)) => Poll::Ready(Err(e))
        }
    }
}

fn would_block<F, T, E>(f: F) -> NbFuture<F, T, E> 
    where F: FnMut() -> nb::Result<T, E>
{
    NbFuture {
        f
    }
}

fn toggle_async_two() {
    let mut a = SlowLed::new();
    let a = would_block(move || a.toggle());

    let mut b = SlowLed::new();
    let b = would_block(move || b.toggle());

    block_on(async {
        join!(a, b);
    });
}

fn main() {
    println!("{:?}", time_it(toggle_sync));
    println!("{:?}", time_it(toggle_async_two));
}