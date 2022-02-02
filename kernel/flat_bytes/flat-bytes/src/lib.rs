#![no_std]

use alloc::string::String;
use core::{borrow::Borrow, convert::TryInto, iter::FromIterator, marker::PhantomData};

pub use flat_bytes_derive::Flat;

extern crate alloc;

use alloc::{vec, vec::Vec};

trait ToVec {
    fn to_vec(self) -> Vec<u8>;
}

impl ToVec for &[u8] {
    fn to_vec(self) -> Vec<u8> {
        Vec::from(self)
    }
}

pub trait Flat: Sized {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize_with_size(data: &[u8]) -> Option<(Self, usize)>;
    fn deserialize(data: &[u8]) -> Option<Self> {
        Self::deserialize_with_size(data).map(|(r, _)| r)
    }
}

macro_rules! impl_primitive {
    ($t:ident) => {
        impl Flat for $t {
            fn serialize(&self) -> Vec<u8> {
                self.to_le_bytes().to_vec()
            }

            fn deserialize_with_size(data: &[u8]) -> Option<(Self, usize)> {
                if data.len() < core::mem::size_of::<Self>() {
                    return None;
                }
                let mut tmp = [0u8; core::mem::size_of::<Self>()];
                tmp.copy_from_slice(&data[..::core::mem::size_of::<Self>()]);
                Some((Self::from_le_bytes(tmp), ::core::mem::size_of::<Self>()))
            }
        }
    };
}

impl_primitive!(u8);
impl_primitive!(u16);
impl_primitive!(u32);
impl_primitive!(u64);
impl_primitive!(i8);
impl_primitive!(i16);
impl_primitive!(i32);
impl_primitive!(i64);
impl_primitive!(usize);
impl_primitive!(isize);

impl Flat for bool {
    fn serialize(&self) -> Vec<u8> {
        Flat::serialize(&(if *self { 1u8 } else { 0u8 }))
    }

    fn deserialize_with_size(data: &[u8]) -> Option<(Self, usize)> {
        <u8 as Flat>::deserialize_with_size(data).map(|(v, s)| (v != 0, s))
    }
}

macro_rules! impl_array {
    (@step ($d: ident, $idx:expr,) -> ($($body:tt)*)) => {
        impl_array!(@as_expr [$($body)*])
    };
    (@step ($d: ident, $idx:expr, $t:ident, $($ts:ident,)*) -> ($($body:tt)*)) => {
        impl_array!(@step ($d, $idx+1, $($ts,)*) -> ($($body)* $t::deserialize(&$d[::core::mem::size_of::<$t>()*($idx)..])?,));
    };
    (@as_expr $e:expr) => {$e};
    {$n:expr, $t:ident $($ts:ident)*}=> {
        impl<T: Flat> Flat for [T; $n] {
            fn serialize(&self) -> Vec<u8> {
                self.iter().map(Flat::serialize).flatten().collect()
            }

            fn deserialize_with_size(data: &[u8]) -> Option<(Self, usize)> {
                let res =
                    impl_array!(@step (data, 0, $t, $($ts,)*) -> ());
                Some((res, ::core::mem::size_of::<Self>()))
            }
        }
        impl_array!{($n - 1), $($ts)*}
    };
    {$n:expr,} => {
        impl<T: Flat> Flat for [T; $n] {
            fn serialize(&self) -> Vec<u8> {
                vec![]
            }

            fn deserialize_with_size(_data: &[u8]) -> Option<(Self, usize)> {
                Some(([], 0))
            }
        }
    };
}
impl_array! {32, T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T T}

use alloc::boxed::Box;
impl Flat for Box<[u8]> {
    fn serialize(&self) -> Vec<u8> {
        self.to_vec()
    }

    fn deserialize_with_size(data: &[u8]) -> Option<(Self, usize)> {
        Some((Box::from(data), data.len()))
    }
}

trait LimiterTrait {}

/// Returns None if the amount of items in the iterator is larger than the iterator
#[inline]
pub fn iter_to_flat<'a, T: 'a + Flat>(s: impl Iterator<Item = &'a T>) -> Option<Vec<u8>> {
    // We can copy because &T: Copy
    let mut size: u64 = 0;
    let iter = s.map(|s| s.serialize()).scan((), |_, this| {
        size += 1;
        Some(this)
    });
    Some(
        iter.collect::<Vec<_>>()
            .into_iter()
            .fold(size.serialize(), |mut first, second| {
                first.extend_from_slice(&second);
                first
            }),
    )
}

