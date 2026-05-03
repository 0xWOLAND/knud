use std::{pin::Pin, task::{Context, Poll, RawWaker, RawWakerVTable, Waker}};

// Implement the 2-state machine that Knuth described with Futures
#[derive(Debug)]
enum State {
    A(i32),
    B(i32),
    Done(i32),
}

struct StateFuture {
    state: State,
    steps_left: i32,
}

impl Future for StateFuture {
    type Output = i32;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        println!("poll: {:?}", self.state);

        if self.steps_left == 0 {
            let n = match self.state {
                State::A(n) | State::B(n) | State::Done(n) => n,
            };

            self.state = State::Done(n);
            return Poll::Ready(n);
        }

        self.state = match self.state {
            State::A(n) => State::B(n + 1),
            State::B(n) => State::A(n),
            State::Done(n) => State::Done(n),
        };

        self.steps_left -= 1;

        Poll::Pending
    }
}

impl StateFuture {
    fn new(start: i32, steps: i32) -> StateFuture {
        StateFuture {
            state: State::A(start),
            steps_left: steps,
        }
    }
}


fn dummy_waker() -> Waker {
    unsafe fn clone(_: *const ()) -> RawWaker { raw_waker() }
    unsafe fn noop(_: *const ()) {}

    fn raw_waker() -> RawWaker {
        RawWaker::new(
            std::ptr::null(),
            &RawWakerVTable::new(clone, noop, noop, noop),
        )
    }

    unsafe { Waker::from_raw(raw_waker()) }
}

fn main() {
    let mut fut = StateFuture::new(1, 10);
    let waker = dummy_waker();
    let mut cx = Context::from_waker(&waker);
    let mut fut = unsafe { Pin::new_unchecked(&mut fut) };

    loop {
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(n) => {
                println!("done: {}", n);
                break;
            }
            Poll::Pending => {}
        }
    }
}