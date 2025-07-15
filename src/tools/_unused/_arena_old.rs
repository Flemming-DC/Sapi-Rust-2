// use std::{alloc::{alloc, dealloc, handle_alloc_error, Layout}, cell::Cell, fmt, hash, ptr::NonNull};
// use allocator_api2::alloc::{AllocError, Allocator, Global};
// use crate::{dbg_assert, P};
// use super::Id;

// // ------- Arena ------- //

// #[derive(Debug)] // PartialEq, Display
// pub struct Arena {
//     base: *mut u8, // only mutable for the sake of dropping
//     offset_: Cell<usize>,
//     capacity: usize,
//     parent: Option<*mut Arena>, // None means its the root Arena. This cannot be a Pia, since the root is globally allocated.
//     #[cfg(debug_assertions)] id: Id<Arena>,
//     #[cfg(debug_assertions)] has_emitted_scope: Cell<bool>,
//     #[cfg(debug_assertions)] max_consumption_ratio: f64,
// }


// impl Arena {
//     pub fn new(capacity: usize, dbg_max_consumption_ratio: f64) -> Self {
//         let layout = Layout::from_size_align(capacity, 1).unwrap();
//         let base = unsafe { alloc(layout) };
//         if base.is_null() { handle_alloc_error(layout); }

//         return Arena {base, offset_: Cell::new(0), capacity, parent: None, 
//             #[cfg(debug_assertions)] id: Id::new(), 
//             #[cfg(debug_assertions)] has_emitted_scope: Cell::new(false), 
//             #[cfg(debug_assertions)] max_consumption_ratio: dbg_max_consumption_ratio
//         }
//     }
//     // pub fn new_in<A: Allocator>(allocator: A, capacity: usize, dbg_max_consumption_ratio: f64) -> Self {
//     //     let layout = Layout::from_size_align(capacity, 1).unwrap();
//     //     let mut base = allocator.allocate(layout).unwrap();
//     //     let base = (&mut (*unsafe {base.as_mut()})[0]) as *mut u8;

//     //     return Arena {base, offset_: Cell::new(0), capacity, parent: None, 
//     //         #[cfg(debug_assertions)] id: Id::new(), #[cfg(debug_assertions)] max_consumption_ratio: dbg_max_consumption_ratio
//     //     }
//     // }

//     pub fn new_scope(&mut self) -> Box<Arena> {
//         dbg_assert!(!self.has_emitted_scope.get(), "You cannot create a scop in this arena, before the previous scope gets dropped.");
//         let parent = Some(self as *mut Arena);
//         let scope = self.temp(Arena {
//             base: (self.base as usize + self.offset_.get()) as *mut u8, 
//             offset_: Cell::new(0), 
//             capacity: self.capacity - self.offset_.get(), 
//             parent: parent, 
//             #[cfg(debug_assertions)] id: Id::new(), 
//             #[cfg(debug_assertions)] has_emitted_scope: Cell::new(false), 
//             #[cfg(debug_assertions)] max_consumption_ratio: self.max_consumption_ratio,
//         }) as *mut Arena;
//         #[cfg(debug_assertions)] self.has_emitted_scope.set(true);
//         return unsafe { Box::from_raw(scope) };
//     }

//     #[inline] pub fn perm<T>(&self, value: T) -> Pia<T> {
//         let ptr = self.temp::<T>(value);
//         return Pia {ptr: ptr as *mut T, #[cfg(debug_assertions)] arena_id: self.id}
//     }
//     // In a sense this mutates self, since it mutates the arenas buffered memory.
//     // However, it doesn't invalidate self, so its ok.
//     #[inline] pub fn temp<'a, T>(&'a self, value: T) -> &'a mut T {
//         let ptr_unaligned = self.base as usize + self.offset_.get();
//         let ptr = ((ptr_unaligned + align_of::<T>() - 1) & !(align_of::<T>() - 1)) as *mut T; // align must be a power of 2
//         self.offset_.set(self.offset_.get() + size_of::<T>());

//         dbg_assert!((self.offset_.get() as f64) < self.capacity as f64 * self.max_consumption_ratio, format!("
//             The Arena's memory consumption exeeds `capacity * max_consumption_ratio`, 
//             which is reported as a bug in debug mode. 
//             Adjust the capacity to be at least {}.

