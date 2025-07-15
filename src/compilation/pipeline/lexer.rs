use sqlparser::tokenizer::{Location, Token, TokenWithSpan, Tokenizer};
use sqlparser::dialect::Dialect; // PostgreSqlDialect, 
use sqlparser::keywords::Keyword;
use crate::tools::err::Fallible;
use super::*;
use crate::*;
use crate::tools::arena;
use crate::tools::arena::*;


pub fn tokenize(source: &str, dialect: &dyn Dialect) -> Fallible<TokData> {
    // evt use tokenize_with_location_into_buf or tokenize_with_location
    let mut tokenizer = Tokenizer::new(dialect, source);
    let tokens = match tokenizer.tokenize_with_location() {
        Err(e) => return query_error!(e.location.line, "{}", e.message),
        Ok(t) => t,
    };

    let count_guess = tokens.len() / 2;
    let mut tok_types = arena::new_vec(count_guess);
    let mut locations = arena::new_vec(count_guess);
    let mut texts = arena::new_vec(count_guess);
    let mut ignored = arena::new_vec(count_guess);

    let mut i = 0;
    while i <= tokens.len() && tok_types.last() != Some(&TokTy::Eof) {
        // loop over ignored until we hit the relevant token or the eof
        let (tok_typ, loc, text, ignored_) = token_with_ignored_prefix(&tokens, &mut i)?;
        tok_types.push(tok_typ);
        locations.push(loc);
        texts.push(text);
        ignored.push(ignored_);

        if step(&tokens, &mut i) == None {break;}
    }

    return Ok(TokData {
        tok_types: tok_types.as_pia(),
        locations: locations.as_pia(),
        texts: texts.as_pia(),
        ignored: ignored.as_pia(),
    });
}    



#[inline] fn token_with_ignored_prefix(tokens: &Vec<TokenWithSpan>, i: &mut usize
) -> Fallible<(TokTy, Location, StringA, StringA)> {
    let mut tok_typ: Option<TokTy> = None;
    let mut loc: Location = Location {line: u64::MAX, column: u64::MAX};
    let mut text: StringA = arena::new_string(0);
    let mut ignored_: StringA = arena::new_string(0);
    // loop over ignored until we hit the relevant token or the eof
    loop {
        if *i == tokens.len() {break;}
        let tok_span = &tokens[*i];
        loc = tok_span.span.start;
        (tok_typ, text) = classify_tok(&tok_span.token);
        match tok_typ {
            Some(TokTy::Dot) => {
                tok_typ = Some(one_or_3_dots(&tokens, i, &mut text)?);
                break;
            }
            Some(_) => {break;}
            None => {
                ignored_.push_str(&text); 
                step(&tokens, i);
            },
        }
    }
    if *i == tokens.len() { // handle eof
        tok_typ = Some(TokTy::Eof);
        loc = tokens.last().unwrap().span.end;
        text = "".as_string_a();
    }
    dbg_assert!(tok_typ != None 
        && loc != Location {line: u64::MAX, column: u64::MAX}
        && (text.len() != 0 || tok_typ == Some(TokTy::Eof)), 
        format!("{:?}, {}, {}", tok_typ, loc, text));
    Ok((tok_typ.unwrap(), loc, text, ignored_))
}


#[inline] fn classify_tok(tok: &Token) -> (Option<TokTy>, StringA) {   
    use Keyword::*;
    let tok_ty = match tok {
        Token::Placeholder(_) => Some(TokTy::Placeholder), // has relevant text segment
        Token::EOF => Some(TokTy::Eof),
        Token::SemiColon => Some(TokTy::SemiColon),
        Token::Period => Some(TokTy::Dot),
        Token::Arrow => Some(TokTy::Arrow),
        Token::LParen => Some(TokTy::LParen),
        Token::RParen => Some(TokTy::RParen),
        Token::Word(w) => { match w.keyword { // w.value contains the actual text, in case location isn't enough 
            SELECT => Some(TokTy::Select), 
            FROM => Some(TokTy::From), 
            // JOIN | WHERE | GROUP | ORDER | BY | LIMIT => Some(TokTy::AfterFrom),
            INSERT => Some(TokTy::Insert), 
            INTO => Some(TokTy::Into), 
            VALUES => Some(TokTy::Values),
            UPDATE => Some(TokTy::Update), 
            DELETE => Some(TokTy::Delete),
            // WITH => TokTy::With,
            // EMPTY => TokTy::Ident, // contrary to the docs. empty doesn't mean NoKeyword, so it is ignored.

            SCHEMAS => Some(TokTy::Schemas),
            NoKeyword => { match &*w.value {
                "across" => Some(TokTy::Across),
                // "schemas" => Some(TokTy::Ident),
                _ => Some(TokTy::Ident),
            }}, 
            _ => None, // whitespace or ignored keyword
        }}
        _ => None, // whitespace or ignored token
    };
    let text = if tok_ty != Some(TokTy::Eof) {&tok.to_string()} else {""};
    // let text = StringA::from_str_in(text, arena::arena_gen());
    let text = text.as_string_a();
    
    // let text = match tok {
    //     Token::Placeholder(s) => s,
    //     Token::Word(w) => w.to_string(), // to_string includes quotes around quoted identifiers.
    //     _ => String::new(), // has relevant text segment
    // };

    return (tok_ty, text);
}


#[inline] fn one_or_3_dots(tokens: &Vec<TokenWithSpan>, i: &mut usize, text: &mut StringA) -> Fallible<TokTy> {
    let next_is_dot = |i: usize| {i + 1 < tokens.len() && tokens[i + 1].token == Token::Period};

    dbg_assert!(tokens[*i].token == Token::Period); // 1 dot
    if !next_is_dot(*i) { return Ok(TokTy::Dot); }

    step(&tokens, i); // 2 dots
    if !next_is_dot(*i) {return query_error!(tokens[*i].span.start.line, 
        "Expected one or three dots in a row, found two."
    )};
    text.push_str(&tokens[*i].token.to_string());

    step(&tokens, i); // 3 dots
    text.push_str(&tokens[*i].token.to_string());

    if next_is_dot(*i) {return query_error!(tokens[*i].span.start.line, 
        "Expected one or three dots in a row, found four."
    )};

    Ok(TokTy::TripleDot)
}


#[inline] fn step(tokens: &Vec<TokenWithSpan>, i: &mut usize) -> Option<()> {
    #[cfg(debug_assertions)] { // check that start == prev_end if prev_end exist
        let i = *i;
        if i > 0 && i < tokens.len() {
            let start = tokens[i].span.start; 
            let prev_end = tokens[i - 1].span.end;
            dbg_assert!(start.line == prev_end.line && start.column == prev_end.column, format!(
                "end of previous token doesn't match start of next token. prev_end = {:?}. start = {:?}",
                prev_end, start
            ));
        }
    };
    *i += 1;
    Some(())
}


