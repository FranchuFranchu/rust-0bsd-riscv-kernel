#![cfg_attr(not(test), no_std)]
#![feature(unsized_fn_params)]

extern crate alloc;

use alloc::boxed::Box;
use core::{any::TypeId, mem::size_of};

pub use to_trait_macro::to_trait;

pub trait Null {}

pub trait ToTrait {
    fn cast_to_trait(self, target_type_id: TypeId) -> Option<Box<dyn Null>>;
    fn cast_to_trait_ref(&self, target_type_id: TypeId) -> Option<&dyn Null>;
    fn cast_to_trait_mut(&mut self, target_type_id: TypeId) -> Option<&mut dyn Null>;
}

pub trait ToTraitExt: ToTrait {
    fn to_trait<T: 'static + ?Sized>(self) -> Option<Box<T>> {
        let id = core::any::TypeId::of::<T>();
        let t = self.cast_to_trait(id)?;
        Some(unsafe { Box::from_raw(core::mem::transmute_copy(&Box::into_raw(t))) })
    }
    fn to_trait_ref<T: 'static + ?Sized>(&self) -> Option<&T> {
        assert!(size_of::<&T>() == size_of::<&dyn Null>());
        let id = core::any::TypeId::of::<T>();
        let t = self.cast_to_trait_ref(id)?;
        Some(unsafe { core::mem::transmute_copy(&t) })
    }
    fn to_trait_mut<T: 'static + ?Sized>(&mut self) -> Option<&mut T> {
        assert!(size_of::<&mut T>() == size_of::<&mut dyn Null>());
        let id = core::any::TypeId::of::<T>();
        let t = self.cast_to_trait_mut(id)?;
        Some(unsafe { core::mem::transmute_copy(&t) })
    }
}

pub trait ToTraitAny: ToTrait + core::any::Any {}

impl<T> ToTraitExt for T where T: ToTrait + ?Sized {}

impl<T> ToTraitAny for T where T: ToTrait + core::any::Any + ?Sized {}

#[test]
#[derive(Debug)]
#[to_trait(alloc::fmt::Debug, core::any::Any)]
struct A {
    a: u64,
}

#[test]
pub fn test() {
    let k = A { a: 6 };
    let k2: &dyn alloc::fmt::Debug = k.to_trait_ref().unwrap();
    println!("{:?}", k2);
}
