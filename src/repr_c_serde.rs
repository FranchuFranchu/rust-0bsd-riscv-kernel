//! Serialization/Deserialization for repr C Copy structs
//! Don't think it's fully safe, use at your own risk!
//! (but, it's meant to be safe)

use core::ops::{Deref};
use core::marker::{Copy};
use core::slice;

pub struct ReprCDeserializer<'a, T: Copy + Sized> {
	data: &'a T,
}

impl <'a, T: Copy> ReprCDeserializer<'a, T>  {
    pub fn new(data: &'a [u8]) -> Self {
    	unsafe {
	    	Self {
	    		data: (data.as_ptr() as *const T).as_ref().unwrap()
	    	}
	    }
    }
}

impl<'a, T: Copy> Deref for ReprCDeserializer<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.data
    }
	// add code here
}

pub struct ReprCSerializer<'a> {
	data: &'a [u8],
}

impl <'a> ReprCSerializer<'a>  {
    pub fn new<T: Copy + Sized>(data: &'a T) -> Self {
    	unsafe {
	    	Self {
	    		data: slice::from_raw_parts(data as *const T as *const u8, core::mem::size_of_val(data))
	    	}
	    }
    }
}
