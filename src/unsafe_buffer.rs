//! Un-lifetimed slices that are unsafe to create, but safe to use.
//!
//! People who create new instances of these structs must make sure to drop them before the slice they were created from gets dropped.
//! Failure to do this might result in UB.
//!
//! These are mostly just a tool to make code cleaner. Instead of passing around `(*const T, usize)` and saying "yes, this buffer is valid" you can pass one of these slices which have the invariant in their `new()` method.
//!
//! See the documentation of each of this module's structs for examples

use core::marker::PhantomData;

#[derive(Debug)]

/// A shared, un-lifetimed slice that is unsafe to create, but safe to use.
///
/// Analogous to a `(*const T, usize)` that is guaranteed to be valid
///
/// # Examples
///
/// ```
/// let array = [2, 3, 5, 7, 11];
/// // We promise to hold UnsafeSlice::new()'s invariants with this `unsafe` block
/// let unsafe_slice = unsafe { UnsafeSlice::new(&array) };
/// // Now, we must make sure to not drop `array` before we drop `unsafe_slice`
///
/// assert!(array == array.get());
///
/// // We can also place this buffer in a future, in a driver's internal state, etc.
/// // Finally, when we're done:
///
/// drop(unsafe_slice);
/// drop(array);
/// ```
pub struct UnsafeSlice<T> {
    address: *const T,
    length: usize,
}

impl<T> UnsafeSlice<T> {
    /// Creates a new unsafe shared slice from a regular Rust slice
    /// # Safety
    /// `slice` must be still alive when the struct is dropped
    pub unsafe fn new(slice: &[T]) -> Self {
        Self {
            address: slice.as_ptr(),
            length: slice.len(),
        }
    }

    ///
    pub fn get(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.address, self.length) }
    }

    /// The starting address of this slice
    pub fn address(&self) -> *const T {
        self.address
    }

    /// The length of this slice. This is the amount of elements, not the amount of bytes.
    pub fn length(&self) -> usize {
        self.length
    }

    /// Copies this slice.
    /// # Safety
    /// Since this artifically extends the lifetime of the unsafe slice, you must make sure that the original Rust slice is still alive during the lifetime of the new unsafe slice.
    pub unsafe fn copy(&self) -> Self {
        Self {
            address: self.address,
            length: self.length,
        }
    }

    /// Turns this slice into a mutable slice. I don't really think this is usually sound.
    pub unsafe fn upcast(self) -> UnsafeSliceMut<T> {
        UnsafeSliceMut {
            address: self.address as *mut T,
            length: self.length,
        }
    }
}

#[derive(Debug)]
/// An exclusive, un-lifetimed slice that is unsafe to create, but safe to use.
///
/// Analogous to a `(*mut T, usize)` that is guaranteed to be valid and exclusive
///
/// # Examples
///
/// ```
/// let array = [2, 3, 5, 7, 11];
/// // We promise to hold UnsafeSliceMut::new()'s invariants with this `unsafe` block
/// let unsafe_slice = unsafe { UnsafeSliceMut::new(&mut array) };
/// // Now, we must make sure to not drop `array` before we drop `unsafe_slice`
///
/// // `array[0] = 2` // Don't do this
///
/// let slice_borrow = unsafe_slice.get();
/// // This invalidates unsafe_slice so we can't do this again without reconstructing it
///
/// slice_borrow[0] = 2;
///
/// let unsafe_slice = unsafe { UnsafeSliceMut::new(slice_borrow) };
///
/// // We can also place this buffer in a future, in a driver's internal state, etc.
/// // Finally, when we're done
///
/// drop(unsafe_slice);
/// drop(array);
/// ```
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
