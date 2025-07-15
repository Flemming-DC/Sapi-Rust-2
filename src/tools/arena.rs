use std::{cell::RefCell, fmt, hash, ops::{Deref, DerefMut}};
use std::{alloc::Layout, ptr::NonNull};
use allocator_api2::alloc::{AllocError, Allocator};
use allocator_api2::vec::Vec;
use crate::vendor::string_alloc::String;
use hashbrown::HashMap;
use hashbrown::DefaultHashBuilder;
use crate::vendor::bumpalo::Bump;
use extend::ext;

thread_local! {
    static arena_: /*LocalKey*/RefCell<Bump> = RefCell::new(Bump::with_capacity(30_000));
    static generation: /*LocalKey*/RefCell<usize> = RefCell::new(0);
}

// returns Pia<T> and Pia is modified to do generation check and to impl deref
// #[inline] pub fn alloc<T>(f: impl FnOnce() -> T) -> Pia<T> {
//     let ptr = get_arena().alloc_with(f) as *mut T;
//     Pia {ptr: ptr, #[cfg(debug_assertions)] gen: get_generation()}
// }

pub fn alloc_array<T: Clone>(count: usize, f: impl FnMut(usize) -> T) -> Pia<[T]> {
    let ptr = get_arena().alloc_slice_fill_with(count, f);
    Pia {ptr: ptr, #[cfg(debug_assertions)] gen: get_generation()}
}

// pub fn alloc_array_const<T: Clone>(count: usize, t: &T) -> Pia<[T]> {
//     let ptr = get_arena().alloc_slice_fill_clone(count, t);
//     Pia {ptr: ptr, #[cfg(debug_assertions)] gen: get_generation()}
// }

// pub fn alloc_array_default<T: Clone + Default>(count: usize) -> Pia<[T]> {
//     let ptr = get_arena().alloc_slice_fill_default(count);
//     Pia {ptr: ptr, #[cfg(debug_assertions)] gen: get_generation()}
// }

pub fn reset() { 
    arena_.with_borrow_mut(|a| { 
        a.reset();
    });
}

pub fn allocated_bytes() -> usize { 
    arena_.with_borrow(|a| {
        a.allocated_bytes()
    })
}


#[derive(Eq, PartialOrd)] // Debug, Display, Copy, Clone, Hash, PartialEq
pub struct Pia<T: ?Sized> { 
    ptr: *mut T,
    #[cfg(debug_assertions)] gen: usize,
}

// impl<T: ?Sized + Clone> Pia<T> {
//     pub fn deep_copy(self) -> Self { 
//         alloc(|| unsafe {(*self.ptr).clone()})
//     }
// }
// impl<T: ?Sized + Clone> Pia<[T]> {
//     pub fn deep_copy_array(self) -> Self { 
//         alloc_array(self.ptr.len(), |i| unsafe {(*self.ptr)[i].clone()})
//     }
//     pub fn pia_slice(&self, from: usize, to: usize) -> Pia<[T]> {
//         Pia {
//             ptr: &self[from..to] as *const [T] as *mut [T], 
//             #[cfg(debug_assertions)] gen: get_generation()
//         }
//     }
// }


// The derefered version of Pia must not live passed an arena reset.
// If you wish to eliminate the possibility of lifetime bugs entirely, 
// then use the pia.with(|internal| ...) idiom.
impl<T: ?Sized> Deref for Pia<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        #[cfg(debug_assertions)] assert!(self.gen == get_generation(), "The arena has been reset, so this pointer has becomes invalid.");
        unsafe{&*self.ptr}
    }
}
impl<T: ?Sized> DerefMut for Pia<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        #[cfg(debug_assertions)] assert!(self.gen == get_generation(), "The arena has been reset, so this pointer has becomes invalid.");
        unsafe{&mut* self.ptr}
    }
}

// trivial impl (bitwise copy, inhirited display and call hash)
// impl<T: ?Sized + fmt::Display> fmt::Display for Pia<T> { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self) } }
impl<T: ?Sized> Clone for Pia<T> { fn clone(&self) -> Self { *self } } 
impl<T: ?Sized> Copy for Pia<T> {}
impl<T: ?Sized + PartialEq> PartialEq for Pia<T> { fn eq(&self, other: &Self) -> bool { **self == **other } }
impl<T: ?Sized> hash::Hash for Pia<T> { fn hash<H: hash::Hasher>(&self, state: &mut H) { self.ptr.hash(state); } }

