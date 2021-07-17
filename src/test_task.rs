use core::ops::{BitAnd, BitXor};
use core::task::Context;
use core::pin::Pin;

use alloc::{collections::BTreeSet, vec::Vec, boxed::Box};

use crate::{cpu, process};

// random-ish function I just made up
fn twist(value: &mut usize) -> usize {
	*value = value.wrapping_add(0x902392093222).bitxor(0b10101110101).bitand(0xFF);
	return *value;
}

pub fn test_task() {
	
	// Calculate primes
	let mut sieve = Vec::new();
	let mut not_removed = BTreeSet::new();
	for i in 0..500 {
		sieve.push(false);
		if i > 1 {
			not_removed.insert(i);
		}
	}
	for idx in 2..sieve.len() {
		if sieve[idx] == true {
			continue;
		}
		let mut jdx = idx * 2;
		while jdx < 500 {
			sieve[jdx] = true;
			jdx += idx;
		}
		for maybe_prime_idx in 2..idx {
			if sieve[maybe_prime_idx] == false && not_removed.contains(&maybe_prime_idx) {
				println!("Prime: {}", maybe_prime_idx);
				not_removed.remove(&maybe_prime_idx);
			}
		}
	}
	
	
}

pub fn test_task_2() {
	
	
	
	// Allocating tons of memory
	let mut twisted_value = 0;
	let mut vector_vec = Vec::with_capacity(10);
	for i in 0..70 {	
		let mut v: Vec<usize> = Vec::with_capacity(twisted_value);
		v.resize(twisted_value, 0);
		for i in v.iter_mut() {
			*i = i as *mut usize as usize;
		}
		vector_vec.push(v);
	};
	for v in vector_vec.iter() {
		for i in v.iter() {
			assert!(*i == i as *const usize as usize);
		}
	}
	drop(vector_vec);
	
	use crate::timeout::TimeoutFuture;
	// On QEMU, 10_000_000 timebaser is 1 second
	let mut future = TimeoutFuture { for_time: cpu::get_time() + 10_000_000 };
	let waker = process::Process::this().write().construct_waker();
	use core::future::Future;
	
	info!("Scheduling timeout..");
	
	// Poll the future until it resolves
	while TimeoutFuture::poll(Pin::new(&mut future), &mut Context::from_waker(&waker)) == core::task::Poll::Pending {
		// Trigger a "yield" smode-to-smode syscall
		trigger_yield_syscall();
	}
	
	info!("Timeout finished");
}

#[inline]
fn trigger_yield_syscall() {
	unsafe {
		llvm_asm!(r"
			li a7, 2
			# Trigger a timer interrupt
			csrr t0, sip
			# Set SSIP
			ori t0, t0, 2
			csrw sip, t0
		"::: "a7", "t0")
	}
}