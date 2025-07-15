// use std::{alloc::{alloc, dealloc, handle_alloc_error, Layout}, cell::Cell, fmt, hash, ptr::NonNull};
// use allocator_api2::alloc::{AllocError, Allocator, Global};
// use crate::{dbg_assert, P};
// use super::Id;

// /// so far this code is just sandbox
// /// example
// /// let pool = MemoryPool::<5> {};

// // slot_size vs type that can be enum
// struct MemoryPool<const slot_size: usize> { // A: Allocator
//     // data e.g. a restricted array, or just a Vec<T, A> or a chunk list
//     // free-list 
//     // next_ptr (in case that cannot be readily retrieved)
//     // capacity, dbg max_consumption_ratio
// }
// // I think I prefer a chunk list of optional T's
// // maybe 

// /*
// fn new(capacity, dbg_max_consumption_ratio) -> Self

// fn make<T>() -> some sort of reference e.g. ResArray idx
//     dbg_assert size_of T <= slot_size
//     scan free list to get the first unoccupied slot and mark it as occupied
//     if no unoccupied slot found then use next slot and handle capacity overflow
//     init data
//     return data

// fn free<T>(t: T)
//     mark slot as empty
//     call drop


// unsafe impl Allocator for &MemoryPool {
//     fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        
//     }
    
//     unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
//         // We leak the memory.
//     }

// }
// */


// /* 
// evt. let pool and arena draw from a parent allocator
// evt. let the Arena and Pool resize dynamically by a chunk-list
// evt. create Chunk-list as a seperate data structure
// evt. introduce a make_with method akin to bumpalo in order to avoid expensive double initialization. 
// */



