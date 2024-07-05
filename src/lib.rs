//! # Moving Average Library
//!
//! `moving_average` is a library for calculating the moving average on a stream of data.
//!
//! ## Features
//!
//! - Calculate moving average in an ergonomic way.
//!
//! ## Usage
//!
//! First, add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! moving_average = "0.1.0"
//! ```
//!
//! Then, add this to your crate root:
//!
//! ```rust
//! extern crate moving_average;
//! ```
//!
//! ### Basic Operations
//!
//! You can create a new `Moving` instance and add or subtract values from it:
//!
//! ```rust
//! use moving_average::Moving;
//!
//! let mut moving_average: Moving<usize> = Moving::new();
//! moving_average.add(10);
//! moving_average.add(20);
//! assert_eq!(moving_average, 15);
//! ```

use std::ops::{AddAssign, Deref};

macro_rules! from_size {
    ($($ty:ty),*) => {
        $(
            impl FromUsize for $ty {
                fn from_usize(value: usize) -> Self {
                    value as Self
                }
            }

            impl ToFloat64 for $ty {
                fn to_f64(self) -> f64 {
                    self as f64
                }
            }
        )*
    };
}

macro_rules! assign_types {
    ($($ty:ty),*) => {
        $(
            impl AddAssign<$ty> for Moving<$ty> {
                fn add_assign(&mut self, other: $ty) {
                    self.add(other);
                }
            }

        )*


    };
}

macro_rules! partials {
    ($($ty:ty),*) => {
        $(
            impl PartialEq<$ty> for Moving<$ty> {
                fn eq(&self, other: &$ty) -> bool {
                    self.mean == *other as f64
                }
            }

            impl PartialOrd<$ty> for Moving<$ty> {
                fn partial_cmp(&self, other: &$ty) -> Option<std::cmp::Ordering> {
                    self.mean.partial_cmp(&(*other as f64))
                }
            }

            impl PartialEq<Moving<$ty>> for $ty {
                fn eq(&self, other: &Moving<$ty>) -> bool {
                    *self as f64 == other.mean
                }
            }

            impl PartialOrd<Moving<$ty>> for $ty {
                fn partial_cmp(&self, other: &Moving<$ty>) -> Option<std::cmp::Ordering> {
                    (*self as f64).partial_cmp(&other.mean)
                }
            }

        )*

    };
}

macro_rules! partial_non {
    ($($ty:ty), *) => {
        $(
        impl PartialEq<f32> for Moving<$ty> {
            fn eq(&self, other: &f32) -> bool {
                self.mean == *other as f64
            }
        }

        impl PartialEq<f64> for Moving<$ty> {
            fn eq(&self, other: &f64) -> bool {
                self.mean == *other
            }
        }

    )*

    };
}

macro_rules! signed {
    ($($ty:ty), *) => {
        $(
        impl Sign for $ty {
            fn is_unsigned() -> bool {
                false
            }
        }
        )*
    };
}
macro_rules! unsigned {
    ($($ty:ty), *) => {
    $(
        impl Sign for $ty {
            fn is_unsigned() -> bool {
                true
            }
        }
    )*
    };
}

from_size!(usize, i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, f32, f64);
assign_types!(usize, i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, f32, f64);
partials!(usize, i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, f32, f64);
partial_non!(usize, i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);
signed!(i8, i16, i32, i64, i128, f32, f64);
unsigned!(usize, u8, u16, u32, u64, u128);

#[derive(Debug, Default)]
pub struct Moving<T> {
    count: usize,
    mean: f64,
    phantom: std::marker::PhantomData<T>,
}

pub trait FromUsize {
    fn from_usize(value: usize) -> Self;
}

pub trait ToFloat64 {
    fn to_f64(self) -> f64;
}

pub trait Sign {
    fn is_unsigned() -> bool;
}

impl<T> Moving<T>
where
    T: FromUsize + ToFloat64 + Sign,
{
    pub fn new() -> Self {
        Self {
            count: 0,
            mean: 0.0,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn add(&mut self, value: T) {
        let value = T::to_f64(value);
        self.count += 1;
        self.mean += (value - self.mean) / self.count as f64;
    }
}

impl<T> Deref for Moving<T> {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.mean
    }
}

impl<T> std::fmt::Display for Moving<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.mean)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_moving_average() {
        let mut moving_average: Moving<usize> = Moving::new();
        moving_average.add(10);
        assert_eq!(moving_average, 10);
        moving_average.add(20);
        assert_eq!(moving_average, 15);
    }

    #[test]
    fn float_moving_average() {
        let mut moving_average: Moving<f32> = Moving::new();
        moving_average.add(10.0);
        moving_average.add(20.0);
        assert_eq!(moving_average, 15.0);
    }

    #[test]
    fn assign_add() {
        let mut moving_average: Moving<usize> = Moving::new();
        moving_average.add(10);
        moving_average += 20;
        assert_eq!(moving_average, 15);
    }

    #[test]
    fn assign_add_float() {
        let mut moving_average: Moving<f32> = Moving::new();
        moving_average.add(10.0);
        moving_average += 20.0;
        assert_eq!(moving_average, 15.0);
    }

    #[test]
    fn assign_add_i64() {
        let mut moving_average: Moving<i64> = Moving::new();
        moving_average.add(10);
        moving_average += 20;
        assert_eq!(moving_average, 15);
    }
    #[test]
    fn default_works() {
        let moving_average: Moving<usize> = Default::default();
        assert_eq!(moving_average, 0);
        let moving_average: Moving<f32> = Default::default();
        assert_eq!(moving_average, 0.0);
    }

    #[test]
    fn binary_operations() {
        let mut moving_average: Moving<usize> = Moving::new();
        moving_average.add(10);
        moving_average.add(20);
        assert!(moving_average < usize::MAX)
    }

    #[test]
    fn binary_operations_float() {
        let mut moving_average: Moving<f32> = Moving::new();
        moving_average.add(10.0);
        moving_average.add(20.0);
        assert!(moving_average < f32::MAX)
    }

    #[test]
    fn many_operations() {
        let mut moving_average: Moving<_> = Moving::new();
        for i in 0..1000 {
            moving_average.add(i);
        }
        assert_eq!(moving_average, 999.0 / 2.0);
    }
}
