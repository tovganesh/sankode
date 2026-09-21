use crate::span::Span;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords (शब्दसङ्ग्रह)
    Kriya,      // क्रिया (fn/def)
    Mana,       // मान (let)
    Vikarya,    // विकार्य (mut)
    Sthira,     // स्थिर (const)
    Samracana,  // संरचना (struct)
    Vikalpa,    // विकल्प (enum)
    Guna,       // गुण (trait)
    Vidhana,    // विधान (impl)
    Yadi,       // यदि (if)
    Anyatha,    // अन्यथा (else)
    Yavat,      // यावत् (while)
    Pratyeka,   // प्रत्येक (for each)
    Iti,        // इति (end / block close)
    Prati,      // प्रति (return)
    Bhanga,     // भङ्ग (break)
    Anuvrtta,   // अनुवृत्त (continue)
    Rna,        // ऋण (& / borrow)
    CalaRna,    // चलऋण (&mut)
    Satyam,     // सत्यम् (true)
    Mithya,     // मिथ्या (false)

    // Literals (मान)
    DevanagariInteger(i64, String), // e.g. १० -> 10, with raw Devanagari text
    DevanagariFloat(f64, String),   // e.g. ३.१४ -> 3.14
    StringLiteral(String),          // "नमस्ते"
    Identifier(String),             // मुख्य, फिबोनाची

    // Operators & Punctuation
    Danda,        // । (U+0964 - statement terminator)
    DoubleDanda,  // ॥ (U+0965 - section marker / comment / module boundary)
    Plus,         // +
    Minus,        // -
    Star,         // *
    Slash,        // /
    Percent,      // %
    Equal,        // =
    EqualEqual,   // ==
    NotEqual,     // !=
    Less,         // <
    LessEqual,    // <=
    Greater,      // >
    GreaterEqual, // >=
    Arrow,        // ->
    Comma,        // ,
    Colon,        // :
    Dot,          // .
    LParen,       // (
    RParen,       // )
    LBracket,     // [
    RBracket,     // ]

    // EOF
    Eof,
}

impl TokenKind {
    /// Maps a Sanskrit keyword string to its TokenKind
    pub fn from_keyword(s: &str) -> Option<TokenKind> {
        match s {
            "क्रिया" => Some(TokenKind::Kriya),
            "मान" => Some(TokenKind::Mana),
            "विकार्य" => Some(TokenKind::Vikarya),
            "स्थिर" => Some(TokenKind::Sthira),
            "संरचना" => Some(TokenKind::Samracana),
            "विकल्प" => Some(TokenKind::Vikalpa),
            "गुण" => Some(TokenKind::Guna),
            "विधान" => Some(TokenKind::Vidhana),
            "यदि" => Some(TokenKind::Yadi),
            "अन्यथा" => Some(TokenKind::Anyatha),
            "यावत्" => Some(TokenKind::Yavat),
            "प्रत्येक" => Some(TokenKind::Pratyeka),
            "इति" => Some(TokenKind::Iti),
            "प्रति" => Some(TokenKind::Prati),
            "भङ्ग" => Some(TokenKind::Bhanga),
            "अनुवृत्त" => Some(TokenKind::Anuvrtta),
            "ऋण" => Some(TokenKind::Rna),
            "चलऋण" => Some(TokenKind::CalaRna),
            "सत्यम्" => Some(TokenKind::Satyam),
            "मिथ्या" => Some(TokenKind::Mithya),
            _ => None,
        }
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Kriya => write!(f, "क्रिया"),
            TokenKind::Mana => write!(f, "मान"),
            TokenKind::Vikarya => write!(f, "विकार्य"),
            TokenKind::Sthira => write!(f, "स्थिर"),
            TokenKind::Samracana => write!(f, "संरचना"),
            TokenKind::Vikalpa => write!(f, "विकल्प"),
            TokenKind::Guna => write!(f, "गुण"),
            TokenKind::Vidhana => write!(f, "विधान"),
            TokenKind::Yadi => write!(f, "यदि"),
            TokenKind::Anyatha => write!(f, "अन्यथा"),
            TokenKind::Yavat => write!(f, "यावत्"),
            TokenKind::Pratyeka => write!(f, "प्रत्येक"),
            TokenKind::Iti => write!(f, "इति"),
            TokenKind::Prati => write!(f, "प्रति"),
            TokenKind::Bhanga => write!(f, "भङ्ग"),
            TokenKind::Anuvrtta => write!(f, "अनुवृत्त"),
            TokenKind::Rna => write!(f, "ऋण"),
            TokenKind::CalaRna => write!(f, "चलऋण"),
            TokenKind::Satyam => write!(f, "सत्यम्"),
            TokenKind::Mithya => write!(f, "मिथ्या"),
            TokenKind::DevanagariInteger(v, raw) => write!(f, "{}({})", raw, v),
            TokenKind::DevanagariFloat(v, raw) => write!(f, "{}({})", raw, v),
            TokenKind::StringLiteral(s) => write!(f, "\"{}\"", s),
            TokenKind::Identifier(id) => write!(f, "{}", id),
            TokenKind::Danda => write!(f, "।"),
            TokenKind::DoubleDanda => write!(f, "॥"),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::Equal => write!(f, "="),
            TokenKind::EqualEqual => write!(f, "=="),
            TokenKind::NotEqual => write!(f, "!="),
            TokenKind::Less => write!(f, "<"),
            TokenKind::LessEqual => write!(f, "<="),
            TokenKind::Greater => write!(f, ">"),
            TokenKind::GreaterEqual => write!(f, ">="),
            TokenKind::Arrow => write!(f, "->"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Dot => write!(f, "."),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::LBracket => write!(f, "["),
            TokenKind::RBracket => write!(f, "]"),
            TokenKind::Eof => write!(f, "<अन्त>"),
        }
    }
}
