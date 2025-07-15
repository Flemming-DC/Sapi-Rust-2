use super::*;
use crate::tools::*;

pub fn generate_sql(tok_data: TokData, semantics: VecA<Semantic>) -> StringA {
    let mut sql: StringA = arena::new_string(tok_data.texts.len() * 3);
    let mut next_tok = 0;
    
    for (i, semantics) in semantics.iter().enumerate() {
        match semantics {
            Semantic::Selection {root, path, resolved_cols} 
                => select_generate(&root, &path, &resolved_cols, &tok_data, &mut next_tok, &mut sql),
            // _ => unimplemented!(),
        };
    }
    copy_from(&tok_data, &arena::new_vec(0), &mut sql, &mut next_tok, tok_data.tok_types.len()); 
    dbg_assert!(next_tok == tok_data.tok_types.len());
    sql
}

fn select_generate(
    root: &PrefixedTab, path: &[Relata], resolved_cols: &[ResolvedColumn], 
    tok_data: &TokData, next_tok: &mut usize, sql: &mut StringA
) {
    let resolved_tables: Vec<Pia<str>> = resolved_cols.iter().map(|c| c.table).collect(); 

    // let mut tab_idx_to_relata: HashMapA<TabIdx, &Relata> = arena::new_map(path.len());
    // for relata in path {
    //     tab_idx_to_relata.insert(relata.tab.idx, &relata);
    // }

    let from_tab = if path.is_empty() {""
        // } else if resolved_tables.contains(&path[0].name) {&*format!("{} as {}_to", path[0].name, path[0].name)
        } else if !&path[0].arrow_tabs.is_empty() {&*format!("{} as {}", path[0].name, alias(&path[0].arrow_tabs, &path[0].name))
        } else {&path[0].name};        
    let mut joins: VecA<String> = arena::new_vec(0);

    for relata in path {
        if relata.parent == None {continue;} // we skip the root. nb: The root is at from, not at any of the subsequent joins
        let parent = relata.parent.unwrap(); // tab_idx_to_relata[&relata.parent.unwrap().tab.idx];
        
        let is_resolved = resolved_tables.contains(&relata.name);
        let (tab_as_alias, alias) = if is_resolved {
            let a = alias(&relata.arrow_tabs, &relata.name);
            (format!("{} as {}", relata.name, a).as_pia(), a)
        } else if relata.needs_skema_prefix {
            let skema_tab = format!("{}.{}", relata.tab.skema, relata.name).as_pia();
            (skema_tab, skema_tab)
        } else {(relata.name, relata.name)};

        let join = if relata.fk_to_parent != None {format!(
            "JOIN {} on {}.{} = {}.{}", tab_as_alias, alias, relata.fk_to_parent.as_ref().unwrap(), parent.name, parent.pk
        )} else {format!(
            "JOIN {} on {}.{} = {}.{}", tab_as_alias, alias, relata.pk, parent.name, parent.fk_to_children[&relata.tab.idx]
        )};
        joins.push(join);
    }
    let generated_text = " ".as_string_a() + from_tab + "\n" + joins.join("\n").as_str();

    copy_from(&tok_data, resolved_cols, sql, next_tok, root.tab.raw(0)); 
    sql.push_str(&*generated_text);
    let skip_count = if root.across_schemas {4} else {2}; 
    *next_tok = root.tab.raw(skip_count); // next is just after `... (Across Schemas)?`
    
}



fn copy_from(tok_data: &TokData, resolved_cols: &[ResolvedColumn], sql: &mut StringA, next_tok: &mut usize, up_to: usize) {
    debug_assert!(*next_tok <= up_to);
    let mut i = *next_tok;
    while i < up_to {
        sql.push_str(&tok_data.ignored[i]);
        
        // resolve arrows 
        let mut has_resolved = false;
        for rc in resolved_cols {
            if i != rc.start_idx.raw(0) {continue;}
            let alias = alias(&rc.arrow_tabs, &*rc.table); // rc.table
            let col_text = tok_data.texts(rc.end_idx).to_string();
            sql.push_str(format!("{}.{}", alias, col_text).as_str());
            i = rc.end_idx.raw(1);
            has_resolved = true;
            break;
        }
        if has_resolved {continue;}

        sql.push_str(&tok_data.texts[i]);
        i += 1;
    }
    *next_tok = up_to;
}


fn alias(arrow_tabs: &[Pia<str>], ref_tab: &str) -> Pia<str> {
    let mut out = arena::new_string(arrow_tabs.len());
    for tab in arrow_tabs {
        out.push_str(&format!("{}_to_", *tab));
    }
    out.push_str(&format!("{}", ref_tab));
    out.as_pia()
}



/*
    // --- generate from clause --- //
    // path must be ordered correctly
    joins = [] // evt. include "from root" but then these must also be removed from the input query to avoid double counting
    for relata in path
        join = 
            "join relata on relata.fk = relata.parent.pk" 
            if relata.fk_to_parent != None else
            "join relata on relata.pk = relata.parent.fk_to_children[relata]"
        joins.push(join)
        // extra features: 
        //    left join 
        //    jsonic / grouped / hierachic select
        //    prefix leaf by middle node (e.g. prefix ba_variable_name by filter)
    
    generated_token = tok of type Generated with text "\n".concat(joins)
    out_tokens = [token-before-DotDotDot, generated_token, token-after-DotDotDot]
*/

