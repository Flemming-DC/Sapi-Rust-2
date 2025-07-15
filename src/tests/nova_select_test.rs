use crate::compilation;
use crate::tools::err::ResultTEExt;
use crate::tools::*;
use super::*;
use proptest::prelude::*;


#[test] fn nrv() { perform_test("
    select nrkt_ ||' ('|| nrla_ ||')', 
    from la...
", "
    select nrkt_ ||' ('|| nrla_ ||')', 
    from nrla
    join nrkt on nrkt.nrkt_id = nrla.nrkt_id
");}

#[test] fn lev_nrv() { perform_test("
    select lev_, nrkt_ ||' ('|| nrla_ ||')', 
    from la... across schemas
", "
    select lev_, nrkt_ ||' ('|| nrla_ ||')', 
    from nrla
    join nrkt on nrkt.nrkt_id = nrla.nrkt_id
    join ad on ad.nrla_id = nrla.nrla_id
    join lev on lev.ad_id = ad.ad_id
");}

#[test] fn lev_nrv_root_lev() { perform_test("
    select lev_, nrkt_ ||' ('|| nrla_ ||')', 
    from lev... across schemas
", "
    select lev_, nrkt_ ||' ('|| nrla_ ||')', 
    from lev
    join ad on ad.ad_id = lev.ad_id
    join nrla on nrla.nrla_id = ad.nrla_id
    join nrkt on nrkt.nrkt_id = nrla.nrkt_id
");}

#[test] fn validering_tabs() { perform_test("
    select la_, nc_, ik_, la.vgt.vgt_, rev_, ek_
    from la...
", "
    select la_, nc_, ik_, la.vgt.vgt_, rev_, ek_
    from la
    join nrla on nrla.la_id = la.la_id
    join nc on nc.la_id = la.la_id
    join ik on ik.la_id = la.la_id
    join la.vgt on la.vgt.nrla_id = nrla.nrla_id
    join rev on rev.nrla_id = nrla.nrla_id
    join ek on ek.nrla_id = nrla.nrla_id
");}

#[test] fn la_ba() { perform_test("
    select la_, ba_
    from la...    across schemas
", "
    select la_, ba_
    from la
    join ba on ba.la_id = la.la_id
");}

// problem: if root = ba, then query yields ocsure error, due to pruning top of tree
// which changes what counts as joins up/down the tree. Furthermore, arrow resolution 
// only allows joining down the tree, not up again. ie. You resolve parents, not children.
#[test] fn ba_trins() { perform_test("
    select 
        bt_, fil_, tv_, kor_, hir_, ba.vgt.vgt_, kt_,
        bt->tv->var_
    from ba...
", "
    select
        bt_, fil_, tv_, kor_, hir_, ba.vgt.vgt_, kt_,
        bt_to_tv_to_var.var_
    from bt
    join fil on fil.bt_id = bt.bt_id
    join tv on tv.bt_id = bt.bt_id
    join kor on kor.bt_id = bt.bt_id
    join hir on hir.bt_id = bt.bt_id
    join ba.vgt on ba.vgt.bt_id = bt.bt_id
    join kt on kt.bt_id = bt.bt_id
    join var as bt_to_tv_to_var on bt_to_tv_to_var.var_id = tv.var_id
");}

#[test] fn skema_no_tab_col_err() { perform_test_err("
    select la_, nc_, ik_, la.vgt_, rev_, ek_
    from la...
");}

#[test] fn wrong_datamodel_err() { perform_test_err("
    select col1_1 from tab1...
");}

#[test] fn lev_nrv_cant_join_across_schemas() { perform_test_err("
    select lev_, nrkt_ ||' ('|| nrla_ ||')', 
    from la...
");}


fn perform_test(query: &str, expected: &str) { // essentially dublicated from select_test
    let (query, expected) = (query.to_string(), expected.to_string());
    err::set_panic_on_query_error(true);
    let model = nova_dm::get_model();
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


fn perform_test_err(query: &str) { // essentially dublicated from select_test
    err::set_panic_on_query_error(false);
    let model = nova_dm::get_model();
    let out = compilation::compile(query.to_string(), &model);
    if !out.is_err() {P!(&out);}
    assert!(out.is_err());
}


