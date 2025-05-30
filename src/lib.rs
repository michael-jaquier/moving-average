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

use num_traits::ToPrimitive;
use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::Display,
    marker::PhantomData,
    ops::{AddAssign, Deref},
};

use ordered_float::OrderedFloat;

#[derive(Debug, Clone)]
pub struct Value(f64);

impl Deref for Value {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

macro_rules! impl_partial_eq {
    ($($ty:ty), *) => {
        $(
            impl PartialEq<$ty> for Value {
                fn eq(&self, other: &$ty) -> bool {
                    self.0 == *other as f64
                }
            }
        )*
    };
    () => {

    };
}

impl_partial_eq!(usize, i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, f32, f64);

macro_rules! utilities {
    ($($ty:ty),*) => {
        $(
            impl AddAssign<$ty> for Moving<$ty> {
                fn add_assign(&mut self, other: $ty) {
                    let _ = self.add(other);
                }
            }

            impl PartialEq<$ty> for Moving<$ty> {
                fn eq(&self, other: &$ty) -> bool {
                    self.mean() == *other as f64
                }
            }

            impl PartialOrd<$ty> for Moving<$ty> {
                fn partial_cmp(&self, other: &$ty) -> Option<std::cmp::Ordering> {
                    self.mean().partial_cmp(&(*other as f64))
                }
            }

            impl PartialEq<Moving<$ty>> for $ty {
                fn eq(&self, other: &Moving<$ty>) -> bool {
                    *self as f64 == other.mean()
                }
            }

            impl PartialOrd<Moving<$ty>> for $ty {
                fn partial_cmp(&self, other: &Moving<$ty>) -> Option<std::cmp::Ordering> {
                    (*self as f64).partial_cmp(&other.mean())
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
                self.mean() == *other as f64
            }
        }

        impl PartialEq<f64> for Moving<$ty> {
            fn eq(&self, other: &f64) -> bool {
                self.mean() == *other
            }
        }
    )*

    };
}

macro_rules! signed {
    ($($ty:ty), *) => {
        $(
        impl Sign for $ty {
            fn signed() -> bool {
               true
            }
        }
        )*
    };
}
macro_rules! unsigned {
    ($($ty:ty), *) => {
    $(
        impl Sign for $ty {
            fn signed() -> bool {
               false
            }
        }
    )*
    };
}

utilities!(usize, i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, f32, f64);
partial_non!(usize, i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);
signed!(i8, i16, i32, i64, i128, f32, f64);
unsigned!(usize, u8, u16, u32, u64, u128);

pub trait Sign {
    fn signed() -> bool;
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
/// Represents the possible errors that can occur in the `Moving` struct.
pub enum MovingError {
    /// Error indicating that a negative value was attempted to be added to an unsigned type.
    NegativeValueToUnsignedType,

    /// Error indicating that an overflow occurred during an operation.
    /// Note: This is unlikely to occur with floating-point operations.
    Overflow,

    /// Error indicating that an underflow occurred during an operation.
    Underflow,

    /// Error indicating that the count of values has overflowed.
    CountOverflow,

