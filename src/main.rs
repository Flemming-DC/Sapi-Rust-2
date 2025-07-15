#![allow(dead_code, unused_imports, unused_variables, unused_parens, non_upper_case_globals)]
// #![allow(non_upper_case_globals)]
mod compilation;
mod tools;
mod tests;
mod vendor;
use sqlx::query;
// use std::hint::black_box;
use tools::{debug_macros::P, err, StringA};




fn main() { if let Err(e) = main_() { eprintln!("{}", e); } }

fn main_() -> err::AnyFallible<()> {


    err::set_panic_on_query_error(true);
    // let query = "
    //     SELECT 
    //         col00_2,
    //         col1_2,
    //         (SELECT count(col2_2) FROM tab20)
    //     --FROM tree
    //     --join cte ON tree.col_1 = cte.col0_1
    //     FROM tab...
    // ".to_string();
    // let model = tests::dm_1::get_model();

    let query = "
        select 
            bt_, fil_, tv_, kor_, hir_, ba.vgt.vgt_, kt_,
            tv->bt->var_
        from ba...
    ".to_string();
    let model = tests::nova_dm::get_model();
    
    // for _ in 0..100 {
    //     let sql = compilation::compile(query.clone(), &model).unwrap();
    //     black_box(sql);
    // }
    let sql: StringA = compilation::compile(query.clone(), &model).unwrap();
    println!("done");
    Ok(())
}

