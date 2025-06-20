#![allow(unused)]

mod iterator {
    use crate::Array;

    pub struct ArrayIterator<'a, A: Array> {
        array: &'a A,
        pos: usize,
    }

    impl<'a, A: Array> Iterator for ArrayIterator<'a, A> {
        type Item = ();

        fn next(&mut self) -> Option<Self::Item> {
            todo!()
        }
    }
}

mod string_array {
    use bitvec::prelude::BitVec;

    use crate::Array;

    pub struct StringArray {
        data: Vec<u8>,
        offset: Vec<usize>,
        bitmap: BitVec,
    }
    impl Array for StringArray {
        type RefItem<'a> = &'a str;

        fn get(&self, idx: usize) -> Option<Self::RefItem<'_>> {
            if self.bitmap[idx] {
                let range = self.offset[idx]..self.offset[idx + 1];
                Some(unsafe { std::str::from_utf8_unchecked(&self.data[range]) })
            } else {
                None
            }
        }

        fn len(&self) -> usize {
            self.bitmap.len()
        }
    }

    impl StringArray {}
}

mod primitive_array {
    use bitvec::prelude::BitVec;

    use crate::Array;

    pub trait PrimitiveType: Copy + Send + Sync + Default + 'static {}

    impl PrimitiveType for i32 {}

    pub struct PrimitiveArray<T: PrimitiveType> {
        data: Vec<T>,
        bitmap: BitVec,
    }

    impl<T: PrimitiveType> Array for PrimitiveArray<T> {
        type RefItem<'a> = T;

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
    }
}

trait Array: Sized + 'static {
    type RefItem<'a>;
    fn get(&self, idx: usize) -> Option<Self::RefItem<'_>>;
    fn len(&self) -> usize;
}

mod tests {
    use crate::Array;

    fn build_array_from_vec<A: Array>(items: &[Option<A::RefItem<'_>>]) -> A {
        todo!()
    }

    fn test_build_int32_array() {
        let data = vec![Some(1), None, Some(2), Some(3), None, Some(4)];
        // let array = build_array_from_vec::<>()
        todo!()
    }
}
