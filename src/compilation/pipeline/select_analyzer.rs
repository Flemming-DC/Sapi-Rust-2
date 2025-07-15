use std::collections::VecDeque;
use hashbrown::{HashMap, HashSet};
use proptest::bits::usize;
use proptest::string;

use crate::tools::err::Fallible;
use crate::tools::{arena, stdstringStringExt, HashMapA, Pia, RefTExt, RefstrExt, SliceOfTExt, StringAExt, VecA};
use crate::StringA;
use super::*;
use super::Node::*;


static log: bool = true;


#[derive(Debug, PartialEq, Clone)]
struct ProtoRelata { // table related to root
    tab: Pia<Table>,
    parents: VecA<Pia<Table>>,
    is_candidate: bool,
    generation: u32,
    // pub name: Pia<str>, 
    // children 
    // pub pk: Pia<str>, // evt. optimize to Pia<str>
    // pub fk_to_parent: Option<Pia<str>>, 
    // pub fk_to_children: HashMap<TabIdx, String>,
}

#[derive(Debug, PartialEq, Clone)]
struct RelataSplit { // table related to root
    tab: Pia<Table>,
    parent: Option<Pia<Table>>,
    is_candidate: bool,
    arrow_line_names: Vec<Pia<str>>,
}
type ArrowLine = VecA<TokIdx>;


pub fn analyze_select(names: &[Name], root_ast: PrefixedTab, tok_data: &TokData, model: &DataModel) -> Fallible<Semantic> {
    
    let cols: Vec<&Name> = names.into_iter().filter(
        |(n, _)| n.last().unwrap() != &root_ast.tab).collect();
    let root = get_root(tok_data, model, root_ast)?;
    let candidates = find_candidates(&cols, model, tok_data, root, root_ast.across_schemas)?;

    let proto_relatas: VecA<ProtoRelata> = make_proto_tree(root, &model, candidates);
    let relata_tree: VecA<RelataSplit> = split_relata_tree(proto_relatas, model, &cols, tok_data)?;
    let path: VecA<RelataSplit> = prune_relata_tree(&relata_tree, &model, &cols, tok_data)?;
    let mut path: VecA<Relata> = equip_with_info(path, model);
    // consider making some form of error check

    let resolved_cols: VecA<ResolvedColumn> = find_resolved_columns(&mut path, &cols, model, tok_data);

    Ok(Semantic::Selection {root: root_ast, path, resolved_cols})
}


fn get_root<'a>(tok_data: &TokData, model: &'a DataModel, root_indices: PrefixedTab) -> Fallible<&'a Table> {
    let root_line = tok_data.line(root_indices.tab);
    let root_schema = if let Some(root_schema_idx) = root_indices.schema {
        Some(&**tok_data.texts(root_schema_idx))
    } else {None};
    let root_tab_name = tok_data.texts(root_indices.tab).clone();

    let root_tab = model.table_by_prefixed_tab_name(root_line, root_schema, &root_tab_name)?;
    Ok(root_tab)
}

fn find_candidates(cols: &[&Name], model: &DataModel, tok_data: &TokData, root: &Table, join_across_schemas: bool
) -> Fallible<VecA<(TabIdx, Pia<[Pia<str>]>)>> {
    
    let mut candidates: VecA<(TabIdx, Pia<[Pia<str>]>)> = arena::new_vec(0);
    for (prefixed_col, has_arrow) in cols {
        let mut arrow_tabs = arena::new_vec(0);
        if *has_arrow {
            let cands_for_col = model.candidates_for_arrow_resolution(tok_data, prefixed_col)?;
            for (i, tok) in prefixed_col[0..prefixed_col.len() - 1].iter().enumerate() {
                arrow_tabs.push(tok_data.texts(*tok).as_pia());
            }
            for &cand in &*cands_for_col {
                if !join_across_schemas && cand.skema != root.skema {return query_error!(
                    tok_data.line(prefixed_col[0]), "
                    Cannot join across schemas, unless explicitly requested. 
                    Ergo {}.{} won't be joined with {}.{}. 
                    hint: add the `across schemas` clause. `from {}... across schemas`.
                    ", cand.skema, cand.name , root.skema, root.name, root.name
                )};
                candidates.push((cand.idx, arrow_tabs.as_pia()));
            }
            
        } else {
            let ref_tab = model.table_by_prefixed_column(tok_data, prefixed_col)?;
            if !join_across_schemas && ref_tab.skema != root.skema { return query_error!(
                tok_data.line(prefixed_col[0]), "
                Cannot join across schemas, unless explicitly requested. 
                Ergo {}.{} won't be joined with {}.{}. 
                hint: add the `across schemas` clause. `from {}... across schemas`.
                ", ref_tab.skema, ref_tab.name , root.skema, root.name, root.name
            )};
            candidates.push((ref_tab.idx, [].as_pia()));
        }
    }
    if log {dbg!(&candidates);}
    Ok(candidates)
}





