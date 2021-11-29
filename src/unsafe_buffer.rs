use core::marker::PhantomData;

/// Un-lifetimed slice that is unsafe to create, but safe to use

#[derive(Debug)]
pub struct UnsafeSlice<T> {
    address: *const T,
    length: usize,
}

impl<T> UnsafeSlice<T> {
	/// # Safety
	/// `slice` must be still alive when the struct is dropped
	pub unsafe fn new(slice: &[T]) -> Self {
		Self {
			address: slice.as_ptr(),
			length: slice.len(),
		}
	}
	
	pub fn get(&self) -> &'static [T] {
		unsafe { core::slice::from_raw_parts(self.address, self.length) }
	}
	
	pub fn address(&self) -> *const T {
		self.address
	}
	
	pub fn length(&self) -> usize {
		self.length
	}
	
	pub unsafe fn copy(&self) -> Self {
		Self {
			address: self.address,
			length: self.length,
		}
	}
	pub unsafe fn upcast(self) -> UnsafeSliceMut<T> {
		UnsafeSliceMut {
			address: self.address as *mut T,
			length: self.length,
		}
	}
}

#[derive(Debug)]
pub struct UnsafeSliceMut<T> {
    address: *mut T,
    length: usize,
}


impl<T> UnsafeSliceMut<T> {
	/// # Safety
	/// `slice` must be still alive when the struct is dropped
	pub unsafe fn new(slice: &mut [T]) -> Self {
		Self {
			address: slice.as_mut_ptr(),
			length: slice.len(),
		}
	}
	
	pub fn get(self) -> &'static mut [T] {
		unsafe { core::slice::from_raw_parts_mut(self.address, self.length) }
	}
	
	pub fn address(&self) -> *mut T {
		self.address
	}
	
	pub fn length(&self) -> usize {
		self.length
	}
	pub unsafe fn downgrade(self) -> UnsafeSlice<T> {
		UnsafeSlice {
			address: self.address,
			length: self.length,
		}
	}
}

unsafe impl<T> Send for UnsafeSlice<T> where T: Send + Sync {}
unsafe impl<T> Sync for UnsafeSlice<T> where T: Sync {}

unsafe impl<T> Send for UnsafeSliceMut<T> where T: Send {}
unsafe impl<T> Sync for UnsafeSliceMut<T> where T: Sync {}