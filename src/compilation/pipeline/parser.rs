use crate::tools::{arena, Pia, StringA, VecA, VecATExt};
use crate::tools::err::*;
use super::*;
use crate::*;
use super::Node::*;
use super::TokTy::*;
/*
Script -> (Query else pass)*
Query -> Selection | Insertion | Updation | Deletion | WithQuery
// (Query) is a nested query and thus either a subquery or a cte
// pass kunne evt hedde invariant, since they are treated the same way by every query 
pass -> ((identifier.)*identifier | (Query))*

// this from clause doesnt allow combining identifier... with join
Selection -> select json? multiple? to_indices? pass from (identifier... else pass) pass
Insertion -> insert into ((identifier... (key_fixes) (sql-insert or perhaps explicit values blah | Selection)+) else pass) 
Updation -> 
Deletion -> 
WithQuery -> With Script


// overwrite ??

Stmt -> Query | DDL // DDL is omitted in sapi, so query is root node
Block_with_paren -> (Query)
Block -> Selection | Insertion | Updation | Deletion
*/

pub fn parse(tok_data: TokData) -> Fallible<Ast> {
    let tok_idx = TokIdx::new(&tok_data, 0);
    let mut parser = Parser {
        tok_data, 
        tok_idx: tok_idx, 
        use_log: false,
        ast: Ast {
            nodes: arena::new_vec(16),
            names_stack: arena::new_vec(16),
        }
    };
    parser.parse_script()?;
    return Ok(parser.ast);
}

struct Parser {
    tok_data: TokData,
    tok_idx: TokIdx,
    use_log: bool,
    ast: Ast,
}

impl Parser {
    // Script -> (Query else pass)*
    pub fn parse_script(&mut self) -> Fallible<()> {
        let mut queries: VecA<NodeIdx> = arena::new_vec(1);
        let mut errors: VecA<Box<QueryError>> = arena::new_vec(0); // errors is also located in err.rs
        while self.tok_ty() != Eof {
            if [Select, Insert, Update, Delete].contains(&self.tok_ty()) {
                match self.parse_query() {
                    Ok((is_sapi, query_idx)) => { if is_sapi {queries.push(query_idx);}},
                    Err(e) => {errors.push(e);},
                }
            } else {self.step()?;}
            
        }
        dbg_assert!(queries.iter().all(|query_idx| matches!(self.ast.nodes[query_idx.raw], Selection {..} | Insertion {..})));
        dbg_assert!(self.ast.names_stack.iter().all(|(query_idx, _)| matches!(self.ast.nodes[query_idx.raw], Selection {..} | Insertion {..})));

        if !errors.is_empty() {
            let mut msg = String::with_capacity(errors.len() * 20);
            for e in errors {
                msg += &format!("{}", e);
            }
            return query_error!(0, "{}", msg);
        }
        self.ast.nodes.push(Script {queries: queries.as_pia()});
        Ok(())
    }

    // Query -> Selection | Insertion | Updation | Deletion | WithQuery
    fn parse_query(&mut self) -> Fallible<(bool, NodeIdx)> {
        self.log("parse_query");
        dbg_assert!([Select, Insert, Update, Delete].contains(&self.tok_ty()));
        let uninit = NodeIdx::new(&self.ast, usize::MAX);
        self.ast.names_stack.push((uninit, arena::new_vec(4))); // usize::MAX means uninit
        let fal_is_sapi = match &self.tok_ty() {
            Select => self.parse_select(), 
            Insert => self.parse_insert(), 
            Update => self.parse_update(), 
            Delete => self.parse_delete(), 
            // With => self.parse_with(), 
            _ => unreachable!(),
        };

        let query_idx = if self.ast.nodes.is_empty() {uninit} else {
            NodeIdx::new(&self.ast, self.ast.nodes.len() - 1)
        };
        if fal_is_sapi != Ok(true) { self.ast.names_stack.pop(); } // don't process non-sapi any further.
        else { self.ast.names_stack.last_mut().unwrap().0 = query_idx; } 
        
        Ok((fal_is_sapi?, query_idx))
    }

