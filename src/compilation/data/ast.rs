use std::hash::{Hash, Hasher};
use std::fmt;
use crate::tools::arena::*;
use super::TokIdx;
use super::*;

#[derive(Debug, Eq, PartialEq, Clone, Hash, PartialOrd)] // Display
pub enum Node {
    Script {queries: Pia<[NodeIdx]>},
    // each query has a full_names subset (not a contigous slice) and a optional root
    Selection {root: PrefixedTab}, // query_idx aren't used
    Insertion {root: PrefixedTab}, // query_idx aren't used
    // Updation {name_idx: NodeIdx, root: VecA<TokIdx>}, 
    // Deletion {name_idx: NodeIdx, root: VecA<TokIdx>}, 
    // Upsertion aka. Overwrite
    // WithQuery, // with is treated as a subscript
}
/*
enum Name {
    Single(TokIdx), // or pia<str>
    Dual(TokIdx, TokIdx),
    Triple(TokIdx, TokIdx, TokIdx),
    Resolver(Pia<[TokIdx]>),
}
*/

pub type Name = (Pia<[TokIdx]>, bool); // bool = is_arrow_prefix
#[derive(Eq, Clone, Copy, PartialOrd)] // Display, Debug, PartialEq, Hash
pub struct NodeIdx {pub raw: usize, 
    #[cfg(debug_assertions)] nodes: Pia<[Node]>,
    #[cfg(debug_assertions)] names_stack: Pia<[(NodeIdx, VecA<Name>)]>, 
}
// pub type NodeIdx = usize;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd)]
pub struct PrefixedTab {pub schema: Option<TokIdx>, pub tab: TokIdx, pub across_schemas: bool}

#[derive(Debug)]
pub struct Ast {
    pub nodes: VecA<Node>, 
    // map query -> list of qualified_idents.
    pub names_stack: VecA<(NodeIdx, VecA<Name>)>, 
}

// ----------- functions ----------- //

impl NodeIdx {
    // pub fn uninitialized() -> Self { NodeIdx {raw: usize::MAX, #[cfg(debug_assertions)] ast: ptr::null::<Ast>() } }
    pub fn new(ast: &Ast, raw: usize) -> Self { 
        dbg_assert!(raw == usize::MAX || raw < ast.nodes.len());
        NodeIdx {raw, 
            #[cfg(debug_assertions)] nodes: ast.nodes.as_pia(),
            #[cfg(debug_assertions)] names_stack: ast.names_stack.as_pia(),
        } 
    }
}

// ----------- traits ----------- //

impl fmt::Display for Node { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{:?}", self) } }
impl fmt::Display for PrefixedTab { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{:?}", self) } }
impl fmt::Display for Ast { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{:?}", self) } }
impl PartialEq for NodeIdx { fn eq(&self, other: &Self) -> bool { self.raw == other.raw } }
impl Hash for NodeIdx { fn hash<H: Hasher>(&self, state: &mut H) { self.raw.hash(state); } }

impl fmt::Debug for NodeIdx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.raw == usize::MAX {return write!(f, "Unititialized NodeIdx");}
        #[cfg(debug_assertions)] {
            dbg_assert!(self.raw < self.nodes.len());
            let node = &self.nodes[self.raw];

            for (query_idx, names) in &*self.names_stack {
                if query_idx.raw == self.raw { 
                    return write!(f, "{}\n    with names {:?}\n", node, names);
                }
            }
            return write!(f, "{}\n    with no names\n", node); 
        }
        #[cfg(not(debug_assertions))] write!(f, "NodeIdx {}", self.raw)
    }
}

