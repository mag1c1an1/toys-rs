// SPDX-FileCopyrightText: LakeSoul Contributors
//
// SPDX-License-Identifier: Apache-2.0

use crate::array::{Array, F32Array, I32Array, StringArray};
use std::{fmt::Debug, panic::RefUnwindSafe};

pub trait Scalar: Clone + Send + Sync + Debug + 'static {
    type ArrayType: Array<OwnedItem = Self>;
    type RefType<'a>: ScalarRef<'a, ScalarType = Self, ArrayType = Self::ArrayType>;

    fn as_scalar_ref(&self) -> Self::RefType<'_>;
}

pub trait ScalarRef<'a>: Debug + Clone + Copy {
    type ArrayType: Array<RefItem<'a> = Self>;
    type ScalarType: Scalar<RefType<'a> = Self>;

    fn to_owned_scalar(&self) -> Self::ScalarType;
}

impl Scalar for i32 {
    type ArrayType = I32Array;

    type RefType<'a> = i32;

    fn as_scalar_ref(&self) -> i32 {
        *self
    }
}

impl<'a> ScalarRef<'a> for i32 {
    type ArrayType = I32Array;

    type ScalarType = i32;

    fn to_owned_scalar(&self) -> Self::ScalarType {
        *self
    }
}

impl Scalar for f32 {
    type ArrayType = F32Array;

    type RefType<'a> = f32;

    fn as_scalar_ref(&self) -> f32 {
        *self
    }
}

impl<'a> ScalarRef<'a> for f32 {
    type ArrayType = F32Array;

    type ScalarType = f32;

    fn to_owned_scalar(&self) -> Self::ScalarType {
        *self
    }
}

/// Implement [`Scalar`] for `String`.
impl Scalar for String {
    type ArrayType = StringArray;
    type RefType<'a> = &'a str;

    fn as_scalar_ref(&self) -> &str {
        self.as_str()
    }
}

/// Implement [`ScalarRef`] for `&str`.
impl<'a> ScalarRef<'a> for &'a str {
    type ArrayType = StringArray;
    type ScalarType = String;

    fn to_owned_scalar(&self) -> String {
        self.to_string()
    }
}
#[cfg(test)]
mod tests {
    use crate::array::{Array, ArrayBuilder, I32Array};

    fn build_array_repeated<A: Array>(item: A::RefItem<'_>, len: usize) -> A {
        let mut builder = A::Builder::with_capacity(len);
        for _ in 0..len {
            builder.push(Some(item));
        }
        builder.finish()
    }

    fn build_array_repeated_owned<A: Array>(item: A::OwnedItem, len: usize) -> A {
        let mut builder = A::Builder::with_capacity(len);
        for _ in 0..len {
            builder.push(Some(item.as_scalar_ref()));
        }
        builder.finish()
    }

    /// Test if an array has the same repeating content
    fn check_array_eq<'a, A: Array>(array: &'a A, item: A::RefItem<'a>)
    where
        A::RefItem<'a>: PartialEq,
    {
        for a in array.iter() {
            assert_eq!(a, Some(item));
        }
    }

    #[test]
    fn test_build_int32_repeat_array() {
        let array = build_array_repeated::<I32Array>(1, 233);
        check_array_eq(&array, 1);
        let array = build_array_repeated_owned::<I32Array>(1, 233);
        check_array_eq(&array, 1);
    }
}