    /// Error indicating that a value has reached or exceeded the specified threshold.
    ///
    /// This error is triggered when a value added to the `Moving` instance meets or exceeds
    /// the threshold value specified during the creation of the instance. This can be used
    /// to signal that a certain limit has been reached, which might require special handling
    /// or termination of further processing.
    ThresholdReached,
}

/// `Moving<T>` provides an ergonomic way to compute the moving average, mode, and count
/// for a sequence of values of type `T`. It supports both signed and unsigned numeric types,
/// and can enforce a threshold to stop accepting new values when the mean reaches or exceeds it.
///
/// Internally, it tracks:
/// - The number of values added (`count`)
/// - The current mean (`mean`)
/// - The frequency of each value for mode calculation (`mode`)
/// - An optional threshold (`threshold`)
///
/// # Type Parameters
///
/// - `T`: The numeric type of the values (e.g., `usize`, `i32`, `f64`).
///
/// # Examples
///
/// ```
/// use moving_average::Moving;
///
/// let moving = Moving::new();
/// moving.add(10);
/// moving.add(10);
/// moving.add(10);
/// moving.add(20);
/// assert_eq!(moving.mean(), 12.5);
/// assert_eq!(moving.count(), 4);
/// assert_eq!(moving.mode(), 10.0);
/// ```
#[derive(Debug, Default)]
pub struct Moving<T> {
    count: RefCell<usize>,
    mean: RefCell<f64>,
    mode: RefCell<HashMap<OrderedFloat<f64>, usize>>,
    threshold: f64,
    phantom: std::marker::PhantomData<T>,
}

impl<T> Moving<T>
where
    T: Sign + ToPrimitive,
{
    /// Creates a new [`Moving<T>`] instance with default values.
    ///
    /// # Returns
    ///
    /// A new instance of [`Moving<T>`].
    /// Values can ge added to this instance to calculate the moving average.
    pub fn new() -> Self {
        Self {
            count: RefCell::new(0),
            mean: RefCell::new(0.0),
            mode: RefCell::new(HashMap::new()),
            threshold: f64::MAX,
            phantom: PhantomData,
        }
    }

    /// Creates a new [`Moving<T>`] instance with a specified threshold.
    ///
    /// This method initializes the `count` to 0, `mean` to 0.0, `is_error` to `false`,
    /// and `threshold` to the provided value.
    ///
    /// # Parameters
    ///
    /// - `threshold`: The threshold value to be used for the new instance.
    ///
    /// # Returns
    ///
    /// A new instance of [`Moving<T>`] with the specified threshold.
    /// Values can be added to this instance to calculate the moving average.
    /// When values are greater than or equal to the threshold, the [`MovingResults::ThresholdReached`] variant is returned and no further values are added.
    pub fn new_with_threshold(threshold: f64) -> Self {
        Self {
            count: RefCell::new(0),
            mean: RefCell::new(0.0),
            mode: RefCell::new(HashMap::new()),
            threshold,
            phantom: PhantomData,
        }
    }
    /// Adds a value to the current statistical collection, updating the mean accordingly.
    ///
    /// This function converts the input value to an `f64` and then updates the mean of the collection
    /// based on the new value.
    ///
    /// # Returns
    /// If the mean is less than the threshold, the [`MovingResults::Value`] variant is returned with the new mean.
    ///
    /// # Panics
    ///
    /// Panics if the type `T` is unsigned and a negative value is attempted to be added. This is because
    /// negative values are not allowed for unsigned types. If negative values are needed, it is recommended
    /// to use signed types instead.
    pub fn add_with_result(&self, value: T) -> Result<f64, MovingError> {
        let value_f64: f64 = value.to_f64().unwrap();
        if !T::signed() && value_f64 < 0.0 {
            return Err(MovingError::NegativeValueToUnsignedType);
        }
        let mut count = self.count.borrow_mut();
        let mut mean = self.mean.borrow_mut();
        let mut mode = self.mode.borrow_mut();
        mode.entry(OrderedFloat(value_f64))
            .and_modify(|e| *e += 1)
            .or_insert(1);

        *count += 1;
        *mean += (value_f64 - *mean) / *count as f64;

        if *mean >= self.threshold {
            return Err(MovingError::ThresholdReached);
        }

        Ok(*mean)
    }

    /// Adds a value to the current statistical collection, ignoring the result.
    ///
    /// This method calls the `add` method and ignores any errors that occur.
    pub fn add(&self, value: T) {
        let _ = self.add_with_result(value);
    }

    /// Returns the mean value of the moving average
    pub fn mean(&self) -> f64 {
        *self.mean.borrow()
    }

    /// Returns the statistical mode of the values added so far.
    ///
    /// The **mode** is the value that appears most frequently in the data set.
    ///
    /// # Behavior
    ///
    /// - If **no values** have been added, returns `0.0`.
    /// - If **all values are unique** (no repeats), returns the current mean.
    /// - If **one value** occurs more frequently than any other, returns that value as the mode.
    /// - If **multiple values** are tied for the highest frequency (i.e., a multi-modal distribution),
    ///   returns the value among the tied modes that is **closest to the mean**.
    ///
    /// # Examples
    ///
    /// ```
    /// use moving_average::Moving;
    /// let moving = Moving::new();
    /// moving.add(10);
    /// moving.add(20);
    /// moving.add(10);
    /// assert_eq!(moving.mode(), 10.0); // 10 appears most frequently
    ///
    /// let moving = Moving::new();
    /// moving.add(1);
    /// moving.add(2);
    /// moving.add(3);
    /// assert_eq!(moving.mode(), moving.mean()); // all unique, returns mean
    ///
    /// let moving = Moving::new();
    /// moving.add(10);
    /// moving.add(20);
    /// moving.add(10);
    /// moving.add(20);
    /// // Ensure the mean is lowered
    /// moving.add(1);
    /// // Both 10 and 20 appear twice, closest to mean is returned
    /// let mode = moving.mode();
    /// assert_eq!(mode, 10.0); // 10 is closer to the mean than 20
    /// ```
    pub fn mode(&self) -> f64 {
        let mode_map = self.mode.borrow();
        if mode_map.is_empty() {
            return 0.0;
        }
        // Find the maximum count
        let max_count = match mode_map.values().max() {
            Some(&max) if max > 1 => max,
            // If all values are unique, return the mean
            _ => return self.mean(),
        };
        // Collect all values with the maximum count
        let modes: Vec<_> = mode_map
            .iter()
            .filter(|&(_, &count)| count == max_count)
            .collect();
        // If there's a tie return the mode closest to the mean
        if modes.len() != 1 {
            let mean = self.mean();
            let closest = modes
                .iter()
                .min_by_key(|&&(value, _)| (value.0 - mean).abs() as i64)
                .unwrap();
            return closest.0.into_inner();
        }
        // Otherwise, return the mode value
        modes[0].0.into_inner()
    }

    /// Returns the count of events added
    ///
    /// # Examples
    ///
    /// ```
    /// use moving_average::Moving;
    /// let moving = Moving::new();
    /// moving.add(3);
    /// assert_eq!(moving.count(), 1);
    /// moving.add(3);
    /// assert_eq!(moving.count(), 2);
    /// assert_eq!(moving.mean(), 3.0);
    /// ```
    pub fn count(&self) -> usize {
        *self.count.borrow()
    }
}

impl<T> std::fmt::Display for Moving<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.mean.borrow())
    }
}