#[inline]
pub fn iter_to_flat_owned<T: Flat>(s: impl Iterator<Item = T>) -> Option<Vec<u8>> {
    // We can copy because &T: Copy
    let mut size: u64 = 0;
    let iter = s.map(|s| s.serialize()).scan((), |_, this| {
        size += 1;
        Some(this)
    });
    Some(
        iter.collect::<Vec<_>>()
            .into_iter()
            .fold(size.serialize(), |mut first, second| {
                first.extend_from_slice(&second);
                first
            }),
    )
}

struct OwnedFlatIterator<T: Flat> {
    data: Vec<u8>,
    index: usize,
    phantom: PhantomData<fn() -> T>,
}

impl<T: Flat> OwnedFlatIterator<T> {
    fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            index: 0,
            phantom: PhantomData,
        }
    }
    fn new_with_max(data: &[u8]) -> Option<(Self, usize)> {
        use core::convert::TryInto;
        let (max, read): (u64, usize) = u64::deserialize_with_size(data)?;
        Some((Self::new(Vec::from(&data[read..])), max.try_into().ok()?))
    }
}

impl<T: Flat> Iterator for OwnedFlatIterator<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        match T::deserialize_with_size(&self.data[self.index..]) {
            Some((thing, size)) => {
                self.index += size;
                Some(thing)
            }
            None => None,
        }
    }
}

impl<T: Flat> Flat for Vec<T> {
    fn serialize(&self) -> Vec<u8> {
        iter_to_flat(self.iter()).unwrap()
    }

    fn deserialize_with_size(data: &[u8]) -> Option<(Self, usize)> {
        let (data, max) = OwnedFlatIterator::new_with_max(data)?;
        Some((data.collect(), max))
    }
}

impl Flat for String {
    fn serialize(&self) -> Vec<u8> {
        iter_to_flat_owned(self.bytes()).unwrap()
    }

    fn deserialize_with_size(data: &[u8]) -> Option<(Self, usize)> {
        let (data, max): (OwnedFlatIterator<u8>, usize) = OwnedFlatIterator::new_with_max(data)?;
        Some((String::from_utf8(data.collect()).ok()?, max))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate as flat_bytes;

    #[derive(Flat)]
    #[repr(u8)]
    pub enum Foo {
        Bar = 1,
        Baz(bool) = 3,
        Blubb { a: bool, b: u8 },
    }

    static FOO: [u16; 4] = [1, 2, 3, 4];

    #[derive(Flat)]
    struct Header {
        magic: [u8; 2],
        size: u16,
        admin: bool,
    }

    #[derive(Flat)]
    struct Wrapper(Foo);

    #[test]
    fn serialize() {
        let a = Foo::Bar;
        assert_eq!(a.serialize(), vec![1]);
        let b = Foo::Baz(true);
        assert_eq!(b.serialize(), vec![3, 1]);
        let b = Foo::Baz(false);
        assert_eq!(b.serialize(), vec![3, 0]);
        let c = Foo::Blubb { a: true, b: 1 };
        assert_eq!(c.serialize(), vec![4, 1, 1]);
        let c = Foo::Blubb { a: false, b: 2 };
        assert_eq!(c.serialize(), vec![4, 0, 2]);

        assert_eq!(FOO.serialize(), vec![1, 0, 2, 0, 3, 0, 4, 0]);

        let h = Header {
            magic: *b"AB",
            size: 123,
            admin: true,
        };
        assert_eq!(h.serialize(), vec![0x41, 0x42, 123, 0, 1]);

        let w = Wrapper(Foo::Bar);
        assert_eq!(w.serialize(), vec![1]);
    }

    #[test]
    fn deserialize() {
        assert!(Foo::deserialize(&[]).is_none());
        assert!(Foo::deserialize(&[5]).is_none());
        assert!(Foo::deserialize(&[0]).is_none());
        let a = Foo::deserialize(&[1]).unwrap();
        assert!(matches!(a, Foo::Bar));
        let b = Foo::deserialize(&[3, 0]).unwrap();
        assert!(matches!(b, Foo::Baz(false)));
        let b = Foo::deserialize(&[3, 1]).unwrap();
        assert!(matches!(b, Foo::Baz(true)));
        let c = Foo::deserialize(&[4, 1, 1]).unwrap();
        assert!(matches!(c, Foo::Blubb { a: true, b: 1 }));
        let c = Foo::deserialize(&[4, 0, 2]).unwrap();
        assert!(matches!(c, Foo::Blubb { a: false, b: 2 }));

        assert_eq!(
            <[u16; 4]>::deserialize(&[1, 0, 2, 0, 3, 0, 4, 0]).unwrap(),
            FOO
        );

        let w = Wrapper::deserialize(&[1]).unwrap();
        assert!(matches!(w.0, Foo::Bar));
    }
}