    // Selection -> select json? multiple? to_indices? pass from (identifier... else pass) pass
    fn parse_select(&mut self) -> Fallible<bool> {
        self.log("parse_select");
        self.expect(Select).assert_ok();
        // optionally check for qualifiers
        self.parse_until(&[From, SemiColon, Eof, RParen])?;
        if self.tok_ty() != From { 
            return Ok(false); 
        }
        self.expect(From).assert_ok();
        let root = self.parse_root()?;
        self.parse_until(&[SemiColon, Eof, RParen])?;

        if root == None { return Ok(false); }
        self.ast.nodes.push(Selection {root: root.unwrap() });
        Ok(true)
    }

    // Insertion -> insert into ((identifier... (key_fixes) (sql-insert or perhaps explicit values blah | Selection)+) else pass) 
    fn parse_insert(&mut self) -> Fallible<bool> {
        self.log("parse_insert");
        self.expect(Insert).assert_ok();
        self.expect(Into)?;
        let root = self.parse_root()?;
        self.parse_until(&[SemiColon, Eof, RParen])?;

        if root == None { return Ok(false); }
        self.ast.nodes.push(Insertion {root: root.unwrap() });
        Ok(true)
    }
    // Updation -> 
    fn parse_update(&mut self) -> Fallible<bool> {
        unimplemented!();
    }
    // Deletion -> 
    fn parse_delete(&mut self) -> Fallible<bool> {
        unimplemented!();
    }
    // // WithQuery -> With Script
    // fn parse_with(&mut self) -> Fallible<bool> {
    //     self.log("parse_with");
    //     self.expect(With).assert_ok();
    //     self.parse_script()
    // }
    



    // --------- helpers --------- //

    // pass -> ((identifier.)*identifier | Placeholder | (Query))*
    fn parse_until(&mut self, targets: &[TokTy]) -> Fallible<()> {
        self.log("parse_until");
        loop {
            match self.tok_ty() {
                Ident => { self.parse_ident()?; }, 
                Placeholder => unimplemented!(),
                LParen => { 
                    self.step()?; // eat (
                    if [Select, Insert, Update, Delete].contains(&self.tok_ty()) {
                        match self.parse_query() {
                            Ok(_) => {self.expect(RParen)?;},
                            Err(_) => {while self.step()? != RParen {};}
                        }
                    } else {
                        self.parse_until(&[RParen])?;
                        self.step()?; // eat )
                    }
                    dbg_assert!(self.tok_ty() != RParen);
                },
                // Generated => unreachable!(),
                _ if {targets.contains(&self.tok_ty())} => break, // dbg_assert!(false);
                _ => {self.wrong_token(targets)?;},
            }
            // dbg_assert!(self.tok_ty() != End);
        }
 
        Ok(())
    }

    fn parse_root(&mut self) -> Fallible<Option<PrefixedTab>> {
        self.log("parse_root");
        let (table, has_arrow) = self.parse_ident()?;
        dbg_assert!(table.len() > 0);
        if has_arrow { return query_error!(self.tok_data.line(self.tok_idx), 
            "cannot use arrow resolution in the table from clause. Found {:?}",
            table.iter().map(|i| self.tok_data.texts(*i)).collect::<Vec<&StringA>>()
        )};
        if table.len() > 2 { return query_error!(self.tok_data.line(self.tok_idx), 
            "table can have at most one prefix, as in skema.table. Found {:?}",
            table.iter().map(|i| self.tok_data.texts(*i)).collect::<Vec<&StringA>>()
        )};

        if self.tok_ty() != TripleDot {return Ok(None);}
        self.expect(TripleDot).assert_ok();

        let across_schemas = (self.tok_ty() == Across);
        if across_schemas {
            self.expect(Across).assert_ok();
            self.expect(Schemas)?;
        }

        let table: PrefixedTab = match table.len() {
            2 => PrefixedTab {schema: Some(table[0]), tab: table[1], across_schemas},
            1 => PrefixedTab {schema: None, tab: table[0], across_schemas},
            _ => unreachable!(),
        }; 
        Ok(Some(table))
    }
    
