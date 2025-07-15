#![allow(dead_code, unused_imports, unused_variables, unused_parens, non_upper_case_globals)]
use std::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use sapi::{self, DataModel, StringA};
// use crate::dm_1;


// fn fibonacci(model: &DataModel) -> StringA {
//     sapi::compile(query, model)
// }

fn criterion_benchmark(c: &mut Criterion) {
    let model = sapi::get_model();
    let query = "
        SELECT 
            col00_2,
            col1_2,
            (SELECT count(col2_2) FROM tab20)
        --FROM tree
        --join cte ON tree.col_1 = cte.col0_1
        FROM tab...
    ".to_string();
    c.bench_function("sapi compile", |b| b.iter(|| 
        sapi::compile(query.clone(), &model)
    ));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);


