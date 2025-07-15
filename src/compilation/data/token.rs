use std::hash::{Hash, Hasher};
use std::fmt;
use sqlparser::tokenizer::Location;
use crate::{tools::arena::*};

// ----------- data ----------- //

#[derive(Eq, PartialEq, Clone, Copy, Hash, PartialOrd)] // Display, Debug
pub enum TokTy {
    Ident,
    Placeholder,
    // Ignored,
    // Generated,
    Dot, LParen, RParen, SemiColon, TripleDot, Arrow,
    Across, Schemas, 
    Select, From, //AfterFrom, // Join, Where, Group, Order, By, Limit,
    Insert, Into, Values,
    Update, 
    Delete,
    // With,
    Eof,
}

#[derive(Eq, Clone, Copy, PartialOrd)] // Display, Debug, PartialEq, Hash
pub struct TokIdx {
    raw: u32, 
    #[cfg(debug_assertions)] tok_ty: TokTy, 
    #[cfg(debug_assertions)] loc: Location, // start location, with lin, column 1-indexed.
    #[cfg(debug_assertions)] text: Pia<str>, // extra indirection
    // #[cfg(debug_assertions)] ignored: Pia<StringA> // ignored stuff just prior to token
}
// pub type TokIdx = usize;

#[derive(PartialEq, Eq)]
pub struct TokData {
    pub tok_types: Pia<[TokTy]>, 
    pub locations: Pia<[Location]>, // start location, with lin, column 1-indexed.
    pub texts: Pia<[StringA]>, // extra indirection
    pub ignored: Pia<[StringA]> // ignored stuff just prior to token
}


// ----------- functions ----------- //

impl TokData {
    // pub fn tok_ty(&self, i: TokIdx) -> TokTy { self.tok_types[i] }

    pub fn shallow_copy(&self) -> Self {
        TokData {
            tok_types: self.tok_types.clone(),
            locations: self.locations.clone(),
            texts: self.texts.clone(),
            ignored: self.ignored.clone(),
        }
    }
    // pub fn deep_copy(&self) -> Self {
    //     TokData {
    //         tok_types: self.tok_types.deep_copy_array(),
    //         locations: self.locations.deep_copy_array(),
    //         texts: self.texts.deep_copy_array(),
    //         ignored: self.texts.deep_copy_array(),
    //     }
    // }

    #[inline(always)] pub fn tok_types(&self, tok_idx: TokIdx) -> TokTy {self.tok_types[tok_idx.raw as usize]}
    #[inline(always)] pub fn line(&self, tok_idx: TokIdx) -> u64 {self.locations[tok_idx.raw as usize].line}
    #[inline(always)] pub fn texts(&self, tok_idx: TokIdx) -> &StringA {&self.texts[tok_idx.raw as usize]}
    // #[inline(always)] pub fn ignored(&self, tok_idx: TokIdx) -> &StringA {&self.ignored[tok_idx.raw as usize]}

}


impl TokIdx {
    pub fn new(tok_data: &TokData, raw: usize) -> Self { TokIdx {
        raw: raw as u32, 
        #[cfg(debug_assertions)] tok_ty: tok_data.tok_types[raw],
        #[cfg(debug_assertions)] loc: tok_data.locations[raw],
        #[cfg(debug_assertions)] text: tok_data.texts[raw].as_pia(),
        // #[cfg(debug_assertions)] ignored: tok_data.ignored[raw].as_pia(),
    }}
    #[inline(always)] pub fn raw(&self, inc: u32) -> usize {(self.raw + inc) as usize}
}

impl fmt::Debug for TokIdx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(debug_assertions)] {
            let out = format!("{{{:?} '{}' {}:{}}}", self.tok_ty, &*self.text, self.loc.line, self.loc.column);
            return write!(f, "{}", out);
        }
        #[cfg(not(debug_assertions))] write!(f, "TokIdx {}", self.raw)
    }
}
impl fmt::Debug for TokTy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use TokTy::*;
        match self {
            Ident       => write!(f, "`identifier`"),
            Placeholder => write!(f, "`placeholder`"),
            // Generated   => write!(f, "`generated tokens`"),
            Dot         => write!(f, "`.`"),
            LParen      => write!(f, "`(`"),
            RParen      => write!(f, "`)`"),
            SemiColon   => write!(f, "`;`"),
            TripleDot   => write!(f, "`...`"),
            Arrow       => write!(f, "`->`"),
            Select      => write!(f, "`select`"),
            From        => write!(f, "`from`"),
            Insert      => write!(f, "`insert`"),
            Into        => write!(f, "`into`"),
            Values      => write!(f, "`values`"),
            Update      => write!(f, "`update`"),
            Delete      => write!(f, "`delete`"),
            Eof         => write!(f, "`end of file`"),
            Across      => write!(f, "`across`"),
            Schemas     => write!(f, "`schemas`"),
        }
    }
}

// ----------- traits ----------- //

impl fmt::Display for TokTy { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{:?}", self) } }
impl fmt::Display for TokIdx { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{:?}", self) } }
impl fmt::Display for TokData { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{:?}", self) } }
impl PartialEq for TokIdx { fn eq(&self, other: &Self) -> bool { self.raw == other.raw } }
impl Hash for TokIdx { fn hash<H: Hasher>(&self, state: &mut H) { self.raw.hash(state); } }

impl fmt::Debug for TokData { 
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { 
        let mut out = String::with_capacity(3 * self.tok_types.len());
        // let mut out = arena::new_string(3 * self.tok_types.len());
        for i in 0..self.tok_types.len() {
            let tok_ty = self.tok_types[i];
            let loc = self.locations[i];
            let text = &*self.texts[i];
            // if text == " " {continue;}
            out.push_str(&format!("[{}:{}] {} '{}'\n", loc.line, loc.column, tok_ty, text));
        }
        write!(f, "{}", out) 
    } 
}

