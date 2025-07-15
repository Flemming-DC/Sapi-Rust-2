use std::hash::{Hash, Hasher};
use std::{cell::RefCell, collections::HashMap, fmt};
use sqlparser::dialect::{dialect_from_str, Dialect}; use crate::tools::{arena, Pia, SliceOfTExt, VecA};
// PostgreSqlDialect, 
use crate::tools::err::{Fallible, ModelError};
use super::*;

#[derive(Clone, Eq)] // Debug
pub struct Table {
    pub idx: TabIdx, 
    pub name: String, // no custom allocation, since it is externally given. However, this forces cloning elsewhere.
    pub skema: String, 
    pub columns: Vec<String>, 
    pub pk: String, 
    pub ref_tab_to_fk: HashMap<TabIdx, String>
}
// type Schema = HashMapA<String, Table>; // tab_name to tab

impl PartialEq for Table { fn eq(&self, other: &Table) -> bool {
    self.idx == other.idx
    // self.name == other.name && self.skema == other.skema
}}

#[derive(Eq, Copy, Clone, PartialOrd)] // Display, Debug, PartialEq, Hash
pub struct TabIdx {raw: u32, #[cfg(debug_assertions)] name_idx: usize 
}

#[cfg(debug_assertions)] thread_local! { // used for printing TabIdx
    static tab_names: /*LocalKey*/RefCell<Vec<String>> = RefCell::new(Vec::new());
}


#[derive(Debug)]
pub struct DataModel {
    dialect: Box<dyn Dialect>,
    // a, b are related, iff a has fk pointing to b or b has fk pointing to a
    table_to_related_tables: HashMap<TabIdx, Vec<TabIdx>>, // [{tab -> [related tables]}]
    tables: Vec<Table>,
    schemas: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TabQueryRow {pub schema: String, pub table: String, pub pkeys: Vec<String>, pub columns: Vec<String> }

#[derive(Debug, Clone, PartialEq)]
pub struct RefQueryRow {
    pub schema: String, pub ref_schema: String, pub table: String, pub ref_table: String, 
    pub fkeys: Vec<String> // pub pkeys: Vec<String>, 
}
// evt. equip these structs with a new method for sqlite and postgres
// something like TabQueryData::from_postgress(&conn)
// alternatively: sqlx::query(TabQueryData::postgress_query()).execute(&conn)



impl DataModel {

    pub fn new(dialect_str: String, tab_query_data: Vec<TabQueryRow>, ref_query_data: Vec<RefQueryRow>
    ) -> Result<Self, Box<ModelError>> {
        let dialect = match dialect_from_str(dialect_str) {
            None => return Err(Box::new(ModelError {msg: "Unrecognized SQL dialect.".to_owned()})),
            Some(dialect) => dialect,
        };
        let mut model = DataModel {
            dialect,
            table_to_related_tables: HashMap::new(),
            tables: Vec::new(),
            schemas: Vec::new(),
        };
        model.setup_metadata(tab_query_data, ref_query_data);
        Ok(model)
    }

    fn setup_metadata(&mut self, tab_query_data: Vec<TabQueryRow>, ref_query_data: Vec<RefQueryRow>) {
        for row in tab_query_data {
            // schema: String, table: String, pkeys: Vec<String>, columns: Vec<String>
            assert!(row.pkeys.len() == 1, "Each table should have exactly one primary key"); // evt. drop this assumption
            if !self.schemas.contains(&row.schema) { 
                self.schemas.push(row.schema.clone());
                // StringA::from_str_in(row.schema.as_str(), arena::arena_gen()) 
            };
            #[cfg(debug_assertions)] let name_idx = tab_names.with_borrow_mut(|r| {
                r.push(row.table.clone());
                r.len() - 1
            });

            let table = Table {
                idx: TabIdx {raw: self.tables.len() as u32, #[cfg(debug_assertions)] name_idx: name_idx }, 
                name: row.table, 
                skema: row.schema, 
                columns: row.columns, 
                pk: row.pkeys[0].clone(), 
                ref_tab_to_fk: HashMap::new(), // filled out later
            };
            self.tables.push(table);
        }