impl<T: fmt::Debug + ?Sized> fmt::Debug for Pia<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Pia {:?}", unsafe{&* self.ptr})
    }
}
impl<T: fmt::Display + ?Sized> fmt::Display for Pia<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", unsafe{&* self.ptr})
    }
}

#[ext] pub impl<T> &T {
    #[inline] fn pia(self) -> Pia<T> {
        Pia {
            ptr: self as *const T as *mut T, 
            #[cfg(debug_assertions)] gen: get_generation()
        }
    }
}
#[ext] pub impl<T> VecA<T> {
    #[inline] fn as_pia(&self) -> Pia<[T]> {
        Pia {
            ptr: self.as_slice() as *const [T] as *mut [T], 
            #[cfg(debug_assertions)] gen: get_generation()
        }
    }
}
#[ext] pub impl<T: Clone> [T] {
    #[inline] fn as_vec_a(&self) -> VecA<T> {
        let mut out = new_vec(self.len());
        for a in self {out.push(a.clone());}
        out
    }
    #[inline] fn as_pia(&self) -> Pia<[T]> {
        alloc_array(self.len(), |i| self[i].clone())
    }
    // fn item_pia(&self, i: usize) -> Pia<T> {
    //     Pia {
    //         ptr: &self[i] as *const T as *mut T, 
    //         #[cfg(debug_assertions)] gen: get_generation()
    //     }
    // }
}
#[ext] pub impl StringA {
    #[inline] fn as_pia(&self) -> Pia<str> {
        Pia {
            ptr: &**self as *const str as *mut str, 
            #[cfg(debug_assertions)] gen: get_generation()
        }
    }
}
#[ext] pub impl std::string::String {
    #[inline] fn as_string_a(&self) -> StringA {
        StringA::from_str_in(self, arena_gen())
    }
    #[inline] fn as_pia(&self) -> Pia<str> { self.as_string_a().as_pia() }
}
#[ext] pub impl &str {
    #[inline] fn as_string_a(&self) -> StringA {
        StringA::from_str_in(self, arena_gen())
    }
    #[inline] fn as_pia(&self) -> Pia<str> { self.as_string_a().as_pia() }
}
impl Pia<str> {
    #[inline] pub fn as_string_a(&self) -> StringA {
        StringA::from_str_in(self, arena_gen())
    }
}

#[inline] fn get_arena() -> &'static Bump { 
    // cast away the false lifetime
    arena_.with(|a| { unsafe { &* (a.as_ptr() as *const Bump) } })
}
#[inline] fn get_generation() -> usize { generation.with_borrow(|g| { *g }) }

// This stores a reference to on Arena along with the generation when the reference was taken.
#[derive(Debug, Clone)] // other traits
pub struct ArenaGen {arena: &'static Bump, generation: usize} 


unsafe impl Allocator for ArenaGen {
    #[inline] fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        if self.generation != get_generation() { return Err(AllocError); }
        return self.arena.allocate(layout);
    }
    
    #[inline] unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: Layout) {
        // We leak the memory.
    }

}

#[inline] fn arena_gen() -> ArenaGen {
    ArenaGen {arena: get_arena(), generation: get_generation()}
}

#[inline] pub fn new_vec<T>(cap: usize) -> VecA<T> {
    Vec::with_capacity_in(cap, arena_gen())
}
#[inline] pub fn new_string(cap: usize) -> StringA {
    String::with_capacity_in(cap, arena_gen())
}
#[inline] pub fn new_map<K, V>(cap: usize) -> HashMapA<K, V> {
    HashMap::with_capacity_in(cap, arena_gen())
}

// if desired, add more growable collections.


pub type VecA<T> = Vec<T, ArenaGen>;
pub type StringA = String<ArenaGen>;
pub type HashMapA<K, V> = HashMap<K, V, DefaultHashBuilder, ArenaGen>;

