use crate::{compilation::{DataModel, RefQueryRow, TabQueryRow}, tools::debug_macros::P};

macro_rules! tab {
    ($skema:expr, $tab:expr; $($columns:tt),*) => {{
        TabQueryRow {
            schema: $skema.to_string(), table: $tab.to_string(), pkeys: vec![$tab.to_string() + "_id"], 
            columns: vec![$tab.to_string() + "_id", $tab.to_string() + "_", $($columns.to_string()),*]
        }        
    }}
}
macro_rules! ref_ {
    ($skema:expr, $tab:expr, $ref_skema:expr, $ref_tab:expr; $($fkeys:tt),*) => {{
        RefQueryRow {
            schema: $skema.to_string(), ref_schema: $ref_skema.to_string(), 
            table: $tab.to_string(), ref_table: $ref_tab.to_string(), 
            fkeys: vec![$($fkeys.to_string()),*] 
        }
    }}
}

pub fn get_model() -> DataModel {

    DataModel::new(
        "postgres".into(), 
        vec![
            tab!("la", "la";   "hv_id"),
            tab!("la", "hv";   ),
            tab!("la", "bv";   "la_id"),
            tab!("la", "lav";  "la_id"),
            tab!("la", "nrla"; "la_id", "nrkt_id"),
            tab!("la", "nrkt"; ),
            tab!("la", "lp";   "nrla_id"),
            tab!("la", "ad";   "nrla_id"),
            tab!("la", "mc";   "la_id"),
            tab!("la", "nc";   "la_id"),
            tab!("la", "ik";   "la_id"),
            tab!("la", "vgt";  "nrla_id"),
            tab!("la", "rev";  "nrla_id"),
            tab!("la", "ek";   "nrla_id"),

            tab!("ba", "ba";   "la_id"),
            tab!("ba", "bav";  "ba_id"),
            tab!("ba", "nrba"; "ba_id", "nrla_id"),
            tab!("ba", "var";  "ba_id"),
            tab!("ba", "bt";   "ba_id"),
            tab!("ba", "fil";  "bt_id"),
            tab!("ba", "tv";   "bt_id", "var_id"),
            tab!("ba", "kor";  "bt_id", "fra_var_id", "til_var_id"),
            tab!("ba", "hir";  "bt_id", "fra_var_id", "til_var_id"),
            tab!("ba", "vgt";  "bt_id", "fra_var_id", "til_var_id"),
            tab!("ba", "kt";   "bt_id", "fra_var_id", "til_var_id"),

            tab!("log", "lev"; "ad_id"),
            tab!("log", "lb";  "lev_id"),
            
        ],
        vec![
            // skema, tab, ref_skema, ref_tab; fkeys
            ref_!("la", "la",   "la", "hv";   "hv"),
            ref_!("la", "bv",   "la", "la";   "la_id"),
            ref_!("la", "lav",  "la", "la";   "la_id"),
            ref_!("la", "nrla", "la", "la";   "la_id"),
            ref_!("la", "nrla", "la", "nrkt"; "nrkt_id"),
            ref_!("la", "lp",   "la", "nrla"; "nrla_id"),
            ref_!("la", "ad",   "la", "nrla"; "nrla_id"),
            ref_!("la", "mc",   "la", "la";   "la_id"),
            ref_!("la", "nc",   "la", "la";   "la_id"),
            ref_!("la", "ik",   "la", "la";   "la_id"),
            ref_!("la", "vgt",  "la", "nrla"; "nrla_id"),
            ref_!("la", "rev",  "la", "nrla"; "nrla_id"), // you ignore the second nrla reference
            ref_!("la", "ek",   "la", "nrla"; "nrla_id"), // you ignore the second nrla reference

            ref_!("ba", "ba",   "la", "la";   "la_id"),
            ref_!("ba", "bav",  "ba", "ba";   "ba_id"),
            ref_!("ba", "nrba", "ba", "ba";   "ba_id"),
            ref_!("ba", "nrba", "la", "nrla"; "nrla_id"),
            ref_!("ba", "var",  "ba", "ba";   "ba_id"),
            ref_!("ba", "bt",   "ba", "ba";   "ba_id"),
            ref_!("ba", "fil",  "ba", "bt";   "bt_id"),
            ref_!("ba", "tv",   "ba", "bt";   "bt_id"),
            ref_!("ba", "tv",   "ba", "var";  "var_id"),
            ref_!("ba", "kor",  "ba", "bt";   "bt_id"),
            ref_!("ba", "kor",  "ba", "var";  "fra_var_id"),
            ref_!("ba", "kor",  "ba", "var";  "til_var_id"),
            ref_!("ba", "hir",  "ba", "bt";   "bt_id"),
            ref_!("ba", "hir",  "ba", "var";  "fra_var_id"),
            ref_!("ba", "hir",  "ba", "var";  "til_var_id"),
            ref_!("ba", "vgt",  "ba", "bt";   "bt_id"),
            ref_!("ba", "vgt",  "ba", "var";  "fra_var_id"),
            ref_!("ba", "vgt",  "ba", "var";  "til_var_id"),
            ref_!("ba", "kt",   "ba", "bt";   "bt_id"),
            ref_!("ba", "kt",   "ba", "var";  "fra_var_id"),
            ref_!("ba", "kt",   "ba", "var";  "til_var_id"),

            ref_!("log", "lev", "la",  "ad";  "ad_id"),
            ref_!("log", "lb",  "log", "lev"; "lev_id"),
            ]
        ).unwrap()
}

