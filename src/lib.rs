#![feature(futures_api, pin, arbitrary_self_types, use_extern_macros)]

use futures::Future as Future01;
use futures::Poll as Poll01;
use futures::task as task01;
use futures::task::Task as Task01;
use futures::executor::{with_notify, NotifyHandle, Notify, UnsafeNotify};


use std::sync::Arc;

use futures_core::Future as Future03;
use futures_core::Poll as Poll03;
use futures_core::task;
use futures_core::task::Executor as Executor03;
use futures_core::task::{Wake, Waker, LocalWaker, local_waker_from_nonlocal};

use std::mem::PinMut;
use std::marker::Unpin;

pub trait Compat01: Future01 {
    fn compat(self) -> Compat<Self, ()> where Self: Sized {
        Compat {
            inner: self,
            exec: None,
        }
    }
}

impl<T: Future01> Compat01 for T {}

pub trait Compat03: Future03 {
    fn compat<E>(self, exec: E) -> Compat<Self, E> 
        where Self: Sized,
              E: Executor03, 
    {
        Compat {
            inner: self,
            exec: Some(exec),
        }
    }
}

impl<T: Future03> Compat03 for T {}

pub struct Compat<F, E> {
    inner: F,
    exec: Option<E>,
}

impl<T> Future03 for Compat<T, ()> where T: Future01 {
    type Output = Result<T::Item, T::Error>;

    fn poll(self: PinMut<Self>, cx: &mut task::Context) -> Poll03<Self::Output> {
        use futures::Async;

        let notify = &WakerToHandle(cx.waker());

        with_notify(notify, 0, move || { unsafe {
                match PinMut::get_mut_unchecked(self).inner.poll() {
                Ok(Async::Ready(t)) => Poll03::Ready(Ok(t)),
                Ok(Async::NotReady) => Poll03::Pending,
                Err(e) => Poll03::Ready(Err(e)),
            }
        }})
    }
}

struct NotifyWaker(Waker);

#[derive(Clone)]
struct WakerToHandle<'a>(&'a Waker);

#[doc(hidden)]
impl<'a> From<WakerToHandle<'a>> for NotifyHandle {
    fn from(handle: WakerToHandle<'a>) -> NotifyHandle {
        let ptr = Box::new(NotifyWaker(handle.0.clone()));

        unsafe {
            NotifyHandle::new(Box::into_raw(ptr))
        }
    }
}

impl Notify for NotifyWaker {
    fn notify(&self, _: usize) {
        self.0.wake();
    }
}

unsafe impl UnsafeNotify for NotifyWaker {
    unsafe fn clone_raw(&self) -> NotifyHandle {
        WakerToHandle(&self.0).into()
    }

    unsafe fn drop_raw(&self) {
        let ptr: *const UnsafeNotify = self;
        drop(Box::from_raw(ptr as *mut UnsafeNotify));
    }
}




impl<T, E> Future01 for Compat<T, E> where T: Future03 + Unpin,
    E: Executor03
{
    type Item = T::Output;
    type Error = ();

    fn poll(&mut self) -> Poll01<Self::Item, Self::Error> {
        use futures::Async;

        let waker = current_as_waker();
        let mut cx = task::Context::new(&waker, self.exec.as_mut().unwrap());
        
        match PinMut::new(&mut self.inner).poll(&mut cx) {
            Poll03::Ready(t) => Ok(Async::Ready(t)),
            Poll03::Pending  => Ok(Async::NotReady),
        }
    }
}

fn current_as_waker() -> LocalWaker {
    let arc_waker = Arc::new(Current(task01::current()));
    local_waker_from_nonlocal(arc_waker)
}

struct Current(Task01);

impl Wake for Current {
    fn wake(arc_self: &Arc<Self>) {
        arc_self.0.notify();
    }
}