fn make_proto_tree(root: &Table, model: &DataModel, mut candidates: VecA<(TabIdx, Pia<[Pia<str>]>)>
) -> VecA<ProtoRelata> {
    let mut relatas_1 = arena::new_vec(candidates.len());
    let mut tab_to_rel_idx = arena::new_map(candidates.len());
    let mut queue = VecDeque::new();
    queue.push_back(root);
    let candidates_2 = candidates.clone();

    let i = candidates.iter().position(|(c, at)| c == &root.idx);
    let is_candidate = i != None;
    let arrow_tabs = i.map_or([].as_pia(), |i| candidates.remove(i).1);

    // let is_candidate = candidates.iter().any(|(c, at)| c == &root.idx);
    // let arrow_tabs = candidates.remove(&root.idx).unwrap_or([].as_pia());
    tab_to_rel_idx.insert(root.idx, 0);
    relatas_1.push(ProtoRelata {tab: root.pia(), parents: [].as_vec_a(), is_candidate, generation: 0});
    let mut max_gen = 0;
    while let Some(parent) = queue.pop_front() {
        let parent_generation = relatas_1[tab_to_rel_idx[&parent.idx]].generation;
        // this generation check ensures that we process all or no members of a each generation
        if candidates.is_empty() && parent_generation > max_gen {break;} 
        for &tab in &*model.related_tables(parent) {
            if relatas_1.iter().any(|r| 
                // (tab, parent) or reverse is already included
                (*r.tab == *tab && r.parents.contains(&parent.pia())) ||
                (*r.tab == *parent && r.parents.contains(&tab.pia()))
            ) {continue;} // no repetitions
            let generation = relatas_1[tab_to_rel_idx[&parent.idx]].generation + 1;
            max_gen = max_gen.max(generation);
            
            match tab_to_rel_idx.get_mut(&tab.idx) {
                Some(rel_idx) => {
                    let pr = &mut relatas_1[*rel_idx];
                    // if pr.generation > parent_generation && !pr.parents.contains(&parent.pia()) {
                    if !pr.parents.contains(&parent.pia()) {
                        dbg_assert!(*pr.tab != *root);
                        pr.parents.push(parent.pia());
                    } 
                },
                None => {
                    // let is_candidate = candidates.iter().any(|(c, at)| c == &tab.idx);
                    let i = candidates.iter().position(|(c, at)| c == &tab.idx);
                    let is_candidate = i != None;
                    let _ = i.map_or([].as_pia(), |i| candidates.remove(i).1);
                    tab_to_rel_idx.insert(tab.idx, relatas_1.len());
                    relatas_1.push(ProtoRelata {
                        tab: tab.pia(), parents: [parent.pia()].as_vec_a(), is_candidate, generation
                    });
                },
            }
            queue.push_back(tab);
        }
    }
    if log {let proto_relatas_raw: _ = &relatas_1; dbg!(proto_relatas_raw);}

    // copy multi-parent proto relatas across related tables
    for pr in &relatas_1 {
        // if
        // if pr.parents < 2 {continue;}
        
    }

    // ------- must be candidate or parent of candidate -------
    let candidates_tabs: Vec<TabIdx> = candidates_2.iter().map(|c| c.0).collect();
    let mut relatas_2: VecA<ProtoRelata> = arena::new_vec(candidates_tabs.len()); // path starts bottom-up
    for relata in relatas_1.into_iter().rev() {
        if relatas_2.contains(&relata) {continue};
        if candidates_tabs.contains(&relata.tab.idx) { 
            relatas_2.push(relata.clone());
            continue;
        }
        let is_parent = relatas_2.iter().any(|r| r.parents.contains(&relata.tab));
        if !is_parent {continue};
        relatas_2.push(relata.clone());
    }
    relatas_2.reverse(); // make relatas_2 top-down
    if log {let proto_relatas_pruned: _ = &relatas_2; dbg!(proto_relatas_pruned);}


    relatas_2
}


