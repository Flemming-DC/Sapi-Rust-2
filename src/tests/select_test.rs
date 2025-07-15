use crate::compilation;
use crate::tools::err::ResultTEExt;
use crate::tools::*;
use super::*;
use proptest::prelude::*;


proptest! {
    #[test] fn no_crash_pure_noise(query in "\\PC*") {
        err::set_panic_on_query_error(false);
        let model = dm_1::get_model();
        compilation::compile(query, &model).ignore_err();
    }

    #[test] fn no_crash_pertubed_query(perturb_str in "\\PC*", perturb_indicies in prop::collection::vec(any::<usize>(), 10)) {
        err::set_panic_on_query_error(false);
        let model = dm_1::get_model();
        let mut query = "
            SELECT 
                col00_2,
                col1_2,
                (SELECT count(col2_2) FROM tab20)
            --FROM tree
            --join cte ON tree.col_1 = cte.col0_1
            FROM tab...
        ".to_string();
        for (i, c) in perturb_str.chars().enumerate() {
            if i >= perturb_indicies.len() {break;}
            if query.is_char_boundary(perturb_indicies[i]) {
                query.insert(perturb_indicies[i], c);
            }
        }
        compilation::compile(query, &model).ignore_err();
    }
}

#[test] fn subquery_and_join_two_level() { perform_test("
    SELECT 
        col00_2,
        col1_2,
        (SELECT count(col2_2) FROM tab20)
    --FROM tree
    --join cte ON tree.col_1 = cte.col0_1
    FROM tab...
    ", "
    SELECT 
        col00_2,
        col1_2,
        (SELECT count(col2_2) FROM tab20)
    --FROM tree
    --join cte ON tree.col_1 = cte.col0_1
    FROM tab
    JOIN tab0 on tab0.tab_id = tab.tab_id
    JOIN tab1 on tab1.tab_id = tab.tab_id
    JOIN tab00 on tab00.tab0_id = tab0.tab0_id
");}

#[test] fn single_tab() { perform_test("
    select col1_1 from tab1...
    ", "
    select col1_1 from tab1
");}
// should this fail as col1 is unrecognized?

#[test] fn joins_single_level() { perform_test("
    select col1_1 
    from tab...
    where col2_1
    ", "
    select col1_1 
    from tab
    join tab1 on tab1.tab_id = tab.tab_id
    join tab2 on tab2.tab_id = tab.tab_id
    where col2_1
");}

#[test] fn pure_sql() { perform_test("
    select col1_1 
    from tab
    join tab1 on tab1.tab_id = tab.tab_id
    group by col_1 -- lol, this is wrong sql
    ", "
    select col1_1 
    from tab
    join tab1 on tab1.tab_id = tab.tab_id
    group by col_1 -- lol, this is wrong sql
");}

#[test] fn irrelevant_top_of_relata_tree() { perform_test("
    select col0_1, col00_1 
    from tab0...
    ", "
    select col0_1, col00_1 
    from tab0
    join tab00 on tab00.tab0_id = tab0.tab0_id
");}

#[test] fn noise_before_and_after_query() { perform_test("
    slewlwemfwf;
    select col_1, col00_1 
    from tab...
    ;
    hejsa vdsd;
    ", "
    slewlwemfwf;
    select col_1, col00_1 
    from tab
    join tab0 on tab0.tab_id = tab.tab_id
    join tab00 on tab00.tab0_id = tab0.tab0_id
    ;
    hejsa vdsd;
");}


#[test] fn multiple_queries() { perform_test("
    select col_1, col00_1 
    from tab...
    ;
    select col_1
    from tab
    ;
    select col_2
    from tab1...
    ;
    ", "
    select col_1, col00_1 
    from tab
    join tab0 on tab0.tab_id = tab.tab_id
    join tab00 on tab00.tab0_id = tab0.tab0_id
    ;
    select col_1
    from tab
    ;
    select col_2
    from tab
");}


#[test] fn simple_arrow_resolve() { perform_test("
    select tab0->col00_1 
    from tab...
    ", "
    select tab0_to_tab00.col00_1 
    from tab00 as tab0_to_tab00
