use sankode_core::{
    is_devanagari_digit, parse_devanagari_i64, Span, Token, TokenKind,
};
use thiserror::Error;
use unicode_normalization::UnicodeNormalization;

#[derive(Error, Debug, PartialEq)]
pub enum LexError {
    #[error("अमान्यः वर्णः (Unexpected character) '{0}' at {1}")]
    UnexpectedChar(char, Span),
    #[error("असमाप्तम् सूत्रम् (Unterminated string literal) at {0}")]
    UnterminatedString(Span),
    #[error("अमान्या संख्या (Invalid number) '{0}' at {1}")]
    InvalidNumber(String, Span),
}

pub struct Lexer<'a> {
    source: &'a str,
    chars: Vec<(usize, char)>, // (byte_offset, char)
    cursor: usize,
    line: usize,
    column: usize,
}

pub fn is_devanagari_ident_start(c: char) -> bool {
    ('\u{0904}'..='\u{0939}').contains(&c)
        || ('\u{0958}'..='\u{0961}').contains(&c)
        || ('\u{0972}'..='\u{097F}').contains(&c)
        || c == '_'
}

pub fn is_devanagari_ident_continue(c: char) -> bool {
    is_devanagari_ident_start(c)
        || ('\u{093E}'..='\u{094C}').contains(&c)
        || ('\u{0962}'..='\u{0963}').contains(&c)
        || c == '\u{094D}' // virama / halant
        || ('\u{0901}'..='\u{0903}').contains(&c) // anusvara, visarga, candrabindu
        || c == '\u{093C}' // nukta
        || is_devanagari_digit(c)
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        let chars: Vec<(usize, char)> = source.char_indices().collect();
        Self {
            source,
            chars,
            cursor: 0,
            line: 1,
            column: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.cursor).map(|(_, c)| *c)
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.cursor + 1).map(|(_, c)| *c)
    }

    fn advance(&mut self) -> Option<char> {
        if let Some((_, c)) = self.chars.get(self.cursor) {
            let ch = *c;
            self.cursor += 1;
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            Some(ch)
        } else {
            None
        }
    }

    fn current_span(&self, start_line: usize, start_col: usize, start_offset: usize) -> Span {
        let end_offset = if self.cursor < self.chars.len() {
            self.chars[self.cursor].0
        } else {
            self.source.len()
        };
        Span::new(start_offset, end_offset, start_line, start_col)
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();

        loop {
            self.skip_whitespace();
            if self.cursor >= self.chars.len() {
                let span = Span::new(self.source.len(), self.source.len(), self.line, self.column);
                tokens.push(Token {
                    kind: TokenKind::Eof,
                    span,
                });
                break;
            }

            let start_line = self.line;
            let start_col = self.column;
            let start_offset = self.chars[self.cursor].0;

            let c = self.advance().unwrap();

            // Comment starting with Double Danda ॥
            if c == '॥' {
                // Read until next ॥ or newline
                let mut comment = String::new();
                while let Some(ch) = self.peek() {
                    if ch == '॥' {
                        self.advance();
                        break;
                    } else if ch == '\n' {
                        break;
                    } else {
                        comment.push(self.advance().unwrap());
                    }
                }
                // Skip comments from the primary token stream
                continue;
            }

            // Single Danda (statement terminator)
            if c == '।' {
                tokens.push(Token {
                    kind: TokenKind::Danda,
                    span: self.current_span(start_line, start_col, start_offset),
                });
                continue;
            }

            // Operators & Single characters
            match c {
                '(' => tokens.push(Token {
                    kind: TokenKind::LParen,
                    span: self.current_span(start_line, start_col, start_offset),
                }),
                ')' => tokens.push(Token {
                    kind: TokenKind::RParen,
                    span: self.current_span(start_line, start_col, start_offset),
                }),
                '[' => tokens.push(Token {
                    kind: TokenKind::LBracket,
                    span: self.current_span(start_line, start_col, start_offset),
                }),
                ']' => tokens.push(Token {
                    kind: TokenKind::RBracket,
                    span: self.current_span(start_line, start_col, start_offset),
                }),
                ',' => tokens.push(Token {
                    kind: TokenKind::Comma,
                    span: self.current_span(start_line, start_col, start_offset),
                }),
                ':' => tokens.push(Token {
                    kind: TokenKind::Colon,
                    span: self.current_span(start_line, start_col, start_offset),
                }),
                '.' => tokens.push(Token {
                    kind: TokenKind::Dot,
                    span: self.current_span(start_line, start_col, start_offset),
                }),
                '+' => tokens.push(Token {
                    kind: TokenKind::Plus,
                    span: self.current_span(start_line, start_col, start_offset),
                }),
                '-' => {
                    if self.peek() == Some('>') {
                        self.advance();
                        tokens.push(Token {
                            kind: TokenKind::Arrow,
                            span: self.current_span(start_line, start_col, start_offset),
                        });
                    } else {
                        tokens.push(Token {
                            kind: TokenKind::Minus,
                            span: self.current_span(start_line, start_col, start_offset),
                        });
                    }
                }
                '*' => tokens.push(Token {
                    kind: TokenKind::Star,
                    span: self.current_span(start_line, start_col, start_offset),
                }),
                '/' => tokens.push(Token {
                    kind: TokenKind::Slash,
                    span: self.current_span(start_line, start_col, start_offset),
                }),
                '%' => tokens.push(Token {
                    kind: TokenKind::Percent,
                    span: self.current_span(start_line, start_col, start_offset),
                }),
                '=' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        tokens.push(Token {
                            kind: TokenKind::EqualEqual,
                            span: self.current_span(start_line, start_col, start_offset),
                        });
                    } else {
                        tokens.push(Token {
                            kind: TokenKind::Equal,
                            span: self.current_span(start_line, start_col, start_offset),
                        });
                    }
                }
                '!' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        tokens.push(Token {
                            kind: TokenKind::NotEqual,
                            span: self.current_span(start_line, start_col, start_offset),
                        });
                    } else {
                        return Err(LexError::UnexpectedChar(
                            c,
                            self.current_span(start_line, start_col, start_offset),
                        ));
                    }
                }
                '<' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        tokens.push(Token {
                            kind: TokenKind::LessEqual,
                            span: self.current_span(start_line, start_col, start_offset),
                        });
                    } else {
                        tokens.push(Token {
                            kind: TokenKind::Less,
                            span: self.current_span(start_line, start_col, start_offset),
                        });
                    }
                }
                '>' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        tokens.push(Token {
                            kind: TokenKind::GreaterEqual,
                            span: self.current_span(start_line, start_col, start_offset),
                        });
                    } else {
                        tokens.push(Token {
                            kind: TokenKind::Greater,
                            span: self.current_span(start_line, start_col, start_offset),
                        });
                    }
                }
                '"' => {
                    // String literal
                    let mut s = String::new();
                    let mut closed = false;
                    while let Some(ch) = self.peek() {
                        if ch == '"' {
                            self.advance();
                            closed = true;
                            break;
                        } else if ch == '\\' {
                            self.advance();
                            if let Some(escaped) = self.advance() {
                                match escaped {
                                    'n' => s.push('\n'),
                                    't' => s.push('\t'),
                                    'r' => s.push('\r'),
                                    '\\' => s.push('\\'),
                                    '"' => s.push('"'),
                                    other => s.push(other),
                                }
                            }
                        } else {
                            s.push(self.advance().unwrap());
                        }
                    }
                    if !closed {
                        return Err(LexError::UnterminatedString(
                            self.current_span(start_line, start_col, start_offset),
                        ));
                    }
                    tokens.push(Token {
                        kind: TokenKind::StringLiteral(s),
                        span: self.current_span(start_line, start_col, start_offset),
                    });
                }
                _ => {
                    // Check if it's a Devanagari digit
                    if is_devanagari_digit(c) {
                        let mut num_str = String::new();
                        num_str.push(c);
                        while let Some(ch) = self.peek() {
                            if is_devanagari_digit(ch) {
                                num_str.push(self.advance().unwrap());
                            } else {
                                break;
                            }
                        }

                        // Check for fractional part: dot followed by digits
                        if self.peek() == Some('.') && self.peek_next().map(is_devanagari_digit).unwrap_or(false) {
                            self.advance(); // consume '.'
                            num_str.push('.');
                            while let Some(ch) = self.peek() {
                                if is_devanagari_digit(ch) {
                                    num_str.push(self.advance().unwrap());
                                } else {
                                    break;
                                }
                            }
                            let span = self.current_span(start_line, start_col, start_offset);
                            // Parse Devanagari float
                            let western_str: String = num_str
                                .chars()
                                .map(|ch| {
                                    if let Some(d) = sankode_core::devanagari_digit_to_u32(ch) {
                                        char::from_digit(d, 10).unwrap()
                                    } else {
                                        ch
                                    }
                                })
                                .collect();
                            let float_val: f64 = western_str
                                .parse()
                                .map_err(|_| LexError::InvalidNumber(num_str.clone(), span))?;
                            tokens.push(Token {
                                kind: TokenKind::DevanagariFloat(float_val, num_str),
                                span,
                            });
                        } else {
                            let span = self.current_span(start_line, start_col, start_offset);
                            let val = parse_devanagari_i64(&num_str)
                                .map_err(|_| LexError::InvalidNumber(num_str.clone(), span))?;
                            tokens.push(Token {
                                kind: TokenKind::DevanagariInteger(val, num_str),
                                span,
                            });
                        }
                    } else if is_devanagari_ident_start(c) || c.is_alphabetic() {
                        let mut ident = String::new();
                        ident.push(c);
                        while let Some(ch) = self.peek() {
                            if is_devanagari_ident_continue(ch) || ch.is_alphanumeric() {
                                ident.push(self.advance().unwrap());
                            } else {
                                break;
                            }
                        }

                        // Normalize to NFKC
                        let normalized: String = ident.nfkc().collect();
                        let span = self.current_span(start_line, start_col, start_offset);

                        if let Some(keyword) = TokenKind::from_keyword(&normalized) {
                            tokens.push(Token { kind: keyword, span });
                        } else {
                            tokens.push(Token {
                                kind: TokenKind::Identifier(normalized),
                                span,
                            });
                        }
                    } else {
                        return Err(LexError::UnexpectedChar(
                            c,
                            self.current_span(start_line, start_col, start_offset),
                        ));
                    }
                }
            }
        }

        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_hello_world() {
        let code = r#"
॥ नमस्ते जगत् ॥
क्रिया मुख्य() -> रिक्त
    मुद्रय("नमस्ते जगत्!")।
    मान गणना = १०।
इति
"#;
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize().expect("Tokenization failed");

        assert_eq!(tokens[0].kind, TokenKind::Kriya);
        assert_eq!(tokens[1].kind, TokenKind::Identifier("मुख्य".to_string()));
        assert_eq!(tokens[2].kind, TokenKind::LParen);
        assert_eq!(tokens[3].kind, TokenKind::RParen);
        assert_eq!(tokens[4].kind, TokenKind::Arrow);
        assert_eq!(tokens[5].kind, TokenKind::Identifier("रिक्त".to_string()));
        assert_eq!(tokens[6].kind, TokenKind::Identifier("मुद्रय".to_string()));
        assert_eq!(tokens[7].kind, TokenKind::LParen);
        assert_eq!(
            tokens[8].kind,
            TokenKind::StringLiteral("नमस्ते जगत्!".to_string())
        );
        assert_eq!(tokens[9].kind, TokenKind::RParen);
        assert_eq!(tokens[10].kind, TokenKind::Danda);
        assert_eq!(tokens[11].kind, TokenKind::Mana);
        assert_eq!(tokens[12].kind, TokenKind::Identifier("गणना".to_string()));
        assert_eq!(tokens[13].kind, TokenKind::Equal);
        assert_eq!(
            tokens[14].kind,
            TokenKind::DevanagariInteger(10, "१०".to_string())
        );
        assert_eq!(tokens[15].kind, TokenKind::Danda);
        assert_eq!(tokens[16].kind, TokenKind::Iti);
        assert_eq!(tokens[17].kind, TokenKind::Eof);
    }
}