fn split_relata_tree(proto_relatas: VecA<ProtoRelata>, model: &DataModel, cols: &[&Name], tok_data: &TokData
) -> Fallible<VecA<RelataSplit>> {


    /* 
    pub type Name = (Pia<[TokIdx]>, bool);
    arrows = [
            // merge1,..., mergeN mergeM 
        col1: [tab21, ..., tab1N, tab1M],
        col2: [tab21, ..., tab2N],
    ]
    */
    let root = &*proto_relatas[0].tab.name;
    let mut reverse_arrow_lines: VecA<(ArrowLine, TokIdx)> = arena::new_vec(cols.len());
    for (name_toks, has_arrow) in cols {
        if !*has_arrow {continue;}
        let mut arrow_line = name_toks[0..name_toks.len() - 1].as_vec_a();
        let col = *name_toks.last().unwrap();

        // check that arrow line is oriented from root to leaf

        for pair in arrow_line.windows(2) {
            let (prev_tab_tok, tab_tok) = (pair[0], pair[1]);
            let tab_name = &**tok_data.texts(tab_tok);
            let prev_tab_name = &**tok_data.texts(prev_tab_tok);
            let rel = proto_relatas.iter().find(|&r| r.tab.name == tab_name);
            let prev_rel = proto_relatas.iter().find(|&r| r.tab.name == prev_tab_name);
            if rel == None || prev_rel == None {
                dbg_assert!(false, "");
                return query_error!(tok_data.line(col), 
                    "Unrecognized tables {}->{} in arrow resolution", tab_name, prev_tab_name
                );
            }
            let (rel, prev_rel) = (rel.unwrap(), prev_rel.unwrap());
            if rel.generation < prev_rel.generation {return query_error!(tok_data.line(col), "
                You cannot join from {} through {}->{}, since {} is closer to {} than {}. 
                Hint: join through {}->{} or change the the root {} to another table or write the joins manually.
                ", root, prev_tab_name, tab_name, prev_tab_name, root, tab_name,
                tab_name, prev_tab_name, root
            )};
        }

        arrow_line.reverse(); // reverse can be omitted by using a queue
        reverse_arrow_lines.push((arrow_line, col));
    }
    if log {dbg!(&reverse_arrow_lines);}

    P!(reverse_arrow_lines.is_empty(), proto_relatas[0].is_candidate);
    if !reverse_arrow_lines.is_empty() && !proto_relatas[0].is_candidate {
        let root_child_count = proto_relatas.iter().filter(|&r| r.generation == 1).count();
        P!(root_child_count);
        if root_child_count == 1 {return query_error!(0, "
            The root table {} is unused. Unused roots may not be combined with arrow resolution, 
            since it would cause unintuitive behaiviour.
            when the root is unused 
            ", root
        )};
    }

    let mut has_passed_root = false;
    let mut relata_splits = arena::new_vec(proto_relatas.len());

    for rel in &proto_relatas {
        let (tab, parents, is_candidate) = (rel.tab, &rel.parents, rel.is_candidate);
        dbg_assert!(has_passed_root == !parents.is_empty());
        if parents.len() <= 1 {
            has_passed_root = true;
            let parent = parents.get(0).map(|p| *p);
            relata_splits.push(RelataSplit { tab, parent, is_candidate, arrow_line_names: Vec::new() });
            continue;
        }
        dbg_assert!(parents.len() > 1);

        // arrow resolution
        // foreach variable with arrow lines, there must a matching parent.
        for (rev_arrow_line, col) in &mut reverse_arrow_lines {
            let parents_str = "[".as_string_a() 
                + &*parents.iter().map(|t| &*t.name ).collect::<Vec<&str>>().join(", ")
                + "]";

            if rev_arrow_line.is_empty() { return query_error!(tok_data.line(*col), "
                There are multiple join paths from {} through {} to {}, but the arrow resolution doesn't not specify which 
                of the tables {} to join through in order to reach {}. 
                Hint: add one of {} to the arrow tab resolution as in `table->col`. 
                ", root, tab.name, tok_data.texts(*col), parents_str, tab.name, parents_str
            )}; 
            let arrow_line_names: Vec<Pia<str>> = rev_arrow_line.iter().rev().map(|a| tok_data.texts(*a).as_pia()).collect();
            let arrow_tab = tok_data.texts(*rev_arrow_line.last().unwrap());
            let mut matching_parent = None;
            
            for &parent in parents {
                if **arrow_tab != *parent.name {continue;}
                matching_parent = Some(parent);
                rev_arrow_line.pop();
            }
            if matching_parent == None { 
                let parent_to_tab_list = "[".as_string_a() 
                + &*parents.iter().map(|t| format!("{}->{}", t.name, tab.name))
                    .collect::<Vec<String>>().join(", ")
                + "]";
                return query_error!(tok_data.line(*col), "
                There are multiple join paths from {} through {} to {}, but the arrow resolution doesn't match any of them
                Expected one of {} in order to reach {}, but found {}. 
                Hint: replace {} by one of {}.
                ", root, tab.name, tok_data.texts(*col), parent_to_tab_list, tab.name, arrow_tab, arrow_tab, parents_str
            )};

            relata_splits.push(RelataSplit { tab, parent: matching_parent, is_candidate, arrow_line_names }) // add arrow_line

        }
    }
    P!(&reverse_arrow_lines);
    if log {dbg!(&relata_splits);}

    // check that arrow resolution was succesful
    for (i, rel) in relata_splits.iter().enumerate() {

        // if rel_parent
    }

    Ok(relata_splits)
}


