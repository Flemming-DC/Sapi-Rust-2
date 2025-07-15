use crate::tools::err::Fallible;
use crate::tools::{arena, stdstringStringExt, HashMapA, Pia, SliceOfTExt, StringAExt, VecA};
use crate::StringA;
use super::*;
use super::Node::*;

/*
any query
    Pass has already collected identifiers (with any provided prefix) in parser.
    resolve identifers into schema.tab.col or report amguity / unrecognized errors
selection
    build tree
    find path    
insertion
    get keys
*/

type RelataIdx = usize;

pub fn analyze(tok_data: &TokData, ast: &Ast, model: &DataModel) -> Fallible<VecA<Semantic>> {
    // resolve identifiers (evt. do this per query)
    let mut query_semantics: VecA<Semantic> = arena::new_vec(1);
    for (query_idx, names) in &ast.names_stack {
        let semantics = match &ast.nodes[query_idx.raw] {
            Selection {root} => select_analyzer::analyze_select(names, *root, &tok_data, &model)?,
            Insertion {root} => insert_analyzer::analyze_insert(names, *root, &tok_data, &model)?,
            // Updation 
            // Deletion 
            // Upsertion aka. Overwrite
            _ => unreachable!(),
        };
        query_semantics.push(semantics);
    }
    Ok(query_semantics)
}

