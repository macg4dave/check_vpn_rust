use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// A simple cross-platform recurring timer.
///
/// `start_timer` spawns a background thread that executes `task` every `interval_ms` milliseconds
/// until the returned `TimerHandle` is stopped (consumes the handle).
pub struct TimerHandle {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl TimerHandle {
    /// Stop the timer and join the background thread.
    /// Consumes the handle.
    pub fn stop(mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

/// Spawn a background timer which calls `task` every `interval_ms` milliseconds.
///
/// The task must be `Send + 'static` so it can run in the thread. Returns a `TimerHandle`
/// which can be used to stop the timer.
pub fn start_timer(interval_ms: u64, mut task: impl FnMut() + Send + 'static) -> TimerHandle {
    let stop = Arc::new(AtomicBool::new(false));
    let s = stop.clone();

    // Ensure min interval of 1ms to avoid zero-sleep busy-loop.
    let interval = if interval_ms == 0 { 1 } else { interval_ms };

    let handle = thread::spawn(move || {
        while !s.load(Ordering::SeqCst) {
            task();

            // Sleep in small chunks so stop() is responsive.
            let mut slept = 0u64;
            let chunk = 10u64; // ms
            while slept < interval && !s.load(Ordering::SeqCst) {
                let to_sleep = std::cmp::min(chunk, interval - slept);
                thread::sleep(Duration::from_millis(to_sleep));
                slept += to_sleep;
            }
        }
    });

    TimerHandle { stop, thread: Some(handle) }
}

impl Drop for TimerHandle {
    fn drop(&mut self) {
        // If the handle is dropped without explicit stop, signal the thread to exit and detach.
        self.stop.store(true, Ordering::SeqCst);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}
