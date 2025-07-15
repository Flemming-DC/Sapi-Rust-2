use sapi::{DataModel, RefQueryRow, TabQueryRow};


pub fn get_model() -> DataModel {
    DataModel::new(
        "sqlite".into(),
        vec![tab(""), tab("0"), tab("1"), tab("2"), tab("00"), tab("20")],
        vec![ref_("0", ""), ref_("1", ""), ref_("2", ""), ref_("00", "0"), ref_("20", "2")]
        ).unwrap()
}

fn tab(suffix: &str) -> TabQueryRow {
    let tab = "tab".to_string() + suffix;
    let col = "col".to_string() + suffix;
    TabQueryRow {schema: "s".to_string(), table: tab.clone(), pkeys: vec![tab.clone() + "_id"], columns: vec![
            tab.clone() + "_id", col.clone() + "_1", col.clone() + "_2"
    ]}
}
fn ref_(suffix: &str, ref_suffix: &str) -> RefQueryRow {
    let tab = "tab".to_string() + suffix;
    let ref_tab = "tab".to_string() + ref_suffix;
    RefQueryRow {
        schema: "s".to_string(), ref_schema: "s".to_string(), 
        table: tab.clone(), ref_table: ref_tab.clone(), 
        fkeys: vec![ref_tab + "_id"] // pkeys: vec![tab or ref_tab ?? + "_id"], 
    }
}



