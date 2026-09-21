use sankode_core::{
    BinaryOp, Expr, ExprKind, FunctionDecl, Param, Program, Span, Statement, Token, TokenKind,
    TopLevelItem, TypeAnnotation, UnaryOp,
};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ParseError {
    #[error("अपेक्षितः '{expected}', किन्तु प्राप्तः '{found:?}' at {span}")]
    UnexpectedToken {
        expected: String,
        found: TokenKind,
        span: Span,
    },
    #[error("अप्रत्याशितः सङ्केतान्तः (Unexpected EOF)")]
    UnexpectedEof,
    #[error("अमान्यं व्यञ्जनम् (Invalid expression) at {0}")]
    InvalidExpression(Span),
}

pub struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, cursor: 0 }
    }

    fn current(&self) -> &Token {
        if self.cursor < self.tokens.len() {
            &self.tokens[self.cursor]
        } else {
            self.tokens.last().unwrap()
        }
    }

    fn peek_kind(&self) -> &TokenKind {
        &self.current().kind
    }

    fn check(&self, kind: &TokenKind) -> bool {
        self.peek_kind() == kind
    }

    fn advance(&mut self) -> &Token {
        let prev = self.cursor;
        if self.cursor < self.tokens.len() {
            self.cursor += 1;
        }
        &self.tokens[prev.min(self.tokens.len() - 1)]
    }

    fn consume(&mut self, expected: TokenKind, desc: &str) -> Result<Token, ParseError> {
        let cur = self.current().clone();
        if cur.kind == expected {
            self.advance();
            Ok(cur)
        } else {
            Err(ParseError::UnexpectedToken {
                expected: desc.to_string(),
                found: cur.kind,
                span: cur.span,
            })
        }
    }

    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut items = Vec::new();

        while !self.check(&TokenKind::Eof) {
            match self.peek_kind() {
                TokenKind::Kriya => {
                    let func = self.parse_function()?;
                    items.push(TopLevelItem::Function(func));
                }
                TokenKind::DoubleDanda => {
                    self.advance();
                }
                _ => {
                    let cur = self.current();
                    return Err(ParseError::UnexpectedToken {
                        expected: "क्रिया (Function declaration)".to_string(),
                        found: cur.kind.clone(),
                        span: cur.span,
                    });
                }
            }
        }

        Ok(Program { items })
    }

    fn parse_function(&mut self) -> Result<FunctionDecl, ParseError> {
        let kriya_tok = self.consume(TokenKind::Kriya, "क्रिया")?;

        // Function name
        let name = match self.peek_kind() {
            TokenKind::Identifier(id) => {
                let id = id.clone();
                self.advance();
                id
            }
            _ => {
                let cur = self.current();
                return Err(ParseError::UnexpectedToken {
                    expected: "क्रिया-नाम (Function name)".to_string(),
                    found: cur.kind.clone(),
                    span: cur.span,
                });
            }
        };

        // Parameter list: (p1: T1, p2: T2)
        self.consume(TokenKind::LParen, "(")?;
        let mut params = Vec::new();
        if !self.check(&TokenKind::RParen) {
            loop {
                let p_span = self.current().span;
                let p_name = match self.advance().kind.clone() {
                    TokenKind::Identifier(id) => id,
                    other => {
                        return Err(ParseError::UnexpectedToken {
                            expected: "तर्क-नाम (Parameter name)".to_string(),
                            found: other,
                            span: p_span,
                        })
                    }
                };

                self.consume(TokenKind::Colon, ":")?;
                let type_ann = self.parse_type_annotation()?;

                params.push(Param {
                    name: p_name,
                    type_ann,
                    span: p_span,
                });

                if self.check(&TokenKind::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.consume(TokenKind::RParen, ")")?;

        // Optional return type: -> Type
        let return_type = if self.check(&TokenKind::Arrow) {
            self.advance();
            Some(self.parse_type_annotation()?)
        } else {
            None
        };

        // Function Body: statement list until 'इति'
        let mut body = Vec::new();
        while !self.check(&TokenKind::Iti) && !self.check(&TokenKind::Eof) {
            body.push(self.parse_statement()?);
        }

        let end_tok = self.consume(TokenKind::Iti, "इति (End of block)")?;

        Ok(FunctionDecl {
            name,
            params,
            return_type,
            body,
            span: kriya_tok.span.merge(end_tok.span),
        })
    }

    fn parse_type_annotation(&mut self) -> Result<TypeAnnotation, ParseError> {
        let cur = self.current().clone();
        match cur.kind {
            TokenKind::Identifier(name) => {
                self.advance();
                Ok(TypeAnnotation::Simple(name))
            }
            TokenKind::Rna => {
                self.advance();
                let inner = self.parse_type_annotation()?;
                Ok(TypeAnnotation::Reference(Box::new(inner)))
            }
            TokenKind::CalaRna => {
                self.advance();
                let inner = self.parse_type_annotation()?;
                Ok(TypeAnnotation::MutReference(Box::new(inner)))
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "प्रकारः (Type name)".to_string(),
                found: cur.kind,
                span: cur.span,
            }),
        }
    }

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        let cur = self.current().clone();
        match cur.kind {
            // मान [विकार्य] x [: T] = expr।
            TokenKind::Mana => {
                self.advance();
                let mut is_mut = false;
                if self.check(&TokenKind::Vikarya) {
                    self.advance();
                    is_mut = true;
                }

                let var_name = match self.peek_kind() {
                    TokenKind::Identifier(id) => {
                        let id = id.clone();
                        self.advance();
                        id
                    }
                    _ => {
                        return Err(ParseError::UnexpectedToken {
                            expected: "चर-नाम (Variable name)".to_string(),
                            found: self.peek_kind().clone(),
                            span: self.current().span,
                        });
                    }
                };

                let type_ann = if self.check(&TokenKind::Colon) {
                    self.advance();
                    Some(self.parse_type_annotation()?)
                } else {
                    None
                };

                self.consume(TokenKind::Equal, "=")?;
                let init = self.parse_expression()?;
                let danda = self.consume(TokenKind::Danda, "। (Danda terminator)")?;

                Ok(Statement::VarDecl {
                    name: var_name,
                    is_mut,
                    type_ann,
                    init,
                    span: cur.span.merge(danda.span),
                })
            }

            // यदि condition ... [अन्यथा ...] इति
            TokenKind::Yadi => {
                self.advance();
                let condition = self.parse_expression()?;
                let mut then_branch = Vec::new();
                while !self.check(&TokenKind::Anyatha)
                    && !self.check(&TokenKind::Iti)
                    && !self.check(&TokenKind::Eof)
                {
                    then_branch.push(self.parse_statement()?);
                }

                let mut else_branch = None;
                if self.check(&TokenKind::Anyatha) {
                    self.advance();
                    let mut else_stmts = Vec::new();
                    while !self.check(&TokenKind::Iti) && !self.check(&TokenKind::Eof) {
                        else_stmts.push(self.parse_statement()?);
                    }
                    else_branch = Some(else_stmts);
                }

                let end_tok = self.consume(TokenKind::Iti, "इति")?;
                Ok(Statement::If {
                    condition,
                    then_branch,
                    else_branch,
                    span: cur.span.merge(end_tok.span),
                })
            }

            // यावत् condition ... इति
            TokenKind::Yavat => {
                self.advance();
                let condition = self.parse_expression()?;
                let mut body = Vec::new();
                while !self.check(&TokenKind::Iti) && !self.check(&TokenKind::Eof) {
                    body.push(self.parse_statement()?);
                }
                let end_tok = self.consume(TokenKind::Iti, "इति")?;
                Ok(Statement::While {
                    condition,
                    body,
                    span: cur.span.merge(end_tok.span),
                })
            }

            // प्रति [expr]।
            TokenKind::Prati => {
                self.advance();
                let value = if !self.check(&TokenKind::Danda) {
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                let danda = self.consume(TokenKind::Danda, "। (Danda)")?;
                Ok(Statement::Return {
                    value,
                    span: cur.span.merge(danda.span),
                })
            }

            // Assignment or Expression statement
            TokenKind::Identifier(ref id) => {
                // Lookahead to see if next token is '='
                if self.cursor + 1 < self.tokens.len()
                    && self.tokens[self.cursor + 1].kind == TokenKind::Equal
                {
                    let target = id.clone();
                    self.advance(); // consume identifier
                    self.advance(); // consume '='
                    let value = self.parse_expression()?;
                    let danda = self.consume(TokenKind::Danda, "। (Danda)")?;
                    Ok(Statement::Assignment {
                        target,
                        value,
                        span: cur.span.merge(danda.span),
                    })
                } else {
                    let expr = self.parse_expression()?;
                    self.consume(TokenKind::Danda, "। (Danda)")?;
                    Ok(Statement::Expr(expr))
                }
            }

            _ => {
                let expr = self.parse_expression()?;
                self.consume(TokenKind::Danda, "। (Danda)")?;
                Ok(Statement::Expr(expr))
            }
        }
    }

    pub fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        self.parse_equality()
    }

    fn parse_equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_comparison()?;

        while matches!(
            self.peek_kind(),
            TokenKind::EqualEqual | TokenKind::NotEqual
        ) {
            let op_tok = self.advance().clone();
            let op = match op_tok.kind {
                TokenKind::EqualEqual => BinaryOp::Equal,
                TokenKind::NotEqual => BinaryOp::NotEqual,
                _ => unreachable!(),
            };
            let right = self.parse_comparison()?;
            let span = expr.span.merge(right.span);
            expr = Expr {
                kind: ExprKind::Binary {
                    left: Box::new(expr),
                    op,
                    right: Box::new(right),
                },
                span,
            };
        }

        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_term()?;

        while matches!(
            self.peek_kind(),
            TokenKind::Less | TokenKind::LessEqual | TokenKind::Greater | TokenKind::GreaterEqual
        ) {
            let op_tok = self.advance().clone();
            let op = match op_tok.kind {
                TokenKind::Less => BinaryOp::Less,
                TokenKind::LessEqual => BinaryOp::LessEq,
                TokenKind::Greater => BinaryOp::Greater,
                TokenKind::GreaterEqual => BinaryOp::GreaterEq,
                _ => unreachable!(),
            };
            let right = self.parse_term()?;
            let span = expr.span.merge(right.span);
            expr = Expr {
                kind: ExprKind::Binary {
                    left: Box::new(expr),
                    op,
                    right: Box::new(right),
                },
                span,
            };
        }

        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_factor()?;

        while matches!(self.peek_kind(), TokenKind::Plus | TokenKind::Minus) {
            let op_tok = self.advance().clone();
            let op = match op_tok.kind {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Sub,
                _ => unreachable!(),
            };
            let right = self.parse_factor()?;
            let span = expr.span.merge(right.span);
            expr = Expr {
                kind: ExprKind::Binary {
                    left: Box::new(expr),
                    op,
                    right: Box::new(right),
                },
                span,
            };
        }

        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_unary()?;

        while matches!(
            self.peek_kind(),
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent
        ) {
            let op_tok = self.advance().clone();
            let op = match op_tok.kind {
                TokenKind::Star => BinaryOp::Mul,
                TokenKind::Slash => BinaryOp::Div,
                TokenKind::Percent => BinaryOp::Mod,
                _ => unreachable!(),
            };
            let right = self.parse_unary()?;
            let span = expr.span.merge(right.span);
            expr = Expr {
                kind: ExprKind::Binary {
                    left: Box::new(expr),
                    op,
                    right: Box::new(right),
                },
                span,
            };
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        if matches!(self.peek_kind(), TokenKind::Minus) {
            let op_tok = self.advance().clone();
            let expr = self.parse_unary()?;
            let span = op_tok.span.merge(expr.span);
            return Ok(Expr {
                kind: ExprKind::Unary {
                    op: UnaryOp::Neg,
                    expr: Box::new(expr),
                },
                span,
            });
        }

        if matches!(self.peek_kind(), TokenKind::Rna | TokenKind::CalaRna) {
            let op_tok = self.advance().clone();
            let is_mut = op_tok.kind == TokenKind::CalaRna;
            let expr = self.parse_unary()?;
            let span = op_tok.span.merge(expr.span);
            return Ok(Expr {
                kind: ExprKind::Borrow {
                    is_mut,
                    expr: Box::new(expr),
                },
                span,
            });
        }

        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let cur = self.current().clone();
        match cur.kind {
            TokenKind::DevanagariInteger(val, raw) => {
                self.advance();
                Ok(Expr {
                    kind: ExprKind::DevanagariInteger(val, raw),
                    span: cur.span,
                })
            }
            TokenKind::DevanagariFloat(val, raw) => {
                self.advance();
                Ok(Expr {
                    kind: ExprKind::DevanagariFloat(val, raw),
                    span: cur.span,
                })
            }
            TokenKind::StringLiteral(s) => {
                self.advance();
                Ok(Expr {
                    kind: ExprKind::StringLiteral(s),
                    span: cur.span,
                })
            }
            TokenKind::Satyam => {
                self.advance();
                Ok(Expr {
                    kind: ExprKind::BoolLiteral(true),
                    span: cur.span,
                })
            }
            TokenKind::Mithya => {
                self.advance();
                Ok(Expr {
                    kind: ExprKind::BoolLiteral(false),
                    span: cur.span,
                })
            }
            TokenKind::Identifier(id) => {
                self.advance();
                // Check if it's a function call: id(...)
                if self.check(&TokenKind::LParen) {
                    self.advance(); // consume '('
                    let mut args = Vec::new();
                    if !self.check(&TokenKind::RParen) {
                        loop {
                            args.push(self.parse_expression()?);
                            if self.check(&TokenKind::Comma) {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }
                    let rparen = self.consume(TokenKind::RParen, ")")?;
                    Ok(Expr {
                        kind: ExprKind::Call { callee: id, args },
                        span: cur.span.merge(rparen.span),
                    })
                } else {
                    Ok(Expr {
                        kind: ExprKind::Identifier(id),
                        span: cur.span,
                    })
                }
            }
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expression()?;
                let rparen = self.consume(TokenKind::RParen, ")")?;
                Ok(Expr {
                    kind: expr.kind,
                    span: cur.span.merge(rparen.span),
                })
            }
            _ => Err(ParseError::InvalidExpression(cur.span)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sankode_lexer::Lexer;

    #[test]
    fn test_parse_hello_world() {
        let code = r#"
क्रिया मुख्य() -> रिक्त
    मुद्रय("नमस्ते जगत्!")।
    मान गणना = १०।
    मुद्रय("गणना = ", गणना)।
इति
"#;
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();

        assert_eq!(program.items.len(), 1);
        match &program.items[0] {
            TopLevelItem::Function(f) => {
                assert_eq!(f.name, "मुख्य");
                assert_eq!(f.body.len(), 3);
            }
            _ => panic!("Expected function"),
        }
    }
}