impl<T> PartialEq for Moving<T> {
    fn eq(&self, other: &Self) -> bool {
        *self.mean.borrow() == *other.mean.borrow()
    }
}

impl<T> PartialOrd for Moving<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.mean.borrow().partial_cmp(&*other.mean.borrow())
    }
}

impl std::fmt::Display for MovingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use crate::Moving;

    #[test]
    fn mode() {
        let moving = Moving::new();
        moving.add(10);
        moving.add(20);
        moving.add(10);
        assert_eq!(moving.mode(), 10.0);
    }

    #[test]
    fn big_mode() {
        let moving = Moving::new();
        for i in 0..10000 {
            moving.add(i);
        }
        assert_eq!(moving.mode(), moving.mean());
        moving.add(9999);
        assert_eq!(moving.mode(), 9999.0);
    }

    #[test]
    fn double_mode() {
        let moving = Moving::new();
        moving.add(10);
        moving.add(20);
        moving.add(10);
        moving.add(20);
        moving.add(1);
        assert_eq!(moving.mode(), 10.0);
        moving.add(3000);
        assert_eq!(moving.mode(), 20.0);
    }

    #[test]
    fn partial_order() {
        let m1 = Moving::new();
        let m2 = Moving::new();
        m1.add(10);
        m2.add(20);
        assert!(m1 < m2);
    }

    #[test]
    fn thresholds() {
        let moving_threshold = Moving::new_with_threshold(10.0);
        let result = moving_threshold.add_with_result(9);
        assert_eq!(result.unwrap(), 9.0);
        let result = moving_threshold.add_with_result(15);
        assert!(result.is_err(), "{:?}", result);
        assert_eq!(result.unwrap_err(), crate::MovingError::ThresholdReached);
    }

    #[test]
    fn never_overflow() {
        let moving_average: Moving<usize> = Moving::new();
        let result = moving_average.add_with_result(usize::MAX);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), usize::MAX as f64);
        let result = moving_average.add_with_result(usize::MAX);
        assert!(result.is_ok());

        assert_eq!(result.unwrap(), usize::MAX as f64);
    }

    #[test]
    fn add_moving_average() {
        let moving_average: Moving<usize> = Moving::new();
        moving_average.add(10);
        assert_eq!(moving_average, 10);
        moving_average.add(20);
        assert_eq!(moving_average, 15);
    }

    #[test]
    fn float_moving_average() {
        let moving_average: Moving<f32> = Moving::new();
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
        let moving_average: Moving<usize> = Moving::new();
        moving_average.add(10);
        moving_average.add(20);
        assert!(moving_average < usize::MAX)
    }

    #[test]
    fn binary_operations_float() {
        let moving_average: Moving<f32> = Moving::new();
        moving_average.add(10.0);
        moving_average.add(20.0);
        assert!(moving_average < f32::MAX)
    }

    #[test]
    fn many_operations() {
        let moving_average: Moving<_> = Moving::new();
        for i in 0..1000 {
            moving_average.add(i);
        }
        assert_eq!(moving_average, 999.0 / 2.0);
    }
}