        let mut full_name_to_idx: HashMap<(String, String), TabIdx> = HashMap::with_capacity(self.tables.len());
        dbg_assert!(self.table_to_related_tables.is_empty()); 
        for t in &self.tables {
            full_name_to_idx.insert((t.skema.clone(), t.name.clone()), t.idx);
            self.table_to_related_tables.insert(t.idx, Vec::new());
        }

        for row in ref_query_data {
            // RefQueryRow {schema: String, ref_schema: String, table: String, ref_table: String, pkeys: Vec<String>, fkeys: Vec<String> }
            // assert!(row.pkeys.len() == 1, "Each table should have exactly one primary key"); // evt. drop this assumption
            assert!(row.fkeys.len() == 1, "Always bind tables on a single foreign_key. Not multiple, nor zero."); // evt. drop this assumption

            let tab_idx = full_name_to_idx[&(row.schema, row.table)];
            let ref_tab_idx = full_name_to_idx[&(row.ref_schema, row.ref_table)];
            // store the info that tab is related to ref_tab
            let rt = self.table_to_related_tables.get_mut(&tab_idx).unwrap();
            if !rt.contains(&ref_tab_idx) { rt.push(ref_tab_idx); }
            // store the info that ref_tab is related to tab
            let rt = self.table_to_related_tables.get_mut(&ref_tab_idx).unwrap();
            if !rt.contains(&tab_idx) { rt.push(tab_idx); }

            self.tables[tab_idx.raw as usize].ref_tab_to_fk
                .entry(ref_tab_idx).or_insert(row.fkeys[0].clone());
        }

    }

    pub fn table_by_prefixed_column(&self, tok_data: &TokData, prefixed_col: &Pia<[TokIdx]>) -> Fallible<&Table> {
        /* returns None if it isn't a col name */
        // let (prefixed_col, _has_arrow) = prefixed_col;
        dbg_assert!(prefixed_col.len() >= 1 && prefixed_col.len() <= 3);

        let line = tok_data.line(prefixed_col[0]);
        let first_ident = tok_data.texts(prefixed_col[0]).clone();
        let col_name: &str = &tok_data.texts(*prefixed_col.last().unwrap());

        // resolve schema/table ambiguity;
        let mut schema_name: Option<&str> = None;
        let mut table_name: Option<&str> = None;
        if prefixed_col.len() == 3 {
            schema_name = Some(&*tok_data.texts(prefixed_col[0]));
            table_name = Some(&*tok_data.texts(prefixed_col[1]));
        } else if prefixed_col.len() == 2 {
            // if self.schemas.iter().any(|s| *s == *first_ident) {
            //     schema_name = Some(&*first_ident);
            // } else 
            if self.tables.iter().any(|t| t.name == *first_ident) {
                table_name = Some(&*first_ident);
            } else { return query_error!(line,
                // "Unrecognized column prefix. There is no table or schema named {}",
                "Unrecognized table `{}` in prefix of column `{}`.", // what about views, ctes and hardcoded values?
                first_ident, col_name
            )};
        } else { // prefixed_col.len() is neither 2 nor 3
            dbg_assert!(prefixed_col.len() == 1);
        }

        // filter tables
        let mut matching_tables: VecA<&Table> = arena::new_vec(1);
        for t in &self.tables {
            if schema_name != None && schema_name != Some(&*t.skema) {continue;}
            if table_name != None && table_name != Some(&*t.name) {continue;}
            if !t.columns.iter().any(|c|c == col_name) {continue;}
            matching_tables.push(t);
        };

        // // check if col_name could be someting other than an actual column name, 
        // // in which case we can't find a table, but this need not be an error.
        // if matching_tables.len() < 1 {
        //     if self.schemas.iter().any(|s| col_name != s) {return Ok(None);}
        //     if self.tables.iter().any(|t| col_name != t.name) {return Ok(None);}
        // }
        // report errors
        if matching_tables.len() < 1 { return query_error!(line, 
            "Unresolved reference error. There is no table containing the column {}{}{}",
            schema_name.as_ref().map_or("".to_string(), |s| {s.to_string() + "."}), 
            table_name.as_ref().map_or("".to_string(), |s| {s.to_string() + "."}), 
            col_name, 
        )};
        if matching_tables.len() > 1 { return query_error!(line, 
            "Ambiguity error. {}{}{} can describe multiple tables. Matching tables = [{}]",
            schema_name.as_ref().map_or("".to_string(), |s| {s.to_string() + "."}), 
            table_name.as_ref().map_or("".to_string(), |s| {s.to_string() + "."}), 
            col_name, matching_tables.iter().map(|t| format!("{}.{}", t.skema, t.name)).collect::<Vec<String>>().join(", "),
        )};
        
        let table = matching_tables[0];
        Ok(table)
    }

