use core::ops::{BitAnd, BitXor};

use alloc::{boxed::Box, collections::BTreeSet, vec::Vec};

use crate::cpu;

// random-ish function I just made up
fn twist(value: &mut usize) -> usize {
	*value = value.wrapping_add(0x902392093222).bitxor(0b10101110101).bitand(0xFF);
	return *value;
}

pub fn test_task() {
	println!("SP 0x{:X}", cpu::read_sp());
	info!("Doing lots of math");
	
	
	let mut twisted_value = 0;
	
	let mut vector_vec = Vec::new();
	
	
	println!("{:?}", unsafe { &(*cpu::read_sscratch()).pid as *const usize });
	println!("{:?}", unsafe { (*cpu::read_sscratch()).pid });
	
	
	
	info!("Hii");
	for i in 0..1 {	
		println!("twisted {:?}", twist(&mut twisted_value));
		let mut v: Vec<usize> = Vec::with_capacity(twisted_value);
		v.resize(twisted_value, 0);
		for i in v.iter_mut() {
			*i = i as *mut usize as usize;
		}
		println!("{:?}", unsafe { &(*cpu::read_sscratch()).pid as *const usize });
		println!("{:?}", unsafe { (*cpu::read_sscratch()).pid });
		vector_vec.push(v);
		loop {};
		println!("{:?}", unsafe { (*cpu::read_sscratch()).pid });
	};
	println!("{:?}", unsafe { (*cpu::read_sscratch()).pid });
	vector_vec.remove(10);
	vector_vec.remove(20);
	vector_vec.remove(30);
	info!("Creation done");
	for v in vector_vec.iter() {
		for i in v.iter() {
			assert!(*i == i as *const usize as usize);
			
		}
	}
	
	info!("finished");
	
	
	// Allocating tons of memory
	
	println!("{:?}", unsafe { (*cpu::read_sscratch()).pid });
	// Calculate primes
	let mut sieve = Vec::new();
	let mut not_removed = BTreeSet::new();
	for i in 1..100 {
		sieve.push(false);
		not_removed.insert(i);
	}
	for idx in 0..sieve.len() {
		println!("{:?}", unsafe { (*cpu::read_sscratch()).pid });
		if sieve[idx] == true {
			continue;
		}
		let mut jdx = idx * 2;
		while jdx < sieve.len() {
			sieve[jdx] = true;
			not_removed.remove(&jdx);
			jdx += idx;
		}
	}
	
	
	info!("{:?}","Task 1 reached");
}

pub fn test_task_2() {
	info!("{:?}","Task 2 reached");
}