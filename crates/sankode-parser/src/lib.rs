use sankode_core::{
    BinaryOp, Expr, ExprKind, FieldDecl, FunctionDecl, ImplBlock, Param, Program, Span, Statement,
    StructDecl, Token, TokenKind, TopLevelItem, TypeAnnotation, UnaryOp,
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
                TokenKind::Samracana => {
                    let s = self.parse_struct()?;
                    items.push(TopLevelItem::Struct(s));
                }
                TokenKind::Vidhana => {
                    let imp = self.parse_impl()?;
                    items.push(TopLevelItem::Impl(imp));
                }
                TokenKind::DoubleDanda => {
                    self.advance();
                }
                _ => {
                    let stmt = self.parse_statement()?;
                    items.push(TopLevelItem::Statement(stmt));
                }
            }
        }

        Ok(Program { items })
    }

    fn parse_struct(&mut self) -> Result<StructDecl, ParseError> {
        let sam_tok = self.consume(TokenKind::Samracana, "संरचना")?;
        let name = match self.peek_kind() {
            TokenKind::Identifier(id) => {
                let id = id.clone();
                self.advance();
                id
            }
            _ => {
                let cur = self.current();
                return Err(ParseError::UnexpectedToken {
                    expected: "संरचना-नाम (Struct name)".to_string(),
                    found: cur.kind.clone(),
                    span: cur.span,
                });
            }
        };

        let mut fields = Vec::new();
        while !self.check(&TokenKind::Iti) && !self.check(&TokenKind::Eof) {
            let f_span = self.current().span;
            let f_name = match self.peek_kind() {
                TokenKind::Identifier(id) => {
                    let id = id.clone();
                    self.advance();
                    id
                }
                _ => {
                    let cur = self.current();
                    return Err(ParseError::UnexpectedToken {
                        expected: "क्षेत्र-नाम (Field name)".to_string(),
                        found: cur.kind.clone(),
                        span: cur.span,
                    });
                }
            };
            self.consume(TokenKind::Colon, ":")?;
            let type_ann = self.parse_type_annotation()?;
            if self.check(&TokenKind::Danda) {
                self.advance();
            }
            fields.push(FieldDecl {
                name: f_name,
                type_ann,
                span: f_span,
            });
        }

        let end_tok = self.consume(TokenKind::Iti, "इति")?;
        Ok(StructDecl {
            name,
            fields,
            span: sam_tok.span.merge(end_tok.span),
        })
    }

    fn parse_impl(&mut self) -> Result<ImplBlock, ParseError> {
        let vid_tok = self.consume(TokenKind::Vidhana, "विधान")?;
        let target = match self.peek_kind() {
            TokenKind::Identifier(id) => {
                let id = id.clone();
                self.advance();
                id
            }
            _ => {
                let cur = self.current();
                return Err(ParseError::UnexpectedToken {
                    expected: "विधान-लक्ष्य-नाम (Target struct name)".to_string(),
                    found: cur.kind.clone(),
                    span: cur.span,
                });
            }
        };

        let mut methods = Vec::new();
        while !self.check(&TokenKind::Iti) && !self.check(&TokenKind::Eof) {
            if self.check(&TokenKind::Kriya) {
                let method = self.parse_function()?;
                methods.push(method);
            } else if self.check(&TokenKind::DoubleDanda) {
                self.advance();
            } else {
                let cur = self.current();
                return Err(ParseError::UnexpectedToken {
                    expected: "क्रिया (Method declaration)".to_string(),
                    found: cur.kind.clone(),
                    span: cur.span,
                });
            }
        }

        let end_tok = self.consume(TokenKind::Iti, "इति")?;
        Ok(ImplBlock {
            target,
            methods,
            span: vid_tok.span.merge(end_tok.span),
        })
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

        // Parameter list: (p1: T1, p2: T2) or (स्व, ...) or (ऋण स्व, ...) or (चलऋण स्व, ...)
        self.consume(TokenKind::LParen, "(")?;
        let mut params = Vec::new();
        if !self.check(&TokenKind::RParen) {
            loop {
                let p_span = self.current().span;
                let (p_name, type_ann) = if self.check(&TokenKind::Rna)
                    && self.cursor + 1 < self.tokens.len()
                    && matches!(self.tokens[self.cursor + 1].kind, TokenKind::Identifier(ref id) if id == "स्व")
                {
                    self.advance(); // consume Rna
                    self.advance(); // consume स्व
                    ("स्व".to_string(), TypeAnnotation::Reference(Box::new(TypeAnnotation::Simple("स्व".to_string()))))
                } else if self.check(&TokenKind::CalaRna)
                    && self.cursor + 1 < self.tokens.len()
                    && matches!(self.tokens[self.cursor + 1].kind, TokenKind::Identifier(ref id) if id == "स्व")
                {
                    self.advance(); // consume CalaRna
                    self.advance(); // consume स्व
                    ("स्व".to_string(), TypeAnnotation::MutReference(Box::new(TypeAnnotation::Simple("स्व".to_string()))))
                } else if matches!(self.peek_kind(), TokenKind::Identifier(ref id) if id == "स्व")
                    && (self.cursor + 1 >= self.tokens.len() || self.tokens[self.cursor + 1].kind != TokenKind::Colon)
                {
                    self.advance(); // consume स्व
                    ("स्व".to_string(), TypeAnnotation::Simple("स्व".to_string()))
                } else {
                    let p_name = match self.advance().kind.clone() {
                        TokenKind::Identifier(id) => id,
                        other => {
                            return Err(ParseError::UnexpectedToken {
                                expected: "तर्क-नाम (Parameter name)".to_string(),
                                found: other,
                                span: p_span,
                            });
                        }
                    };

                    self.consume(TokenKind::Colon, ":")?;
                    let type_ann = self.parse_type_annotation()?;
                    (p_name, type_ann)
                };

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
                // Lookahead to see if next token is '.' followed by identifier and '='
                if self.cursor + 3 < self.tokens.len()
                    && self.tokens[self.cursor + 1].kind == TokenKind::Dot
                    && matches!(self.tokens[self.cursor + 2].kind, TokenKind::Identifier(_))
                    && self.tokens[self.cursor + 3].kind == TokenKind::Equal
                {
                    let target = id.clone();
                    self.advance(); // consume target id
                    self.advance(); // consume '.'
                    let field = match self.advance().kind.clone() {
                        TokenKind::Identifier(f) => f,
                        _ => unreachable!(),
                    };
                    self.advance(); // consume '='
                    let value = self.parse_expression()?;
                    let danda = self.consume(TokenKind::Danda, "। (Danda)")?;
                    Ok(Statement::FieldAssignment {
                        target,
                        field,
                        value,
                        span: cur.span.merge(danda.span),
                    })
                } else if self.cursor + 1 < self.tokens.len()
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
        let mut expr = self.parse_base_primary()?;

        while self.check(&TokenKind::Dot) {
            self.advance(); // consume '.'
            let member_span = self.current().span;
            let member_name = match self.advance().kind.clone() {
                TokenKind::Identifier(name) => name,
                other => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "क्षेत्र-नाम वा विधि-नाम (Field or method name)".to_string(),
                        found: other,
                        span: member_span,
                    });
                }
            };

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
                let span = expr.span.merge(rparen.span);
                expr = Expr {
                    kind: ExprKind::MethodCall {
                        target: Box::new(expr),
                        method: member_name,
                        args,
                    },
                    span,
                };
            } else {
                let span = expr.span.merge(member_span);
                expr = Expr {
                    kind: ExprKind::FieldAccess {
                        target: Box::new(expr),
                        field: member_name,
                    },
                    span,
                };
            }
        }

        Ok(expr)
    }

    fn parse_base_primary(&mut self) -> Result<Expr, ParseError> {
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
                // Check if it's a function call or struct instantiation: id(...)
                if self.check(&TokenKind::LParen) {
                    self.advance(); // consume '('
                    // Check if the next token is an identifier followed by ':'
                    // e.g. बिन्दु(क्ष: ३.०, य: ४.०)
                    if self.cursor + 1 < self.tokens.len()
                        && matches!(self.peek_kind(), TokenKind::Identifier(_))
                        && self.tokens[self.cursor + 1].kind == TokenKind::Colon
                    {
                        let mut fields = Vec::new();
                        if !self.check(&TokenKind::RParen) {
                            loop {
                                let f_name = match self.advance().kind.clone() {
                                    TokenKind::Identifier(f) => f,
                                    _ => unreachable!(),
                                };
                                self.consume(TokenKind::Colon, ":")?;
                                let f_val = self.parse_expression()?;
                                fields.push((f_name, f_val));
                                if self.check(&TokenKind::Comma) {
                                    self.advance();
                                } else {
                                    break;
                                }
                            }
                        }
                        let rparen = self.consume(TokenKind::RParen, ")")?;
                        Ok(Expr {
                            kind: ExprKind::StructInit { name: id, fields },
                            span: cur.span.merge(rparen.span),
                        })
                    } else {
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
                    }
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

    #[test]
    fn test_parse_struct_and_methods() {
        let code = r#"
संरचना बिन्दु
    क्ष: अंश६४।
    य: अंश६४।
इति

विधान बिन्दु
    क्रिया दूरता(स्व) -> अंश६४
        प्रति स्व.क्ष + स्व.य।
    इति
इति

क्रिया मुख्य() -> रिक्त
    मान विकार्य ब = बिन्दु(क्ष: ३.०, य: ४.०)।
    मान द = ब.दूरता()।
    ब.क्ष = ५.०।
इति
"#;
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();

        assert_eq!(program.items.len(), 3);
        match &program.items[0] {
            TopLevelItem::Struct(s) => {
                assert_eq!(s.name, "बिन्दु");
                assert_eq!(s.fields.len(), 2);
                assert_eq!(s.fields[0].name, "क्ष");
                assert_eq!(s.fields[1].name, "य");
            }
            _ => panic!("Expected struct"),
        }
        match &program.items[1] {
            TopLevelItem::Impl(imp) => {
                assert_eq!(imp.target, "बिन्दु");
                assert_eq!(imp.methods.len(), 1);
                assert_eq!(imp.methods[0].name, "दूरता");
                assert_eq!(imp.methods[0].params[0].name, "स्व");
            }
            _ => panic!("Expected impl block"),
        }
        match &program.items[2] {
            TopLevelItem::Function(f) => {
                assert_eq!(f.name, "मुख्य");
                assert_eq!(f.body.len(), 3);
            }
            _ => panic!("Expected function"),
        }
    }
}

