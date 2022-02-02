use alloc::{collections::BTreeSet, vec::Vec};

use kernel_cpu::read_time;

pub fn primes(up_to: usize) {
    let mut sieve = Vec::new();
    let mut not_removed = BTreeSet::new();

    // Initialize the sieve
    for i in 0..up_to {
        sieve.push(false);
        if i > 1 {
            not_removed.insert(i);
        }
    }

    // Fill the sieve
    for idx in 2..sieve.len() {
        // If already filled, continued
        if sieve[idx] {
            continue;
        }

        // Fill up all the multiples of "idx", starting with idx * 2
        for multiple in (idx * 2..up_to).step_by(idx) {
            sieve[multiple] = true;
        }
        for maybe_prime_idx in 2..idx {
            if !sieve[maybe_prime_idx] && not_removed.contains(&maybe_prime_idx) {
                not_removed.remove(&maybe_prime_idx);
                println!("Prime {:?}", maybe_prime_idx);
            }
        }
    }
}

pub fn time_fn<T, F: Fn() -> T>(f: F) -> (core::time::Duration, T) {
    let start_time = read_time();
    let ret: T = f();
    let end_time = read_time();

    let time = core::time::Duration::from_nanos(((end_time - start_time) * 100) as u64);

    println!("{:?} secs", (time.as_micros() as f64) / 1000000f64);

    (time, ret)
}
