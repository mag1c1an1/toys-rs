// SPDX-FileCopyrightText: LakeSoul Contributors
//
// SPDX-License-Identifier: Apache-2.0

use std::fmt::Debug;

use crate::scalar::{Scalar, ScalarRef};

use super::iterator::ArrayIterator;
use super::{Array, ArrayBuilder};
use bitvec::prelude::BitVec;

pub trait PrimitiveType: Scalar + Default {}

impl PrimitiveType for i32 {}
impl PrimitiveType for f32 {}

pub type I32Array = PrimitiveArray<i32>;
pub type F32Array = PrimitiveArray<f32>;

pub struct PrimitiveArray<T: PrimitiveType> {
    data: Vec<T>,
    bitmap: BitVec,
}

impl<T> Array for PrimitiveArray<T>
where
    T: PrimitiveType + Scalar<ArrayType = Self>,
    for<'a> T: ScalarRef<'a, ScalarType = T, ArrayType = Self>,
    for<'a> T: Scalar<RefType<'a> = T>,
{
    type Builder = PrimitiveArrayBuilder<T>;
    type RefItem<'a> = T;
    type OwnedItem = T;

    fn get(&self, idx: usize) -> Option<Self::RefItem<'_>> {
        if self.bitmap[idx] {
            Some(self.data[idx])
        } else {
            None
        }
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn iter(&self) -> ArrayIterator<Self> {
        ArrayIterator::new(self)
    }
}

pub struct PrimitiveArrayBuilder<T: PrimitiveType> {
    data: Vec<T>,
    bitmap: BitVec,
}

impl<T> ArrayBuilder for PrimitiveArrayBuilder<T>
where
    T: PrimitiveType,
    T: Scalar<ArrayType = PrimitiveArray<T>>,
    for<'a> T: ScalarRef<'a, ScalarType = T, ArrayType = PrimitiveArray<T>>,
    for<'a> T: Scalar<RefType<'a> = T>,
{
    type A = PrimitiveArray<T>;

    fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            bitmap: BitVec::with_capacity(capacity),
        }
    }

    fn push(&mut self, value: Option<<Self::A as Array>::RefItem<'_>>) {
        match value {
            Some(v) => {
                self.data.push(v);
                self.bitmap.push(true);
            }
            None => {
                self.data.push(T::default());
                self.bitmap.push(false);
            }
        }
    }

    fn finish(self) -> Self::A {
        PrimitiveArray {
            data: self.data,
            bitmap: self.bitmap,
        }
    }
}