    pub fn candidates_for_arrow_resolution(&self, tok_data: &TokData, arrow_col: &Pia<[TokIdx]>
    ) -> Fallible<Pia<[&Table]>> {
        // This merely returns the tables that contains the column.
        // They will get resolved in the analyzer, by comparing their ancestors to the arrow tabs
        dbg_assert!(!arrow_col.is_empty());
        // let root_schema = if root.schema != None {Some(tok_data.texts(root.schema.unwrap()))} else {None};
        // let root_tab = tok_data.texts(root.tab);
        let line = tok_data.line(arrow_col[0]);
        let col_name = tok_data.texts(*arrow_col.last().unwrap());

        // filter tables
        let mut matching_tables: VecA<&Table> = arena::new_vec(1);
        for t in &self.tables {
            // if schema_name != None && schema_name != Some(&*t.skema) {continue;}
            if !t.columns.iter().any(|c|**c == **col_name) {continue;}
            matching_tables.push(t);
        };
        
        // report errors
        if matching_tables.len() < 1 { return query_error!(line, 
            "Unresolved reference error. There is no table containing the column {}",
            col_name, 
        )};
        // let analyzer resolve arrows
        Ok(matching_tables.as_pia())
    }


    pub fn table_by_prefixed_tab_name(&self, line: u64, schema_name: Option<&str>, table_name: &str) -> Fallible<&Table> {
        // filter tables
        let mut matching_tables: Vec<&Table> = Vec::with_capacity(1);
        for t in &self.tables {
            if schema_name != None && schema_name != Some(&t.skema) {continue;}
            if table_name != &t.name {continue;}
            matching_tables.push(t);
        };
        // report errors
        if matching_tables.len() > 1 { return query_error!(line, 
            "Ambiguity error. {}.{} can describe multiple tables. Matching tables = {:?}",
            schema_name.unwrap_or("None"), table_name, matching_tables,
        )};
        if matching_tables.len() < 1 { return query_error!(line, 
            "Unresolved reference error. There is no table matching {}.{}",
            schema_name.unwrap_or("None"), table_name
        )};
        // return
        let table = matching_tables[0];
        Ok(table)
    }

    pub fn needs_skema_prefix(&self, tab: &Table) -> bool {
        self.tables.iter().filter(|t| t.name == tab.name).count() > 1
    }

    pub fn related_tables(&self, tab: &Table) -> Vec<&Table> {
        let tab_indices = &self.table_to_related_tables[&tab.idx];
        // let tabs = arena::alloc_array(tab_indices.len(), |i| &self.tables[tab_indices[i]]);
        let mut tabs = Vec::with_capacity(tab_indices.len());
        for &tab_idx in tab_indices {
            tabs.push(&self.tables[tab_idx.raw as usize]);
        }
        tabs
    }
    #[inline] pub fn get_dialect(&self) -> &dyn Dialect { &*self.dialect }
    #[inline] pub fn table_by_idx(&self, idx: TabIdx) -> &Table {&self.tables[idx.raw as usize]}
    
    #[cfg(test)] #[inline] pub fn make_tab_idx(&self, raw: u32) -> TabIdx {
        TabIdx {raw: raw, #[cfg(debug_assertions)] name_idx: raw as usize} // raw for name_idx?
    }

}





