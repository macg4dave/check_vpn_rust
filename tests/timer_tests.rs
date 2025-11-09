use std::time::Duration;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

/// Ensure the timer invokes the provided callback repeatedly.
#[test]
fn timer_invokes_callback_repeatedly() {
    let counter = Arc::new(AtomicUsize::new(0));
    let c = counter.clone();

    // Start timer with 20ms interval.
    let handle = check_vpn::timer::start_timer(20, move || {
        c.fetch_add(1, Ordering::SeqCst);
    });

    // Wait ~120ms so we expect ~6 invocations (allow margin).
    std::thread::sleep(Duration::from_millis(120));

    // Stop the timer and join
    handle.stop();

    let val = counter.load(Ordering::SeqCst);
    assert!(val >= 4, "expected >=4 ticks, got {}", val);
}

/// Timer should stop when handle.stop() is called.
#[test]
fn timer_stop_stops_invocations() {
    let counter = Arc::new(AtomicUsize::new(0));
    let c = counter.clone();

    let handle = check_vpn::timer::start_timer(30, move || {
        c.fetch_add(1, Ordering::SeqCst);
    });

    std::thread::sleep(Duration::from_millis(80));
    // stop and record value
    handle.stop();
    let val1 = counter.load(Ordering::SeqCst);

    // Wait a bit to ensure no further increments happen
    std::thread::sleep(Duration::from_millis(80));
    let val2 = counter.load(Ordering::SeqCst);

    assert_eq!(val1, val2, "counter should not increase after stop");
}
