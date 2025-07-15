use std::collections::HashMap;
use sqlparser::tokenizer::Location;
use crate::compilation::pipeline::*;
use crate::compilation::data::*;
use crate::tools::*;

#[test]
fn test_tokenize() {
    let (query, model) = setup();
    let actual = lexer::tokenize(&query, model.get_dialect()).unwrap();

    use TokTy::*;
    fn loc(l: u64, c: u64) -> Location {Location {line: l, column: c}}

    let expected = TokData {
        tok_types: [Select, Ident, Ident, LParen, Select, LParen, Ident, RParen, 
            From, Ident, RParen, From, Ident, TripleDot, Eof].as_pia(),
        locations: [
            loc(2, 1), // SELECT
            loc(3, 5), // col00_2
            loc(4, 5), // col1_2
            loc(5, 5), // (
            loc(5, 6), // SELECT
            loc(5, 18), // ( 
            loc(5, 19), // col2_2
            loc(5, 25), // )
            loc(5, 27), // FROM 
            loc(5, 32), // tab20
            loc(5, 37), // )
            loc(8, 1), // FROM 
            loc(8, 6), // tab
            loc(8, 9), // ...
            loc(8, 12), // Eof
            ].as_pia(),
        texts: [
            "SELECT".as_string_a(),
            "col00_2".as_string_a(),
            "col1_2".as_string_a(),
            "(".as_string_a(),
            "SELECT".as_string_a(),
            "(".as_string_a(),
            "col2_2".as_string_a(),
            ")".as_string_a(),
            "FROM".as_string_a(),
            "tab20".as_string_a(),
            ")".as_string_a(),
            "FROM".as_string_a(),
            "tab".as_string_a(),
            "...".as_string_a(),
            "".as_string_a(), // Eof
        ].as_pia(),
        ignored: [
            "\n".as_string_a(), // SELECT
            "\n    ".as_string_a(), // col00_2
            ",\n    ".as_string_a(), // col1_2
            ",\n    ".as_string_a(), // (
            "".as_string_a(), // SELECT
            " count".as_string_a(), // ( 
            "".as_string_a(), // col2_2
            "".as_string_a(), // )
            " ".as_string_a(), // FROM 
            " ".as_string_a(), // tab20
            "".as_string_a(), // )
"\n--FROM tree
--join cte ON tree.col_1 = cte.col0_1\n".as_string_a(), // FROM 
            " ".as_string_a(), // tab
            "".as_string_a(), // ...
            "".as_string_a(), // Eof
        ].as_pia(),
    };

    for i in 0..actual.tok_types.len() {
        // println!("{}", i);
        // println!("\"{}\" vs. \"{}\" for \"{}\"", actual.ignored[i], expected.ignored[i], actual.texts[i]);
        assert_eq!(actual.tok_types[i], expected.tok_types[i]);
        assert_eq!(actual.locations[i], expected.locations[i]);
        assert_eq!(&*actual.texts[i], &*expected.texts[i]);
        assert_eq!(&*actual.ignored[i], &*expected.ignored[i]);
    }
    assert_eq!(actual.tok_types.len(), expected.tok_types.len());

}

#[test]
fn test_parse() {
    let (query, model) = setup();
    let tok_data = lexer::tokenize(&query, model.get_dialect()).unwrap();
    let actual = parser::parse(tok_data.shallow_copy()).unwrap();

    let mut expected = Ast {
        nodes: arena::new_vec(3),
        names_stack: arena::new_vec(10), // VecA<(NodeIdx, VecA<Name>)>, where Name = VecA<TokIdx>
    };
    let tok = |i: usize| {TokIdx::new(&tok_data, i)};
    use Node::*;
    expected.names_stack.push((NodeIdx::new(&expected, usize::MAX), arena::new_vec(4))); // usize::MAX means uninit

    expected.nodes.push(Selection { root: PrefixedTab {schema: None, tab: tok(12), across_schemas: false } });
    let select = NodeIdx::new(&expected, 0);

    *expected.names_stack.last_mut().unwrap() = (select, [
        ([tok(1)].as_pia(), false),
        ([tok(2)].as_pia(), false),
        ([tok(12)].as_pia(), false),
    ].as_vec_a());
    expected.nodes.push(Script { queries: [select].as_pia() });
        
    for i in 0..actual.nodes.len() {
        assert_eq!(actual.nodes[i], expected.nodes[i]);
        assert_eq!(actual.names_stack.get(i), expected.names_stack.get(i));
    }
    assert_eq!(actual.nodes.len(), expected.nodes.len());

}