");}

#[test] fn arrow_resolve_2() { perform_test("
    select col_1, tab0->col00_1
    from tab...
    ", "
    select col_1, tab0_to_tab00.col00_1 
    from tab
    JOIN tab0 on tab0.tab_id = tab.tab_id
    JOIN tab00 as tab0_to_tab00 on tab0_to_tab00.tab0_id = tab0.tab0_id
");}

#[test] fn arrow_resolve_3() { perform_test("
    select 
        col_1, 
        tab0->cols01_1,
    from tab...
    ", "
    select 
        col_1, 
        tab0_to_tabs01.cols01_1,
    from tab
    JOIN tab0 on tab0.tab_id = tab.tab_id
    JOIN tabs01 as tab0_to_tabs01 on tab0_to_tabs01.tab0_id = tab0.tab0_id
");}
#[test] fn arrow_resolve_4() { perform_test("
    select 
        col_1, 
        tab1->cols01_1,
    from tab...
    ", "
    select 
        col_1, 
        tab1_to_tabs01.cols01_1,
    from tab
    JOIN tab1 on tab1.tab_id = tab.tab_id
    JOIN tabs01 as tab1_to_tabs01 on tab1_to_tabs01.tab1_id = tab1.tab1_id
");}

#[test] fn arrow_resolve_both() { perform_test("
    select 
        col_1, 
        tab0->cols01_1,
        tab1->cols01_1,
    from tab...
    ", "
    select 
        col_1, 
        tab0_to_tabs01.cols01_1,
        tab1_to_tabs01.cols01_1,
    from tab
    JOIN tab0 on tab0.tab_id = tab.tab_id
    JOIN tab1 on tab1.tab_id = tab.tab_id
    JOIN tabs01 as tab0_to_tabs01 on tab0_to_tabs01.tab0_id = tab0.tab0_id
    JOIN tabs01 as tab1_to_tabs01 on tab1_to_tabs01.tab1_id = tab1.tab1_id
");}


#[test] fn err_two_dots() { perform_test_err("
select col1_1 from tab1..
");}
#[test] fn err_four_dots() { perform_test_err("
select col1_1 from tab1....
");}
#[test] fn err_no_from() { perform_test_err("
select col1_1 tab1...
");}
#[test] fn err_unrecognized_column() { perform_test_err("
select col1_1111 tab1...
");}
#[test] fn err_unrecognized_table() { perform_test_err("
select col1_1 tab1111...
");}


fn perform_test(query: &str, expected: &str) {
    let (query, expected) = (query.to_string(), expected.to_string());
    err::set_panic_on_query_error(true);
    let model = dm_1::get_model();
    let actual = compilation::compile(query, &model).unwrap().to_string();

    let (actual, expected) = (actual.trim().to_lowercase(), expected.trim().to_lowercase());
    // let actual = actual.trim().to_lowercase().lines().map(|line| line.trim().to_string()).collect::<Vec<String>>().join('\n');
    let a_lines = actual.split('\n');
    let e_lines = expected.split('\n');
    let (mut actual, mut expected) = ("".to_string(), "".to_string());
    for (a, e) in a_lines.zip(e_lines) {
        // P!(i, a, e);
        assert_eq!(a.trim(), e.trim());
        actual.push_str(a.trim());
        expected.push_str(e.trim());
    }
    // let 
    // let a_len = actual.chars().filter(|c| !c.is_whitespace()).count();
    // let e_len = expected.chars().filter(|c| !c.is_whitespace()).count();
    if actual.len() < expected.len() {println!("remaining expected: {}", &expected[actual.len()..])};
    if expected.len() < actual.len() {println!("remaining actual: {}", &actual[expected.len()..])};
    assert_eq!(actual.len(), expected.len());
}

fn perform_test_err(query: &str) {
    err::set_panic_on_query_error(false);
    let model = dm_1::get_model();
    let out = compilation::compile(query.to_string(), &model);
    if !out.is_err() {P!(&out);}
    assert!(out.is_err());
}