//             nb: This check is not performed in release build. Instead the arena would ask 
//             if `self.offset_ > self.capacity` then fallback on the general heap.
//             ", self.offset_.get()
//         ));
//         if self.offset_.get() > self.capacity { return Box::leak(Box::new(value)); } // fallback to the normal heap. 
       
//         unsafe {
//             ptr.write(value); // here ptr becomes valid
//             return &mut* ptr
//         };
//     }
// } 


// impl Drop for Arena {
//     fn drop(&mut self) {
//         match self.parent {
//             Some(parent) => unsafe { 
//                 (*parent).offset_.set(self.base as usize - (*parent).base as usize); 
//                 #[cfg(debug_assertions)] self.has_emitted_scope.set(false);
//             },
//             None => {
//                 let layout = Layout::from_size_align(self.capacity, 1).unwrap();
//                 unsafe { dealloc(self.base, layout); }
//             }
//         }
//     }
// }
// // impl PartialEq for Arena {
// //     fn eq(&self, other: &Self) -> bool {
// //         self.id == other.id // only works in debug
// //     }
// // }
// // trivial impl (inhirited display)
// impl fmt::Display for Arena { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self) } }

// unsafe impl Allocator for &Arena {
//     fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
//         let ptr_unaligned = self.base as usize + self.offset_.get();
//         let ptr = ((ptr_unaligned + layout.align() - 1) & !(layout.align() - 1)) as *mut u8; // align must be a power of 2
//         self.offset_.set(self.offset_.get() + layout.size());
        
//         dbg_assert!((self.offset_.get() as f64) < self.capacity as f64 * self.max_consumption_ratio, format!("
//             The Arena's memory consumption exeeds `capacity * max_consumption_ratio`, 
//             which is reported as a bug in debug mode. 
//             Adjust the capacity to be at least {}.

//             nb: This check is not performed in release build. Instead the arena would ask 
//             if `self.offset_ > self.capacity` then fallback on the general heap.
//             ", self.offset_.get()
//         ));
//         if self.offset_.get() > self.capacity { return Global.allocate(layout) } // fallback to the normal heap. 
       
//         let nn = NonNull::new(ptr).unwrap();
//         return Ok(NonNull::slice_from_raw_parts(nn, layout.size()));
//     }
    
//     unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
//         // We leak the memory.
//     }

// }


// // ------- Pia ------- //

// /// Pointer into arena. Avoids lifetime spam. Use dereference it with the get, get_mut or get_raw methods.
// #[derive(Debug, PartialEq)] // Display, Copy, Clone, Hash
// pub struct Pia<T: ?Sized> { 
//     ptr: *mut T,
//     #[cfg(debug_assertions)] arena_id: Id<Arena>,
// }


// impl<T: ?Sized> Pia<T> {
//     #[inline] pub fn get<'a>(&self, arena: &'a Arena) -> &'a T {
//         dbg_assert!(self.arena_id == arena.id, format!("
//             You must put provide the Pia with the Arena that it points into.
//             Expected Arena with {:?}, received {:?}.
//             nb: This check is omitted in release builds. 
//             ", self.arena_id, arena.id
//         ));
//         return unsafe { &*self.ptr };
//     }
//     #[inline] pub fn get_mut<'a>(&mut self, arena: &'a mut Arena) -> &'a mut T {
//         dbg_assert!(self.arena_id == arena.id, format!("
//             You must put provide the Pia with the Arena that it points into.
//             Expected Arena with id {:?}, received {:?}.
//             nb: This check is omitted in release builds. 
//             ", self.arena_id, arena.id
//         ));
//         return unsafe { &mut *self.ptr };
//     }
//     #[inline] pub fn get_raw<'a>(&mut self) -> *mut T {
//         return self.ptr;
//     }
// } 


// // trivial impl (bitwise copy, inhirited display and call hash)
// impl<T: ?Sized> Copy for Pia<T> {}
// impl<T: ?Sized> Clone for Pia<T> { fn clone(&self) -> Self { *self } } 
// impl<T: ?Sized + fmt::Display> fmt::Display for Pia<T> { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self) } }
// impl<T: ?Sized> hash::Hash for Pia<T> { fn hash<H: hash::Hasher>(&self, state: &mut H) { self.ptr.hash(state); } }




