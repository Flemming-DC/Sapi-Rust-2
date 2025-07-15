use std::{collections::HashMap, fmt};
use crate::tools::{HashMapA, Pia, RefstrExt, VecA};
use super::{PrefixedTab, TabIdx, Table, TokIdx};


#[derive(Debug, PartialEq)]
pub enum Semantic {
    Selection {root: PrefixedTab, path: VecA<Relata>, resolved_cols: VecA<ResolvedColumn>}, 
}


#[derive(PartialEq, Clone, Debug)]
pub struct Relata { // table related to root
    pub tab: Pia<Table>,
    pub name: Pia<str>, 
    pub parent: Option<Pia<Relata>>, 
    pub pk: Pia<str>, // evt. optimize to Pia<str>
    pub fk_to_parent: Option<Pia<str>>, 
    pub fk_to_children: HashMapA<TabIdx, Pia<str>>,
    pub arrow_tabs: Vec<Pia<str>>, // evt. make this a Vec<TokIdx> to align better with ResolvedColumn.prefixed_col
    pub needs_skema_prefix: bool,
}


#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ResolvedColumn {
    pub table: Pia<str>, // None means the table either already specified or doesn't need not specified
    pub arrow_tabs: Vec<Pia<str>>,
    pub start_idx: TokIdx,
    pub end_idx: TokIdx,

}



// impl fmt::Debug for Relata {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         let p = &*self.parent.map_or("None".as_pia(), |p| p.name);
//         return write!(f, "Relata '{}' parent: '{}'", self.name, p);
//     }
// }