impl fmt::Debug for Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            return write!(f, "Tab '{}'", self.name);
    }
}

impl fmt::Debug for TabIdx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.raw == u32::MAX {return write!(f, "Unititialized NodeIdx");}
        #[cfg(debug_assertions)] {
            let name = tab_names.with_borrow(|r| {
                r[self.name_idx].clone()
            });
            return write!(f, "TabIdx {} '{}'", self.raw, name); 
        }
        #[cfg(not(debug_assertions))] write!(f, "TabIdx {}", self.raw)
    }
}

// impl fmt::Display for TabIdx { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{:?}", self) } }
impl PartialEq for TabIdx { fn eq(&self, other: &Self) -> bool { self.raw == other.raw } }
impl Hash for TabIdx { fn hash<H: Hasher>(&self, state: &mut H) { self.raw.hash(state); } }



/*
col: schema, table, column
pk:  schema, table, primary_key_col
fk:  schema, referenced_schema, table, referenced_table, primary_key_col, foreign_key_col

        columns_query = """
            SELECT 
                schema.nspname as schema_name,
                tab.relname as table_name,
                col.attname as column_name
            FROM pg_namespace  AS schema
            JOIN pg_class      AS tab ON tab.relnamespace = schema.oid
            JOIN pg_attribute  AS col ON col.attrelid = tab.oid    
            WHERE col.attnum > 0 -- exclude system columns
                and not col.attisdropped    
                and tab.relkind = 'r' -- filter out non-table objects in pg_class (e.g. views, sequences etc.)
                and schema.nspname not in ('pg_catalog', 'pg_toast', 'information_schema')
            """,

        primary_keys_query = """
            SELECT
                schema.nspname                          AS schema,
                tab.relname                             AS table,
                string_agg(distinct col.attname,  ', ') AS primary_key_col
            FROM pg_constraint AS con
            JOIN pg_class      AS tab     ON tab.oid = con.conrelid
            JOIN pg_namespace  AS schema  ON schema.oid = tab.relnamespace
            JOIN pg_attribute  AS col     ON col.attnum = ANY(con.conkey) AND col.attrelid = con.conrelid
            WHERE con.contype = 'p'     -- only select foreign keys, not any other constraints
                and col.attnum > 0      -- exclude system columns
                and tab.relkind = 'r'   -- filter out non-table objects in pg_class (e.g. views, sequences etc.)
                and not col.attisdropped 
            GROUP BY schema.nspname, tab.relname -- anything but fk, pk
        """, # used by insert

        foreign_keys_query = """
            SELECT
                schema.nspname                          AS schema,
                fschema.nspname                         AS referenced_schema,
                tab.relname                             AS table,
                ftab.relname                            AS referenced_table,
                string_agg(distinct col.attname,  ', ') AS primary_key_col, 
                string_agg(distinct fcol.attname, ', ') AS foreign_key_col  
            FROM pg_constraint AS con
            JOIN pg_class      AS tab     ON tab.oid = con.conrelid
            JOIN pg_namespace  AS schema  ON schema.oid = tab.relnamespace
            JOIN pg_attribute  AS col     ON col.attnum = ANY(con.conkey) AND col.attrelid = con.conrelid
            JOIN pg_class      AS ftab    ON ftab.oid = con.confrelid
            JOIN pg_namespace  AS fschema ON fschema.oid = ftab.relnamespace
            JOIN pg_attribute  AS fcol    ON fcol.attnum = ANY(con.confkey) AND fcol.attrelid = con.confrelid
            WHERE con.contype = 'f'     -- only select foreign keys, not any other constraints
                and col.attnum > 0      -- exclude system columns
                and fcol.attnum > 0     -- exclude system columns
                and tab.relkind = 'r'   -- filter out non-table objects in pg_class (e.g. views, sequences etc.)
                and ftab.relkind = 'r'  -- filter out non-table objects in pg_class (e.g. views, sequences etc.)
                and not col.attisdropped 
                and not fcol.attisdropped 
            GROUP BY schema.nspname, fschema.nspname, tab.relname, ftab.relname -- anything but fk, pk
            """,
*/