    fn parse_ident(&mut self) -> Fallible<Name> {
        self.log("parse_ident");
        let mut name_toks = arena::new_vec(1);
        // fn full_name_text(tok_data: TokData, name: Name) { name.iter().map(|n| tok_data.texts(*n)).collect();}
        
        while self.peek()? == Dot {
            name_toks.push(self.tok_idx);
            self.expect(Ident)?; // eat ident
            self.expect(Dot).assert_ok();

            if self.peek()? == Arrow { return query_error!(self.tok_data.line(self.tok_idx), 
                "Cannot mix dot and arrow prefixes. Found {:?}", 
                name_toks.iter().map(|n| self.tok_data.texts(*n)).collect::<Vec<&StringA>>()
            )};
            if name_toks.len() > 2 { return query_error!(self.tok_data.line(self.tok_idx), 
                "identifier can have at most two dot prefixes, as in skema.table.column. Found {:?}",
                name_toks.iter().map(|n| self.tok_data.texts(*n)).collect::<Vec<&StringA>>()
            )};
        }
        let mut has_arrow = false;
        while self.peek()? == Arrow {
            name_toks.push(self.tok_idx);
            self.expect(Ident)?; // eat ident
            self.expect(Arrow).assert_ok();

            if self.peek()? == Dot { return query_error!(self.tok_data.line(self.tok_idx), 
                "Cannot mix dot and arrow prefixes. Found {:?}", 
                name_toks.iter().map(|n| self.tok_data.texts(*n)).collect::<Vec<&StringA>>()
            )};
            has_arrow = true;
        }
        name_toks.push(self.tok_idx);
        self.expect(Ident)?; // eat ident

        let name = (name_toks.as_pia(), has_arrow);
        match self.ast.names_stack.last_mut() {
            None => {}, // we are not in a query, so we ignore the name
            Some((_, query_names)) => {
                query_names.push(name);
            }
        };
        Ok(name)
    }

    // #[inline] pub fn node_idx(&self) -> NodeIdx { 
    //     dbg_assert!(self.ast.nodes.len() > 0);
    //     self.ast.nodes.len() - 1 // the index of the last parsed node.
    // }
    #[inline] pub fn tok_ty(&self) -> TokTy { 
        // self.log("tok_ty");
        self.tok_data.tok_types(self.tok_idx)
    }
    #[inline] fn peek(&self) -> Fallible<TokTy> { 
        self.log("peek");
        if self.tok_ty() == TokTy::Eof {return query_error!(self.tok_data.line(self.tok_idx), "Expected token, found end of script.");};
        Ok(self.tok_data.tok_types[self.tok_idx.raw(1)])
    }
    fn step(&mut self) -> Fallible<TokTy> {
        self.log("step");
        if self.tok_ty() == TokTy::Eof {return query_error!(self.tok_data.line(self.tok_idx), "Expected token, found end of script.");};
        let old = self.tok_ty();
        self.tok_idx = TokIdx::new(&self.tok_data, self.tok_idx.raw(1));
        // self.log("step_to");
        return Ok(old);
    }
    fn expect(&mut self, tok_ty: TokTy) -> Fallible<TokTy> { 
        self.log("expect");
        if self.tok_ty() != tok_ty { 
            return self.wrong_token(&[tok_ty]);
        }
        return Ok(self.step()?);
    }

    fn wrong_token(&self, tok_tys: &[TokTy]) -> Fallible<TokTy> {
        self.log("wrong_token");
        let line = self.tok_data.line(self.tok_idx);
        let text = &self.tok_data.texts(self.tok_idx);
        let expected_text = if tok_tys.len() == 1 { 
            format!("{:?}", tok_tys[0])} else {format!("{:?}", tok_tys)};
        let e = query_error!(line, "Expected {}, found `{}`", expected_text, text);
        eprintln!("{}", e.clone().err().unwrap());
        e
    }
    

    #[inline] fn log(&self, func_name: &str) {
        if !self.use_log {return;}
        eprintln!("parser.{}{}", func_name, self.tok_idx);
    }

}


