// use std::{fmt, hash};
// use std::marker::PhantomData;
// use std::any::{type_name};

// use crate::if_dbg;
// // use extend::ext;


// #[derive(Eq)] // Debug, PartialEq, Clone, Copy, Display, Hash
// pub struct Idx<T> { 
//     raw: usize, 
//     _mark: PhantomData<T>, 
//     #[cfg(debug_assertions)] slice: *const [T] 
// }

// impl<T> Idx<T> {
//     pub fn new(slice: &[T], raw: usize) -> Self { Idx {raw, _mark: PhantomData, #[cfg(debug_assertions)] slice }  }
//     pub fn raw(&self) -> usize { self.raw }
// }
// impl Idx<()> {
//     // pub fn new(slice: &[T], raw: usize) -> Self { Idx {raw, _mark: PhantomData, #[cfg(debug_assertions)] slice }  }
    
// }

// // #[ext] pub impl usize {
// //     #[inline] fn as_idx<T>(self) -> Idx<T> { 

// //         Idx {raw: self, _mark: PhantomData, #[cfg(debug_assertions)] collection: ptr } 
// //     }
// // }


// impl<T: fmt::Debug> fmt::Debug for Idx<T> {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         if cfg!(debug_assertions) {
//             write!(f, "Idx {:?}", unsafe{&* self.slice}[self.raw])
//         } else {
//             let type_name = type_name::<T>().rsplit("::").next().unwrap();
//             write!(f, "Idx<{}> {}", type_name, self.raw)
//         }
        
//     }
// }

// // trivial impl (bitwise copy, inhirited display and call hash) (these have broader domain than #[derive(...)])
// impl<T> Copy for Idx<T> {}
// impl<T> Clone for Idx<T> { fn clone(&self) -> Self { *self } } 
// impl<T> PartialEq for Idx<T> { fn eq(&self, other: &Self) -> bool { self.raw == other.raw } }
// impl<T> hash::Hash for Idx<T> { fn hash<H: hash::Hasher>(&self, state: &mut H) { self.raw.hash(state); } }
// impl<T: fmt::Display> fmt::Display for Idx<T> { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self) } }

