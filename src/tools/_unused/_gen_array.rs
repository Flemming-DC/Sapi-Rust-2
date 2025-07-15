use std::cell::Cell;
use hashbrown::hash_map::HashMap;
use hashbrown::DefaultHashBuilder;
use std::ops::{Index, IndexMut};
use std::{marker::PhantomData, slice::{Iter, IterMut}};
use std::{fmt, hash};
use allocator_api2::alloc::Allocator;
use allocator_api2::vec::Vec;
use crate::dbg_assert;
use super::Id;

// ----------- Generational Array ----------- //

// free-list vs. remove_swap
/// size = 3 thick_ptr + (T + U) * item_count + 4 U * deletion_count, where U defaults to u32
/// so overhead = 2 thick_ptr + U * item_count + 4 U * deletion_count
/// If T is small and deletion_count is large, then this is a lot of overhead.
/// If T and item_count is small, then this is a lot of overhead.
/// 
/// You can remove elements from this array without fear of invalidating indices 
/// to other elements. Only the last element gets moved and this fact is recorded in
/// self.generations and self.redirections, which is used adjust the internals of 
/// any indices that comes into contact with the array. 
/// 
/// small optimization: dont repeat len and capacity across self.data and self.generations
#[derive(Debug, Eq, PartialEq, Clone)] // Display, Index, IndexMut, IntoIterator
pub struct GenArray<T, A: Allocator + Clone> {
    data: Vec<T, A>,
    generations: Vec<U, A>,
    redirections: HashMap<GIdx<()>, GIdx<()>, DefaultHashBuilder, A>, // redirecting old idx to new idx. 
    #[cfg(debug_assertions)] id: Id<()>
}

impl<T, A: Allocator + Clone> GenArray<T, A> {
    #[inline] pub fn new_in(capacity: usize, alloc: A) -> Self { // from_vec_a arent useful, but from-vec is for compatibility
        GenArray { 
            data: Vec::with_capacity_in(capacity, alloc.clone()), 
            generations: Vec::with_capacity_in(capacity, alloc.clone()), 
            redirections: HashMap::with_capacity_in(capacity, alloc),
            #[cfg(debug_assertions)] id: Id::new(), 
        }
    }

    #[inline] pub fn from_vec(data: Vec<T, A>) -> Self { // from_vec_a arent useful, but from-vec is for compatibility
        let capacity = data.capacity();
        let alloc = data.allocator().clone();
        GenArray { 
            data, 
            generations: Vec::with_capacity_in(capacity, alloc.clone()), 
            redirections: HashMap::with_capacity_in(capacity, alloc),
            #[cfg(debug_assertions)] id: Id::new(),
        }
    }

    #[inline] pub fn push(&mut self, value: T) -> GIdx<T> { 
        dbg_assert!(self.data.len() as f64 <= overflow_safety * u32::MAX as f64, format!("
            The GenArray can only hold up to {overflow_safety} * U::MAX items. 
            To exceed this limit either change the safety factor {overflow_safety} or the type U.
            They can both be found inside the script that defines the GenArray.
        "));
        self.data.push(value);
        self.generations.push(0);
        self.make_idx(self.data.len() as U - 1, 0)
    }

    #[inline] pub fn remove(&mut self, idx: GIdx<T>) -> T {
        self.check_id(&idx); 
        let last_index = self.data.len() - 1;
        let last_gen = self.generations[last_index];
        self.redirections.insert(
            self.make_idx(last_index as U, last_gen), 
            self.make_idx(idx.raw.get(), idx.raw.get() + 1)
        );
        self.generations[idx.raw.get() as usize] += 1;
        self.generations[last_index] += 1;
        self.data.swap_remove(idx.raw.get() as usize)
    }

    #[inline] pub fn pop(&mut self) -> Option<T> { 
        self.generations[self.data.len() - 1] += 1;
        self.data.pop() 
    }
    // #[inline] pub fn get(&self, idx: &Idx<T>) -> &T             { self.check_id(idx.stack_id); &self.data[idx.raw] }
    // #[inline] pub fn set(&mut self, idx: &Idx<T>, value: T)     { self.check_id(idx.stack_id); self.data[idx.raw] = value; }
    // #[inline] pub fn get_x(&self, idx: &GIdx<T>) -> &T          { self.check_id(idx.stack_id); &self.data[idx.raw] }
    // #[inline] pub fn set_x(&mut self, idx: &GIdx<T>, value: T) { self.check_id(idx.stack_id); self.data[idx.raw] = value; }

    #[inline] pub fn iter(&self) -> Iter<'_, T>                 { self.data.iter() }
    #[inline] pub fn iter_mut(&mut self) -> IterMut<'_, T>      { self.data.iter_mut() }
    // #[inline] fn check_id(&self, array_id: Id<()>) {dbg_assert!(array_id == self.id, "You put the index into the wrong generational array.");}
    
    #[inline] fn check_id(&self, idx: &GIdx<T>) {
        dbg_assert!(idx.array_id == self.id, "You put the index into the wrong generational array.");
        if self.generations[idx.raw.get() as usize] != idx.generation.get() { self.fix_id(idx) }
    }
    #[inline(never)] #[cold] fn fix_id(&self, idx: &GIdx<T>) {
        let typeless_idx = &self.make_idx(idx.raw.get(), idx.generation.get());
        let new_idx = self.redirections.get(typeless_idx).expect(
            &format!("Item at index {} has been removed.", idx.raw.get())
        ); // evt. allow err_handling via try_get
        idx.raw.set(new_idx.raw.get());
        idx.generation.set(new_idx.generation.get());
        // dbg_assert!("Item at index {} has been moved. nb: in release this error is prevent at a small performance cost", idx.raw.get());
    }

    #[inline] pub unsafe fn to_vec(self) -> Vec<T, A> { self.data }

    #[inline] fn make_idx<T2>(&self, raw: U, generation: U) -> GIdx<T2> { 
        GIdx::<T2> { raw: Cell::new(raw), generation: Cell::new(generation), _mark: PhantomData, #[cfg(debug_assertions)] array_id: self.id }
    }
    // #[inline] fn typeless(&self, idx: GIdx<T>) -> GIdx<()> { 
    //     GIdx::<T2> { raw: Cell::new(raw), generation: Cell::new(generation), _mark: PhantomData, #[cfg(debug_assertions)] array_id: self.id }
    // }

    #[inline] pub fn len(&self) -> usize { self.data.len() }
    
}