// #[test]
// fn test_semantic() {
//     let (query, model) = setup();
//     let tok_data = lexer::tokenize(&query, model.get_dialect()).unwrap();
//     let ast = parser::parse(tok_data.shallow_copy()).unwrap();
//     let actual = analyzer::analyze(&tok_data, &ast, &model).unwrap();

//     let tok = |i: usize| {TokIdx::new(&tok_data, i)};
//     let select = NodeIdx::new(&ast, 0);
//     // let tab = |i: usize| TabIdx {raw: i, #[cfg(debug_assertions)] model: &model as *const DataModel};

//     let mut expected = arena::new_vec(1);
//     let resolved_cols = arena::new_vec(0);
//     expected.push((select, Semantic::Selection {
//         root: PrefixedTab {schema: None, tab: tok(12) }, 
//         path: [
//             Relata {
//                 idx: model.make_tab_idx(0),
//                 name: "tab".as_pia(), 
//                 parent: None, 
//                 pk: "tab_id".as_pia(), 
//                 fk_to_parent: None, 
//                 fk_to_children: HashMap::new(), //<model.make_tab_idx(0), "kid_id".to_string()>,
//             },
//             Relata {
//                 idx: model.make_tab_idx(1),
//                 name: "tab0".as_pia(), 
//                 parent: Some(model.make_tab_idx(0)), 
//                 pk: "tab0_id".as_pia(), 
//                 fk_to_parent: Some("tab_id".as_pia()), 
//                 fk_to_children: HashMap::new(),
//             },
//             Relata {
//                 idx: model.make_tab_idx(2),
//                 name: "tab1".as_pia(), 
//                 parent: Some(model.make_tab_idx(0)), 
//                 pk: "tab1_id".as_pia(), 
//                 fk_to_parent: Some("tab_id".as_pia()), 
//                 fk_to_children: HashMap::new(),
//             },
//             Relata {
//                 idx: model.make_tab_idx(4), // skipping idx 3, since it belongs to tab2
//                 name: "tab00".as_pia(), 
//                 parent: Some(model.make_tab_idx(1)), 
//                 pk: "tab00_id".as_pia(), 
//                 fk_to_parent: Some("tab0_id".as_pia()), 
//                 fk_to_children: HashMap::new(),
//             },
//         ].as_vec_a(),
//         resolved_cols
//     }));
//     assert_eq!(actual.len(), expected.len());
//     // same values
//     for (i, sem) in actual.iter().enumerate() {
//         #[allow(irrefutable_let_patterns)]
//         let Semantic::Selection { root: ar, path: ap, resolved_cols: arc } = sem else {panic!()};
//         #[allow(irrefutable_let_patterns)]
//         let Semantic::Selection { root: er, path: ep, resolved_cols: erc } = &expected[i].1 else {panic!()};

//         assert_eq!(ar, er);
        
//         for a in ap {
//             let mut found = false;
//             for e in ep {
//                 if e.idx != a.idx {continue;}
//                 assert!(e.idx == a.idx);
//                 assert!(e.name == a.name);
//                 assert!(e.parent == a.parent);
//                 assert!(e.fk_to_parent == a.fk_to_parent);
//                 assert!(e.fk_to_children == a.fk_to_children);
//                 found = true;
//             }
//             assert!(found);
//         }
//     }
// }


fn setup() -> (String, DataModel) {
    err::set_panic_on_query_error(true);
    let query = "
SELECT
    col00_2,
    col1_2,
    (SELECT count(col2_2) FROM tab20)
--FROM tree
--join cte ON tree.col_1 = cte.col0_1
FROM tab...".to_string();

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
    let model = DataModel::new(
            "sqlite".into(), 
            vec![tab(""), tab("0"), tab("1"), tab("2"), tab("00"), tab("20")],
            vec![ref_("0", ""), ref_("1", ""), ref_("2", ""), ref_("00", "0"), ref_("20", "2")]
            ).unwrap();
    (query, model)
}


