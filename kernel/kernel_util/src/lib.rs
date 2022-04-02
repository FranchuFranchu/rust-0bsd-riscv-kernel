#![no_std]

use core::{
    alloc::Layout,
    mem::{size_of, MaybeUninit},
};

extern crate alloc;

#[macro_export]
macro_rules! dont_recurse {
    ($e:block) => {
        static C: ::core::sync::atomic::AtomicUsize = ::core::sync::atomic::AtomicUsize::new(0);
        C.fetch_add(1, ::core::sync::atomic::Ordering::Release);
        if C.load(::core::sync::atomic::Ordering::Acquire) == 1 {
            $e
        }
        C.fetch_sub(1, ::core::sync::atomic::Ordering::Release);
    };
    ($e:stmt) => {
        ::kernel_util::dont_recurse!({ $e })
    };
    ($e:expr) => {
        ::kernel_util::dont_recurse!({ $e })
    };
}

pub fn boxed_slice_with_alignment<T: Clone>(
    size: usize,
    align: usize,
    initialize: &T,
) -> alloc::boxed::Box<[T]> {
    unsafe {
        let ptr: *mut MaybeUninit<T> =
            alloc::alloc::alloc(Layout::from_size_align(size * size_of::<T>(), align).unwrap())
                as *mut MaybeUninit<T>;
        for i in 0..size {
            *ptr.add(i) = MaybeUninit::new(initialize.clone())
        }
        alloc::boxed::Box::from_raw(core::slice::from_raw_parts_mut(ptr as *mut T, size))
    }
}
pub fn boxed_slice_with_alignment_uninit<T>(
    size: usize,
    align: usize,
) -> alloc::boxed::Box<[MaybeUninit<T>]> {
    unsafe {
        let ptr: *mut MaybeUninit<T> =
            alloc::alloc::alloc(Layout::from_size_align(size * size_of::<T>(), align).unwrap())
                as *mut MaybeUninit<T>;
        alloc::boxed::Box::from_raw(core::slice::from_raw_parts_mut(ptr, size))
    }
}

pub fn struct_to_bytes<'a, T>(struc: &'a T) -> &'a [MaybeUninit<u8>] {
    unsafe {
        core::slice::from_raw_parts(struc as *const T as *const _, core::mem::size_of_val(struc))
    }
}

pub fn struct_to_bytes_mut<'a, T>(struc: &'a mut T) -> &'a mut [MaybeUninit<u8>] {
    unsafe {
        core::slice::from_raw_parts_mut(struc as *mut T as *mut _, core::mem::size_of_val(struc))
    }
}