fn prune_relata_tree(relatas: &[RelataSplit], model: &DataModel, cols: &[&Name], tok_data: &TokData
) -> Fallible<VecA<RelataSplit>> {

    // ------- must be referenced or parent of referenced -------
    let mut path: VecA<RelataSplit> = arena::new_vec(relatas.len()); // path starts bottom-up
    for relata in relatas.into_iter().rev() {
        if path.contains(&relata) {continue};
        if relata.is_candidate { 
            path.push(relata.clone());
            continue;
        }
        let is_parent = path.iter().any(|r| r.parent == Some(relata.tab));
        if !is_parent {continue};
        path.push(relata.clone());
        
    }
    path.reverse(); // make path top-down
    if log {dbg!(&path);}

    // ------- remove unreferenced untraversed top of tree ------- //
    // let using_arrow_resolution = path.iter().any(|r| !r.arrow_line_names.is_empty());
    while let Some(relata) = path.first() {
        if relata.is_candidate {break;}
        // check if relata is a parent of at least two elements of path
        let mut kid_count = 0;
        for previous in &path { // this is technically O(N^2), but path is of order 1, so it is okay.
            if previous.parent == Some(relata.tab) {
                kid_count += 1;
                if kid_count > 1 {break;}
            }
        }
        if kid_count > 1 { break; }
        // if using_arrow_resolution { return query_error!(0, "bib")

        // }

        // this cannot be remove_swap
        path.remove(0); // evt. warn or error when this happens. Could e.g. be controlled by a setting.
        if !path.is_empty() {path[0].parent = None}
    }
    if log {dbg!(&path);}


    Ok(path)
}


