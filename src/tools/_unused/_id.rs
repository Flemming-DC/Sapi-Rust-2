
use std::{fmt, hash};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::marker::PhantomData;

static max_id: AtomicUsize = AtomicUsize::new(0);
 
#[derive(Debug, Eq)] // PartialEq, Clone, Copy, Display, Hash
pub struct Id<T: ?Sized> { raw: usize, _mark: PhantomData<T> }

impl<T: ?Sized> Id<T> {
    pub fn new() -> Self {
        let value = max_id.fetch_add(1, Ordering::Relaxed);
        Id { raw: value, _mark: PhantomData }
    }
}

// trivial impl (bitwise copy, inhirited display and call hash) (these have broader domain than #[derive(...)])
impl<T: ?Sized> Copy for Id<T> {}
impl<T: ?Sized> Clone for Id<T> { fn clone(&self) -> Self { *self } } 
impl<T: ?Sized> PartialEq for Id<T> { fn eq(&self, other: &Self) -> bool { self.raw == other.raw } }
impl<T: ?Sized> hash::Hash for Id<T> { fn hash<H: hash::Hasher>(&self, state: &mut H) { self.raw.hash(state); } }
impl<T: ?Sized + fmt::Display> fmt::Display for Id<T> { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self) } }

// pub trait Identifiable { fn get_id(&self) -> Id<()>; }
