use crate::{tools::{arena, err::Fallible, stdstringStringExt, StringA}};
use std::time::Instant;
use super::*;

pub fn compile(query: String, model: &DataModel) -> Fallible<StringA> {
    #[cfg(debug_assertions)] #[cfg(not(test))] let timer = Instant::now();
    arena::reset();
    if query.trim_start().is_empty() {return Ok(query.as_string_a());} // ignore empty queries

    let tok_data = lexer::tokenize(&query, model.get_dialect())?;
    // P!("{}", &tok_data);
    let ast = parser::parse(tok_data.shallow_copy())?;
    // P!(&ast);
    let semantics = analyzer::analyze(&tok_data, &ast, model)?;
    // P!(&semantics);
    let sql = generator::generate_sql(tok_data, semantics);
    println!("{}", &sql);

    // #[cfg(debug_assertions)] #[cfg(not(test))] measure_performance(&query, timer);
    return Ok(sql);
}


#[allow(dead_code)] fn measure_performance(query: &str, timer: Instant) {
    let query_len = query.len();
    let non_ascii_len = query.chars().filter(|&c| !c.is_ascii()).count();
    let effective_query_len = query_len + 10 * non_ascii_len; // the unicode weight is a guess
    let unicode_factor = 1 + 10 * non_ascii_len / query_len; // the unicode weight is a guess
    // println!("{}", &query);
    // P!(query_len, unicode_factor);

    P!(timer.elapsed().as_micros()); // measured 400-550 us in release (11 us according to criterion) and 600-1000 us in debug
    P!(timer.elapsed().as_micros() / effective_query_len as u128);
    P!(arena::allocated_bytes() / effective_query_len); // measured 154 allocated bytes per query char 

    let bound = ((8 * query_len).max(2000) * unicode_factor) as u128;
    dbg_assert!(timer.elapsed().as_micros() < bound, format!(
        "expected timer.elapsed().as_micros() < {}, but found {}", 
        bound, timer.elapsed().as_micros()
    ));
    let bound = (300 * query_len).max(60_000) * unicode_factor;
    dbg_assert!(arena::allocated_bytes() < bound, format!(
        "expected arena::allocated_bytes() < {}, but found {}", 
        bound, arena::allocated_bytes()
    ));
}