fn equip_with_info(path_split: VecA<RelataSplit>, model: &DataModel) -> VecA<Relata> {

    type RelataIdx = usize;
    let mut tab_idx_to_relata: HashMapA<TabIdx, RelataIdx> = arena::new_map(path_split.len());
    for (i, rel) in path_split.iter().enumerate() {
        tab_idx_to_relata.insert(rel.tab.idx, i);
    }

    let mut path: VecA<Relata> = arena::new_vec(path_split.len());
    for rel in path_split.into_iter() {
        let tab = rel.tab;
        let parent_relata = if rel.parent == None {None} else {
            // We loop top down, so if there is a parent, then we have already passed it.
            // Therefore the parent is included in tab_idx_to_relata and path, which makes the indexing safe. 
            Some(path[tab_idx_to_relata[&rel.parent.unwrap().idx]].pia())
        };
        let fk_to_parent = if rel.parent == None {None} else {
            tab.ref_tab_to_fk.get(&rel.parent.unwrap().idx).map(|fk| fk.as_pia())
        };
        let mut fk_to_children: HashMapA<TabIdx, Pia<str>> = arena::new_map(tab.ref_tab_to_fk.len());
        for (&referenced_tab, fk) in &tab.ref_tab_to_fk {
            if Some(referenced_tab) != rel.parent.map(|p| p.idx) {
                // a referenced table that's not a parent, must be a child.
                fk_to_children.insert(referenced_tab, fk.as_pia());
            }
        }
        
        let relata = Relata {
            tab: tab, 
            name: tab.name.as_pia(), 
            parent: parent_relata, 
            pk: tab.pk.as_pia(), 
            fk_to_parent, 
            fk_to_children, 
            arrow_tabs: rel.arrow_line_names,
            needs_skema_prefix: model.needs_skema_prefix(&tab),
        };

        path.push(relata);
        
    }
    if log {dbg!(&path);}
    path
}


fn find_resolved_columns(path: &mut[Relata], cols: &[&Name], model: &DataModel, tok_data: &TokData) -> VecA<ResolvedColumn> {
    let mut resolved_cols = arena::new_vec(0);
    for (prefixed_col, has_arrow) in cols {
        if !*has_arrow {continue;}

        let col_idx = *prefixed_col.last().unwrap();
        let col_text = tok_data.texts(col_idx).to_string();
        let mut table = None;
        let mut relata = None;
        for rel in path.iter_mut() {
            let tab = model.table_by_idx(rel.tab.idx);
            if tab.columns.contains(&col_text) { // what if you need schema prefix to get a unique table?
                table = Some(tab);
                relata = Some(rel);
                break;
            }
        }
        dbg_assert!(table != None && relata != None); // model should check that names can't be empty
        let relata = relata.unwrap();

        // if relata.arrow_tabs.is_empty() && prefixed_col.len() == 1 {WARNING unnessesary arrow tab}
        dbg_assert!(relata.arrow_tabs.is_empty() || relata.arrow_tabs.len() == prefixed_col.len() - 1);
        
        let arrow_tabs: Vec<Pia<str>> = prefixed_col[..prefixed_col.len()-1].iter()
            .map(|a| tok_data.texts(*a).as_pia()).collect();
        dbg_assert!(!arrow_tabs.is_empty()); // query error?

        if relata.arrow_tabs.is_empty() {
            relata.arrow_tabs = arrow_tabs.clone();
        };
        let prefixed_col = *prefixed_col;

        resolved_cols.push(ResolvedColumn { 
            table: table.unwrap().name.as_pia(), 
            arrow_tabs: arrow_tabs,
            start_idx: prefixed_col[0], 
            end_idx: *prefixed_col.last().unwrap(),
        });
    }
    if log {dbg!(&resolved_cols);}
    resolved_cols
}

