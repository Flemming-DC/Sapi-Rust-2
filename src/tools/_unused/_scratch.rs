// use crate::dbg_assert;
// use super::Arena;
// use std::{cell::RefCell, rc::{Rc, Weak}, thread::LocalKey};


// thread_local! {
//     static stratches: /*LocalKey*/RefCell<Vec<Box<Arena>>> = RefCell::new(Vec::new()); // vec![Arena::new(1_048_576, 0.1)]);
// }

// struct Scratch { arena_idx: usize}

// impl Scratch {
//     pub fn new() -> Self {
//         return stratches.with(|s1| { 
//             let mut s2 = s1.borrow_mut();

//             let new = match s2.last_mut() {
//                 None => Box::new(Arena::new(1_048_576, 0.1)),
//                 Some(prev) => prev.new_scope(),
//             };
//             s2.push(new);
//             return Scratch { arena_idx: s2.len() };
//         });
//     }

//     #[inline] pub fn alloc<'a, T>(&'a mut self, value: T) -> &'a mut T { 
//         return stratches.with(|s1| { 
//             let arena = &mut*s1.borrow_mut()[self.arena_idx];
//             let value = arena.temp(value);

//             return unsafe { &mut* (value as *mut T) };
//         }); 
//     }    
// }

// impl Drop for Scratch {
//     fn drop(&mut self) { 
//         stratches.with(|s1| { 
//             s1.borrow_mut().pop();
//         }); 
//     }
// }


// // struct Cleaner<T, F: FnMut() -> ()> { value: T, cleanup: F}
// // impl<T, F: FnMut() -> ()> Drop for Cleaner<T, F> {
// //     fn drop(&mut self) { (self.cleanup)(); }
// // }
// // denote the type like this Cleaner<T, impl FnMut() -> ()>

