
// #[macro_use] pub mod debug_macros;
pub mod debug_macros;
pub mod err;
pub mod arena; 
// pub mod idx_;

pub use arena::*; 
// pub use debug_macros::*;
// pub use idx_::*;
// pub use crate::tools::debug_macros::*;
// pub use crate::tools::err::query_error;
// pub use debug_macros::*;
// pub use err::query_error;

/* 
cut out the allocator api and bumpalo
build growable array
build MemoryPool of type or size 
MemoryPool of type is essentially a DynArray. Evt. impl as such. Use generational idx
hashMap ???
use pools for comparing ids

*/
