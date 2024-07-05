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

use std::ops::{AddAssign, Deref, SubAssign};

macro_rules! non_float_types {
    ($($ty:ty),*) => {
        $(
            impl std::cmp::PartialEq for Moving<$ty> {
            fn eq(&self, other: &Self) -> bool {
                self.current == other.current
            }
        }
        )*
    };
}

macro_rules! non_float_typesu {
    ($($ty:ty),*) => {
        $(
            impl std::cmp::PartialEq<$ty> for Moving<$ty> {
            fn eq(&self, other: &$ty) -> bool {
                self.current == *other
            }
        }
        )*
    };
}

macro_rules! float_types {
    ($($ty:ty),*) => {
        $(
            impl std::cmp::PartialEq for Moving<$ty> {
                fn eq(&self, other: &Self) -> bool {
                        (self.current - other.current).abs() < <$ty>::EPSILON
                    }
            }
        )*
    };
}
macro_rules! float_typesu {
    ($($ty:ty),*) => {
        $(
            impl std::cmp::PartialEq<$ty> for Moving<$ty> {
                fn eq(&self, other: &$ty) -> bool {
                        (self.current - *other).abs() < <$ty>::EPSILON
                    }
            }
        )*
    };
}

macro_rules! from_size {
    ($($ty:ty),*) => {
        $(
            impl FromUsize for $ty {
                fn from_usize(value: usize) -> Self {
                    value as Self
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

            impl SubAssign<$ty> for Moving<$ty> {
                fn sub_assign(&mut self, other: $ty) {
                    self.sub(other);
                }
            }
        )*


    };
}

non_float_types!(usize, i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);
non_float_typesu!(usize, i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);
from_size!(usize, i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, f32, f64);
assign_types!(usize, i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, f32, f64);
float_types!(f32, f64);
float_typesu!(f32, f64);

#[derive(Debug, Clone, Default)]
pub struct Moving<T> {
    current: T,
    count: usize,
}

pub trait FromUsize {
    fn from_usize(value: usize) -> Self;
}

impl<T> Moving<T>
where
    T: Default
        + Copy
        + std::ops::Div<Output = T>
        + std::ops::Add<Output = T>
        + std::ops::Sub<Output = T>
        + std::ops::Mul<Output = T>
        + std::cmp::PartialEq
        + FromUsize,
{
    pub fn new() -> Self {
        Self {
            current: T::default(),
            count: 0,
        }
    }

    pub fn add(&mut self, value: T) {
        self.current =
            (self.current * T::from_usize(self.count) + value) / T::from_usize(self.count + 1);
        self.count += 1;
    }

    pub fn sub(&mut self, value: T) {
        if self.count > 1 {
            self.current =
                (self.current * T::from_usize(self.count) - value) / T::from_usize(self.count - 1);
        } else {
            self.current = T::default();
        }
        if self.count > 0 {
            self.count -= 1;
        }
    }
}

impl AddAssign for Moving<usize> {
    fn add_assign(&mut self, other: Self) {
        self.add(other.current);
    }
}

impl SubAssign for Moving<usize> {
    fn sub_assign(&mut self, other: Self) {
        self.sub(other.current);
    }
}

impl<T> Deref for Moving<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.current
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
    fn sub_moving_average() {
        let mut moving_average: Moving<usize> = Moving::new();
        moving_average.add(10);
        moving_average.add(20);
        moving_average.sub(10);
        assert_eq!(moving_average, 20);
    }

    #[test]
    fn float_moving_average() {
        let mut moving_average: Moving<f32> = Moving::new();
        moving_average.add(10.0);
        moving_average.add(20.0);
        assert_eq!(moving_average, 15.0);
    }

    #[test]
    fn float_moving_average_sub() {
        let mut moving_average: Moving<f32> = Moving::new();
        moving_average.add(10.0);
        moving_average.add(20.0);
        moving_average.sub(10.0);
        assert_eq!(moving_average, 20.0);
    }

    #[test]
    fn first_operation_sub() {
        let mut moving_average: Moving<usize> = Moving::new();
        moving_average.sub(10);
        assert_eq!(moving_average, 0);
    }

    #[test]
    fn first_operation_sub_float() {
        let mut moving_average: Moving<f32> = Moving::new();
        moving_average.sub(10.0);
        assert_eq!(moving_average, 0.0);
    }

    #[test]
    fn first_operation_sub_then_add() {
        let mut moving_average: Moving<usize> = Moving::new();
        moving_average.sub(10);
        moving_average.add(10);
        assert_eq!(moving_average, 10);
    }

    #[test]
    fn assign_add() {
        let mut moving_average: Moving<usize> = Moving::new();
        moving_average.add(10);
        moving_average += 20;
        assert_eq!(moving_average, 15);
    }

    #[test]
    fn assign_sub() {
        let mut moving_average: Moving<usize> = Moving::new();
        moving_average.add(10);
        moving_average.add(20);
        moving_average -= 10;
        assert_eq!(moving_average, 20);
    }

    #[test]
    fn assign_sub_float() {
        let mut moving_average: Moving<f32> = Moving::new();
        moving_average.add(10.0);
        moving_average.add(20.0);
        moving_average -= 10.0;
        assert_eq!(moving_average, 20.0);
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
}
