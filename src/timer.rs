use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// A simple recurring background timer.
///
/// The timer executes the provided `task` closure, then waits approximately
/// `interval_ms` milliseconds before repeating. The returned `TimerHandle`
/// owns the background thread; consuming the handle with `stop()` signals the
/// thread to exit and blocks until it joins. Dropping the handle will also
/// attempt a graceful stop and join the thread.
pub struct TimerHandle {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl TimerHandle {
    /// Stop the timer and join the background thread. This consumes the
    /// handle and blocks until the background thread exits.
    pub fn stop(mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }

    /// Internal constructor used by `start_timer`.
    fn new(stop: Arc<AtomicBool>, thread: JoinHandle<()>) -> Self {
        Self { stop, thread: Some(thread) }
    }
}

/// Spawn a background timer which calls `task` every `interval_ms` milliseconds.
///
/// The `task` closure must be `Send + 'static` because it runs on a worker
/// thread. To keep `stop()` responsive the implementation sleeps in small
/// chunks (10ms) between invocations and checks the stop flag between chunks.
pub fn start_timer(interval_ms: u64, mut task: impl FnMut() + Send + 'static) -> TimerHandle {
    let stop = Arc::new(AtomicBool::new(false));
    let stop_clone = stop.clone();

    // Normalize zero to a sane minimum interval (1ms) to prevent tight spins.
    let interval = if interval_ms == 0 { 1 } else { interval_ms };
    const CHUNK_MS: u64 = 10;

    let thread = thread::spawn(move || {
        while !stop_clone.load(Ordering::SeqCst) {
            // Execute task first; this preserves the previous behaviour and
            // makes the timer 'fire immediately' on start.
            task();

            // Sleep in small increments so a stop signal is noticed quickly.
            let mut slept = 0u64;
            while slept < interval && !stop_clone.load(Ordering::SeqCst) {
                let to_sleep = std::cmp::min(CHUNK_MS, interval - slept);
                thread::sleep(Duration::from_millis(to_sleep));
                slept += to_sleep;
            }
        }
    });

    TimerHandle::new(stop, thread)
}

impl Drop for TimerHandle {
    fn drop(&mut self) {
        // Ensure the worker thread is signalled to stop and joined when the
        // handle is dropped. We ignore panics in the worker thread when
        // joining; there's little we can do from Drop.
        self.stop.store(true, Ordering::SeqCst);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}
