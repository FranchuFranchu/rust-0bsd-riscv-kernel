#![no_std]


kernel_proc_macros::generate_extra_data_structs!();

pub use kernel_proc_macros::AsRegister;
use smallvec::SmallVec;

pub trait AsRegister {
    fn as_register(&self) -> (usize, SmallVec<[usize; 2]>);
    fn recursive_variant_count() -> usize;
    fn from_register(variant_extra: &(usize, &[usize])) -> Self;
}

// Implement it for all unsigned int types
// Won't work since sizeof T might not be sizeof usize, so you can't horizontally cast their borrows
/*
impl<T> AsRegister for T where T: Into<usize> {
    #[inline]
    fn as_register(&self) -> (usize, &[usize]) {
        (0, &[self.into()])
    }; 
    
    fn recursive_variant_count() -> {
        1
    };
}

*/
impl AsRegister for usize {
    #[inline]
    fn as_register(&self) -> (usize, SmallVec<[usize; 2]>) {
        (0, smallvec::smallvec![(*self)])
    }
    
    fn recursive_variant_count() -> usize {
        1
    }
    
    fn from_register(variant_extra: &(usize, &[usize])) -> Self {
        variant_extra.1[0]
    }
}


macro_rules! impl_as_register_as_same_size_template {
    ( $($t:ty),* ) => {
    $( impl<T> AsRegister for $t
    {
        fn as_register(&self) -> (usize, SmallVec<[usize; 2]>) {
            (0, smallvec::smallvec![(*self as _)])
        }
        
        fn recursive_variant_count() -> usize {
            1
        }
        
        fn from_register(variant_extra: &(usize, &[usize])) -> Self {
            variant_extra.1[0] as _
        }
    }) *
    }
}

impl_as_register_as_same_size_template! { *const T, *mut T}


macro_rules! impl_as_register_as {
    ( $($t:ty),* ) => {
    $( impl AsRegister for $t
    {
        fn as_register(&self) -> (usize, SmallVec<[usize; 2]>) {
            (0, smallvec::smallvec![(*self as _)])
        }
        
        fn recursive_variant_count() -> usize {
            1
        }
        
        fn from_register(variant_extra: &(usize, &[usize])) -> Self {
            variant_extra.1[0] as _
        }
    }) *
    }
}


impl_as_register_as!{ u8, u16, u32, u64, i8, i16, i32, i64 }