// --- Traits --- //
impl<T: Sized + fmt::Display, A: Allocator + Clone> fmt::Display for GenArray<T, A> { 
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self) } 
}

impl<T, A: Allocator + Clone> Index<&GIdx<T>> for GenArray<T, A> { type Output = T;
    #[inline] fn index(&self, idx: &GIdx<T>) -> &Self::Output { 
        self.check_id(idx); 
        &self.data[idx.raw.get() as usize] 
}}
impl<T, A: Allocator + Clone> IndexMut<&GIdx<T>> for GenArray<T, A> {
    #[inline] fn index_mut(&mut self, idx: &GIdx<T>) -> &mut Self::Output { 
        self.check_id(idx); 
        &mut self.data[idx.raw.get() as usize] 
}}

impl<T, A: Allocator + Clone> IntoIterator for GenArray<T, A> {
    type Item = T;
    type IntoIter = allocator_api2::vec::IntoIter<Self::Item, A>;
    #[inline] fn into_iter(self) -> Self::IntoIter { self.data.into_iter() }
}



// macro stack![1, 2, 3, 4]




// ----------- Idx ----------- //

#[derive(Debug, Eq)] // PartialEq, Clone, Display, Hash
pub struct GIdx<T: ?Sized> { raw: Cell<U>, generation: Cell<U>, 
    _mark: PhantomData<T>, #[cfg(debug_assertions)] array_id: Id<()>
}

impl<T: ?Sized> GIdx<T> { 
    pub fn to_int(&self) -> U { self.raw.get() } 
}

// trivial impl (bitwise copy, inhirited display and call hash) (these have broader domain than #[derive(...)])
impl<T: ?Sized> Clone for GIdx<T> { fn clone(&self) -> Self { 
    GIdx { raw: self.raw.clone(), generation: self.generation.clone(), 
        _mark: PhantomData, #[cfg(debug_assertions)] array_id: self.array_id
}}} 
impl<T: ?Sized> PartialEq for GIdx<T> { fn eq(&self, other: &Self) -> bool { self.raw == other.raw && self.generation == other.generation } }
impl<T: ?Sized> hash::Hash for GIdx<T> { fn hash<H: hash::Hasher>(&self, state: &mut H) { self.raw.get().hash(state); } }
impl<T: ?Sized + fmt::Display> fmt::Display for GIdx<T> { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self) } }


type U = u32;
const overflow_safety: f64 = 0.01;
