//! # Moving Average Library
//!
//! `moving_average` is a library for calculating the moving average, mode, and other statistical operations on a stream of data.
//!
//! ## Features
//!
//! - Calculate moving average in an ergonomic way.
//! - Optional feature to calculate the mode of the data.
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
//! let mut moving_average = Moving::new();
//! moving_average.add(10);
//! moving_average.add(20);
//! assert_eq!(moving_average, 15);
//! ```
//!
//! ### Mode Calculation
//!
//! Enable the `mode` feature in your `Cargo.toml` to use mode calculation:
//!
//! ```toml
//! [dependencies.moving_average]
//! version = "0.1.0"
//! features = ["mode"]
//! ```
//!
//! With the `mode` feature enabled, you can calculate the mode of the data:
//!
//! ```rust
//! use moving_average::Moving;
//!
//! let mut moving_average = Moving::new();
//! moving_average.add(10);
//! moving_average.add(20);
//! moving_average.add(20);
//! moving_average.add(20);
//! moving_average.add(30);
//! assert_eq!(moving_average.mode, 20);
//! ```
//!
//! ## Features
//!
//! - `mode`: Enables calculation of the mode of the data.
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

non_float_types!(usize, i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);
non_float_typesu!(usize, i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);
float_types!(f32, f64);
float_typesu!(f32, f64);

#[derive(Debug, Clone)]
pub struct Moving<T> {
    current: T,
    count: usize,
}

impl Default for Moving<usize> {
    fn default() -> Self {
        Self::new()
    }
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
        + From<usize>,
{
    pub fn new() -> Self {
        Self {
            current: T::default(),
            count: 0,
        }
    }

    pub fn add(&mut self, value: T) {
        self.current = (self.current * (T::from(self.count)) + value) / T::from(self.count + 1);
        self.count += 1;
    }

    pub fn sub(&mut self, value: T) {
        self.current = (self.current * (T::from(self.count)) - value) / T::from(self.count - 1);
        self.count -= 1;
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
}
