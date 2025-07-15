use std::ops::{Index, IndexMut};
use std::{marker::PhantomData, slice::{Iter, IterMut}};
use std::{fmt, hash};
use allocator_api2::alloc::Allocator;
use allocator_api2::vec::Vec;
use crate::dbg_assert;
use super::Id;

// ----------- Restricted Array ----------- //
/// The restricted array limits some uses to provide speed and safety.

#[derive(Debug, Eq, PartialEq, Clone, Hash)] // Display, Index, IndexMut, IntoIterator
pub struct ResArray<T, A: Allocator> {
    data: Vec<T, A>,
    #[cfg(debug_assertions)] id: Id<()>
}

impl<T, A: Allocator> ResArray<T, A> {
    #[inline] pub fn new_in(capacity: usize, alloc: A) -> Self { // from_vec_a arent useful, but from-vec is for compatibility
        ResArray { data: Vec::with_capacity_in(capacity, alloc), #[cfg(debug_assertions)] id: Id::new() }
    }

    #[inline] pub fn from_vec(data: Vec<T, A>) -> Self { // from_vec_a arent useful, but from-vec is for compatibility
        ResArray { data, #[cfg(debug_assertions)] id: Id::new() }
    }

    #[inline] pub fn push(&mut self, value: T) -> XIdx<T> { 
        self.data.push(value);
        XIdx {raw: self.data.len() - 1, _mark: PhantomData, #[cfg(debug_assertions)] array_id: self.id }
    }
    #[inline] pub fn swap(&mut self, idx1: XIdx<T>, idx2: XIdx<T>) {
        self.check_id_x(&idx1); 
        self.check_id_x(&idx2); 
        self.data.swap(idx1.raw, idx2.raw);
    }
    #[inline] pub fn remove(&mut self, idx: XIdx<T>, last: XIdx<T>) -> T {
        self.check_id_x(&idx); 
        self.check_id_x(&last); 
        self.data.swap_remove(idx.raw)
    }

    #[inline] pub fn pop(&mut self, last: XIdx<T>) -> Option<T> { self.check_id_x(&last); self.data.pop() }
    // #[inline] pub fn get(&self, idx: &Idx<T>) -> &T             { self.check_id(idx.stack_id); &self.data[idx.value] }
    // #[inline] pub fn set(&mut self, idx: &Idx<T>, value: T)     { self.check_id(idx.stack_id); self.data[idx.value] = value; }
    // #[inline] pub fn get_x(&self, idx: &XIdx<T>) -> &T          { self.check_id(idx.stack_id); &self.data[idx.value] }
    // #[inline] pub fn set_x(&mut self, idx: &XIdx<T>, value: T) { self.check_id(idx.stack_id); self.data[idx.value] = value; }

    #[inline] pub fn iter(&self) -> Iter<'_, T>                 { self.data.iter() }
    #[inline] pub fn iter_mut(&mut self) -> IterMut<'_, T>      { self.data.iter_mut() }
    #[inline] fn check_id_x(&self, idx: &XIdx<T>) {dbg_assert!(idx.array_id == self.id, "You put the index into the wrong restricted array.");}
    #[inline] fn check_id(&self, idx: &SIdx<T>)    {dbg_assert!(idx.array_id == self.id, "You put the index into the wrong restricted array.");}

    #[inline] pub unsafe fn to_vec(self) -> Vec<T, A> { self.data }
    #[inline] pub fn len(&self) -> usize { self.data.len() }
    
}

// --- Traits --- //
impl<T: Sized + fmt::Display, A: Allocator> fmt::Display for ResArray<T, A> { 
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self) } 
}

impl<T, A: Allocator> Index<&SIdx<T>> for ResArray<T, A> { type Output = T;
    #[inline] fn index(&self, idx: &SIdx<T>) -> &Self::Output { 
        self.check_id(idx); 
        unsafe { self.data.get_unchecked(idx.raw) }
}}
impl<T, A: Allocator> IndexMut<&SIdx<T>> for ResArray<T, A> {
    #[inline] fn index_mut(&mut self, idx: &SIdx<T>) -> &mut Self::Output { 
        self.check_id(idx); 
        unsafe { self.data.get_unchecked_mut(idx.raw) }
}}
impl<T, A: Allocator> Index<&XIdx<T>> for ResArray<T, A> { type Output = T;
    #[inline] fn index(&self, idx: &XIdx<T>) -> &Self::Output { 
        self.check_id_x(idx); 
        unsafe { self.data.get_unchecked(idx.raw) }
}}
impl<T, A: Allocator> IndexMut<&XIdx<T>> for ResArray<T, A> {
    #[inline] fn index_mut(&mut self, idx: &XIdx<T>) -> &mut Self::Output { 
        self.check_id_x(idx); 
        unsafe { self.data.get_unchecked_mut(idx.raw) }
}}

impl<T, A: Allocator> IntoIterator for ResArray<T, A> {
    type Item = T;
    type IntoIter = allocator_api2::vec::IntoIter<Self::Item, A>;
    #[inline] fn into_iter(self) -> Self::IntoIter { self.data.into_iter() }
}



// macro stack![1, 2, 3, 4]

// ----------- Idx ----------- //

#[derive(Debug, Eq)] // PartialEq, Clone, Copy, Display, Hash
pub struct SIdx<T: ?Sized> { raw: usize, _mark: PhantomData<T>, 
    #[cfg(debug_assertions)] array_id: Id<()>
}

impl<T: ?Sized> SIdx<T> { pub fn to_int(&self) -> usize { self.raw } }

// trivial impl (bitwise copy, inhirited display and call hash) (these have broader domain than #[derive(...)])
impl<T: ?Sized> Copy for SIdx<T> {}
impl<T: ?Sized> Clone for SIdx<T> { fn clone(&self) -> Self { *self } } 
impl<T: ?Sized> PartialEq for SIdx<T> { fn eq(&self, other: &Self) -> bool { self.raw == other.raw } }
impl<T: ?Sized> hash::Hash for SIdx<T> { fn hash<H: hash::Hasher>(&self, state: &mut H) { self.raw.hash(state); } }
impl<T: ?Sized + fmt::Display> fmt::Display for SIdx<T> { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self) } }


// ----------- XIdx ----------- //

#[derive(Debug, Eq)] // PartialEq, Display, Hash
pub struct XIdx<T: ?Sized>{ raw: usize, _mark: PhantomData<T>, 
    #[cfg(debug_assertions)] array_id: Id<()>
}

impl<T: ?Sized> XIdx<T> {
    pub fn share(self) -> SIdx<T> {
        SIdx { raw: self.raw, _mark: self._mark, #[cfg(debug_assertions)] array_id: self.array_id }
    }
    fn to_int(&self) -> usize { self.raw }
}

// trivial impl (bitwise copy, inhirited display and call hash) (these have broader domain than #[derive(...)])
impl<T: ?Sized> PartialEq for XIdx<T> { fn eq(&self, other: &Self) -> bool { self.raw == other.raw } }
impl<T: ?Sized> hash::Hash for XIdx<T> { fn hash<H: hash::Hasher>(&self, state: &mut H) { self.raw.hash(state); } }
impl<T: ?Sized + fmt::Display> fmt::Display for XIdx<T> { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self) } }


