use core::ops::{BitAnd, BitXor};

use alloc::{collections::BTreeSet, vec::Vec};

use crate::cpu;

// random-ish function I just made up
fn twist(value: &mut usize) -> usize {
	*value = value.wrapping_add(0x902392093222).bitxor(0b10101110101).bitand(0xFF);
	return *value;
}

pub fn test_task() {
	
	info!("{:?}","Calculating primes...");
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
	info!("Finished");
	
	
}

pub fn test_task_2() {
	info!("{:?}","Allocating a whole lot of memory...");
	
	
	
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
	info!("Finished");
	drop(vector_vec);
	info!("Finished dropping");
}