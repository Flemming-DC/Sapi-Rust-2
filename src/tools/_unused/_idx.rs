// use std::marker::PhantomData;
// use crate::vendor::bumpalo::collections::Vec as bVec;
// use std::{fmt, hash};

// pub struct Idx<T: ?Sized>{ idx: usize, _mark: PhantomData<T>, 
//     // #[cfg(debug_assertions)] id: Id<()> 
// }

// impl<'a, T> Idx<T> {
//     pub fn new(idx: usize) -> Self {
//         return Idx { idx, _mark: PhantomData::<T> };
//     }
//     pub fn get(self, v: &'a[T]) -> &'a T {
//         return &v[self.idx];
//     }
//     pub fn get_mut(self, v: &'a mut[T]) -> &'a mut T {
//         return &mut v[self.idx];
//     }    
//     // pub fn new(idx: usize, identifiable: &impl hash::Hash) -> Self {
//     //     return Idx { idx, _mark: PhantomData::<T>, #[cfg(debug_assertions)] id: identifiable.hash() };
//     // }
//     // pub fn from_vec(v: &'a mut Vec<T>, value: T) -> Self {
//     //     v.push(value);
//     //     return Idx { idx: v.len(), _mark: PhantomData::<T> };
//     // }
//     // pub fn from_b_vec(v: &'a mut bVec<T>, value: T) -> Self {
//     //     v.push(value);
//     //     return Idx { idx: v.len(), _mark: PhantomData::<T> };
//     // }
//     // pub fn get(self, v: &'a[T]) -> &'a T {
//     //     return &v[self.idx];
//     // }
//     // pub fn get_mut(self, v: &'a mut[T]) -> &'a mut T {
//     //     return &mut v[self.idx];
//     // }
// }