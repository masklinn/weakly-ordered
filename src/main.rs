use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

// uncomment for incorrect (relaxed)
const ACK: Ordering = Ordering::Relaxed;
const REL: Ordering = Ordering::Relaxed;
// uncomment for correct (acquire/release)
//const ACK: Ordering = Ordering::Acquire;
//const REL: Ordering = Ordering::Release;

static FLAG: AtomicBool = AtomicBool::new(false);
static mut SHARED_VALUE: u32 = 0;

#[inline(never)]
fn do_busy_work(v: *mut i32) {
    loop {
        let i: i32 = rand::random();
        unsafe {
            std::ptr::write_volatile(v, i);
        }
        if (i & 7) == 0 {
            return;
        }
    }
}

const INCREMENTS: usize = 10_000_000;
#[inline(never)]
fn increment_shared_value() {
    let mut count = 0;
    while count < INCREMENTS {
        let mut v = 0;
        do_busy_work(&mut v as _);

        if FLAG
            .compare_exchange(false, true, ACK, Ordering::Relaxed)
            .is_ok()
        {
            // increment
            unsafe {
                SHARED_VALUE += 1;
            }
            // store
            FLAG.store(false, REL);
            // counter
            count += 1;
        }
    }
}

pub fn main() {
    let threads_count = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "2".into())
        .parse()
        .expect("the first argument to be a thread count");

    let mut threads = Vec::with_capacity(threads_count);

    loop {
        unsafe {
            SHARED_VALUE = 0;
        }

        for _ in 0..threads_count {
            threads.push(thread::spawn(increment_shared_value));
        }

        for t in threads.drain(..) {
            t.join().unwrap();
        }
        println!("shared value = {}", unsafe { SHARED_VALUE });
    }
}
