#![allow(dead_code)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs::{self, File, OpenOptions}; // <--- 新增 File IO
use std::io::{Read, Write};             // <--- 新增 IO Trait
use std::process;
use std::rc::Rc;

// =========================================================================
// Tokens & Lexer
// =========================================================================

#[derive(Debug, Clone, PartialEq)]
enum TokenKind {
    Eof, Newline, Indent, Dedent, Name(String), Int(i64), Float(f64), String(String),
    Def, Class, If, Elif, Else, While, For, In, Return, Break, Continue, Pass,
    Try, Except, Raise, As, And, Or, Not, NoneVal, TrueVal, FalseVal, Lambda,
    Import, From,
    Lparen, Rparen, Lbracket, Rbracket, Lbrace, Rbrace,
    Comma, Colon, Dot, Plus, Minus, Star, Slash, Percent,
    Equal, PlusEq, MinusEq, Eqeq, Ne, Lt, Le, Gt, Ge,
}

#[derive(Debug, Clone)]
struct Token { kind: TokenKind, line: usize, col: usize }

fn lex_source(source: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new(); let mut indent_stack = vec![0]; let mut line_no = 1;
    for line in source.lines() {
        let mut col = 0; let mut indent = 0; let chars: Vec<char> = line.chars().collect();
        while col < chars.len() && (chars[col] == ' ' || chars[col] == '\t') { indent += if chars[col] == '\t' { 4 } else { 1 }; col += 1; }
        if col == chars.len() || chars[col] == '#' { line_no += 1; continue; }

        let top = *indent_stack.last().unwrap();
        if indent > top { indent_stack.push(indent); tokens.push(Token { kind: TokenKind::Indent, line: line_no, col: 1 }); }
        else {
            while indent < *indent_stack.last().unwrap() { indent_stack.pop(); tokens.push(Token { kind: TokenKind::Dedent, line: line_no, col: 1 }); }
            if indent != *indent_stack.last().unwrap() { return Err(format!("inconsistent indent at line {}", line_no)); }
        }

        let mut i = col;
        while i < chars.len() {
            let c = chars[i]; if c == '#' { break; } if c.is_ascii_whitespace() { i += 1; continue; }
            if c.is_ascii_alphabetic() || c == '_' {
                let start = i; while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') { i += 1; }
                let text: String = chars[start..i].iter().collect();
                let kind = match text.as_str() {
                    "def" => TokenKind::Def, "class" => TokenKind::Class, "if" => TokenKind::If, "elif" => TokenKind::Elif,
                    "else" => TokenKind::Else, "while" => TokenKind::While, "for" => TokenKind::For, "in" => TokenKind::In,
                    "return" => TokenKind::Return, "break" => TokenKind::Break, "continue" => TokenKind::Continue,
                    "pass" => TokenKind::Pass, "try" => TokenKind::Try, "except" => TokenKind::Except, "raise" => TokenKind::Raise,
                    "as" => TokenKind::As, "and" => TokenKind::And, "or" => TokenKind::Or, "not" => TokenKind::Not,
                    "None" => TokenKind::NoneVal, "True" => TokenKind::TrueVal, "False" => TokenKind::FalseVal,
                    "lambda" => TokenKind::Lambda, "import" => TokenKind::Import, "from" => TokenKind::From,
                    _ => TokenKind::Name(text),
                };
                tokens.push(Token { kind, line: line_no, col: start + 1 }); continue;
            }
            if c.is_ascii_digit() {
                let start = i; let mut is_float = false; while i < chars.len() && chars[i].is_ascii_digit() { i += 1; }
                if i < chars.len() && chars[i] == '.' { is_float = true; i += 1; while i < chars.len() && chars[i].is_ascii_digit() { i += 1; } }
                let text: String = chars[start..i].iter().collect();
                let kind = if is_float { TokenKind::Float(text.parse().unwrap()) } else { TokenKind::Int(text.parse().unwrap()) };
                tokens.push(Token { kind, line: line_no, col: start + 1 }); continue;
            }
            if c == '\'' || c == '"' {
                let quote = c; let start = i; let mut val = String::new(); i += 1;
                while i < chars.len() && chars[i] != quote {
                    if chars[i] == '\\' {
                        i += 1; if i == chars.len() { break; }
                        match chars[i] { 'n' => val.push('\n'), 't' => val.push('\t'), '\\' => val.push('\\'), '\'' => val.push('\''), '"' => val.push('"'), _ => val.push(chars[i]) }
                    } else { val.push(chars[i]); }
                    i += 1;
                }
                if i == chars.len() { return Err(format!("unterminated string line {}", line_no)); } i += 1;
                tokens.push(Token { kind: TokenKind::String(val), line: line_no, col: start + 1 }); continue;
            }
            let start = i;
            let (kind, step) = if i + 1 < chars.len() && chars[i] == '=' && chars[i+1] == '=' { (TokenKind::Eqeq, 2) }
            else if i + 1 < chars.len() && chars[i] == '!' && chars[i+1] == '=' { (TokenKind::Ne, 2) } else if i + 1 < chars.len() && chars[i] == '<' && chars[i+1] == '=' { (TokenKind::Le, 2) }
            else if i + 1 < chars.len() && chars[i] == '>' && chars[i+1] == '=' { (TokenKind::Ge, 2) } else if i + 1 < chars.len() && chars[i] == '+' && chars[i+1] == '=' { (TokenKind::PlusEq, 2) }
            else if i + 1 < chars.len() && chars[i] == '-' && chars[i+1] == '=' { (TokenKind::MinusEq, 2) } else {
                let k = match c {
                    '(' => TokenKind::Lparen, ')' => TokenKind::Rparen, '[' => TokenKind::Lbracket, ']' => TokenKind::Rbracket, '{' => TokenKind::Lbrace, '}' => TokenKind::Rbrace,
                    ',' => TokenKind::Comma, ':' => TokenKind::Colon, '.' => TokenKind::Dot, '+' => TokenKind::Plus, '-' => TokenKind::Minus, '*' => TokenKind::Star,
                    '/' => TokenKind::Slash, '%' => TokenKind::Percent, '=' => TokenKind::Equal, '<' => TokenKind::Lt, '>' => TokenKind::Gt,
                    _ => return Err(format!("unexpected '{}' line {}", c, line_no)),
                }; (k, 1)
            };
            tokens.push(Token { kind, line: line_no, col: start + 1 }); i += step;
        }
        tokens.push(Token { kind: TokenKind::Newline, line: line_no, col: chars.len() + 1 }); line_no += 1;
    }
    while indent_stack.len() > 1 { indent_stack.pop(); tokens.push(Token { kind: TokenKind::Dedent, line: line_no, col: 1 }); }
    tokens.push(Token { kind: TokenKind::Eof, line: line_no, col: 1 }); Ok(tokens)
}

// =========================================================================
// AST Nodes
// =========================================================================

#[derive(Debug, Clone, Copy, PartialEq)] enum Op { Add, Sub, Mul, Div, Mod, Eq, Ne, Lt, Le, Gt, Ge, Neg, Not }
#[derive(Debug, Clone, Copy, PartialEq)] enum LogicOp { And, Or }

#[derive(Debug, Clone)]
enum Expr {
    NoneVal, Bool(bool), Int(i64), Float(f64), String(String), Name(String),
    List(Vec<Expr>), Dict(Vec<(Expr, Expr)>), Tuple(Vec<Expr>),
    ListComp(Box<Expr>, String, Box<Expr>, Option<Box<Expr>>), Lambda(Vec<String>, Box<Expr>),
    BinOp(Op, Box<Expr>, Box<Expr>), UnaryOp(Op, Box<Expr>), Compare(Op, Box<Expr>, Box<Expr>), Logical(LogicOp, Box<Expr>, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>), Attribute(Box<Expr>, String), Subscript(Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone)]
enum Stmt {
    Expr(Expr), Assign(String, Expr), AssignIndex(Expr, Expr, Expr), AssignAttr(Expr, String, Expr),
    If(Expr, Vec<Stmt>, Vec<Stmt>), While(Expr, Vec<Stmt>), For(String, Expr, Vec<Stmt>),
    FunctionDef(String, Vec<String>, Vec<Stmt>), ClassDef(String, Vec<Stmt>),
    Try(Vec<Stmt>, Option<String>, Option<String>, Vec<Stmt>), Raise(Expr),
    Import(String), FromImport(String, Vec<String>),
    Return(Option<Expr>), Break, Continue, Pass,
}

// =========================================================================
// Parser
// =========================================================================

struct Parser<'a> { tokens: &'a [Token], pos: usize, filename: &'a str }

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token], filename: &'a str) -> Self { Self { tokens, pos: 0, filename } }
    fn peek(&self) -> &Token { &self.tokens[self.pos] }
    fn prev(&self) -> &Token { &self.tokens[self.pos - 1] }
    fn match_token(&mut self, kind: &TokenKind) -> bool {
        if core::mem::discriminant(&self.peek().kind) == core::mem::discriminant(kind) { self.pos += 1; true } else { false }
    }
    fn expect(&mut self, kind: TokenKind, msg: &str) -> Result<&Token, String> {
        if core::mem::discriminant(&self.peek().kind) != core::mem::discriminant(&kind) {
            Err(format!("{}:{}:{}: {}", self.filename, self.peek().line, self.peek().col, msg))
        } else { self.pos += 1; Ok(self.prev()) }
    }
    fn skip_newlines(&mut self) { while self.match_token(&TokenKind::Newline) {} }

    fn parse_dotted_name(&mut self) -> Result<String, String> {
        let mut name = if let TokenKind::Name(n) = &self.expect(TokenKind::Name("".into()), "expected module name")?.kind { n.clone() } else { unreachable!() };
        while self.match_token(&TokenKind::Dot) {
            if let TokenKind::Name(n) = &self.expect(TokenKind::Name("".into()), "expected module name after dot")?.kind { name.push('.'); name.push_str(n); } else { unreachable!() }
        }
        Ok(name)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        let tok = self.peek().clone();
        match &tok.kind {
            TokenKind::NoneVal => { self.pos += 1; Ok(Expr::NoneVal) }
            TokenKind::TrueVal => { self.pos += 1; Ok(Expr::Bool(true)) } TokenKind::FalseVal => { self.pos += 1; Ok(Expr::Bool(false)) }
            TokenKind::Int(v) => { self.pos += 1; Ok(Expr::Int(*v)) } TokenKind::Float(v) => { self.pos += 1; Ok(Expr::Float(*v)) }
            TokenKind::String(v) => { self.pos += 1; Ok(Expr::String(v.clone())) }
            TokenKind::Name(n) => { self.pos += 1; let mut e = Expr::Name(n.clone()); self.parse_postfix(&mut e)?; Ok(e) }
            TokenKind::Lparen => {
                self.pos += 1;
                if self.match_token(&TokenKind::Rparen) { let mut e = Expr::Tuple(vec![]); self.parse_postfix(&mut e)?; return Ok(e); }
                let first = self.parse_expr()?;
                if self.match_token(&TokenKind::Comma) {
                    let mut items = vec![first];
                    if self.peek().kind != TokenKind::Rparen { loop { items.push(self.parse_expr()?); if !self.match_token(&TokenKind::Comma) || self.peek().kind == TokenKind::Rparen { break; } } }
                    self.expect(TokenKind::Rparen, "expected ')'")?; let mut e = Expr::Tuple(items); self.parse_postfix(&mut e)?; Ok(e)
                } else { self.expect(TokenKind::Rparen, "expected ')'")?; let mut e = first; self.parse_postfix(&mut e)?; Ok(e) }
            }
            TokenKind::Lbracket => {
                self.pos += 1; let mut items = Vec::new();
                if self.match_token(&TokenKind::Rbracket) { let mut e = Expr::List(vec![]); self.parse_postfix(&mut e)?; return Ok(e); }
                let first = self.parse_expr()?;
                if self.match_token(&TokenKind::For) {
                    let v = if let TokenKind::Name(n) = &self.expect(TokenKind::Name("".into()), "expected var")?.kind { n.clone() } else { unreachable!() };
                    self.expect(TokenKind::In, "expected 'in'")?; let iter = self.parse_expr()?;
                    let cond = if self.match_token(&TokenKind::If) { Some(Box::new(self.parse_expr()?)) } else { None };
                    self.expect(TokenKind::Rbracket, "expected ']'")?;
                    let mut e = Expr::ListComp(Box::new(first), v, Box::new(iter), cond); self.parse_postfix(&mut e)?; return Ok(e);
                } else {
                    items.push(first);
                    if self.match_token(&TokenKind::Comma) && self.peek().kind != TokenKind::Rbracket {
                        loop { items.push(self.parse_expr()?); if !self.match_token(&TokenKind::Comma) || self.peek().kind == TokenKind::Rbracket { break; } }
                    }
                    self.expect(TokenKind::Rbracket, "expected ']'")?; let mut e = Expr::List(items); self.parse_postfix(&mut e)?; Ok(e)
                }
            }
            TokenKind::Lbrace => {
                self.pos += 1; let mut pairs = Vec::new();
                if !self.match_token(&TokenKind::Rbrace) {
                    loop { let k = self.parse_expr()?; self.expect(TokenKind::Colon, "expected ':'")?; pairs.push((k, self.parse_expr()?)); if !self.match_token(&TokenKind::Comma) || self.peek().kind == TokenKind::Rbrace { break; } }
                    self.expect(TokenKind::Rbrace, "expected '}'")?;
                }
                let mut e = Expr::Dict(pairs); self.parse_postfix(&mut e)?; Ok(e)
            }
            _ => Err(format!("{}:{}:{}: expected expr", self.filename, tok.line, tok.col)),
        }
    }

    fn parse_postfix(&mut self, expr: &mut Expr) -> Result<(), String> {
        loop {
            if self.match_token(&TokenKind::Lparen) {
                let mut args = Vec::new();
                if !self.match_token(&TokenKind::Rparen) {
                    loop { args.push(self.parse_expr()?); if !self.match_token(&TokenKind::Comma) || self.peek().kind == TokenKind::Rparen { break; } }
                    self.expect(TokenKind::Rparen, "expected ')'")?;
                }
                *expr = Expr::Call(Box::new(expr.clone()), args);
            } else if self.match_token(&TokenKind::Dot) {
                if let TokenKind::Name(n) = &self.expect(TokenKind::Name("".into()), "expected attr")?.kind { *expr = Expr::Attribute(Box::new(expr.clone()), n.clone()); }
            } else if self.match_token(&TokenKind::Lbracket) {
                let idx = self.parse_expr()?; self.expect(TokenKind::Rbracket, "expected ']'")?; *expr = Expr::Subscript(Box::new(expr.clone()), Box::new(idx));
            } else { break; }
        } Ok(())
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        if self.match_token(&TokenKind::Minus) { Ok(Expr::UnaryOp(Op::Neg, Box::new(self.parse_unary()?))) } else { self.parse_primary() }
    }
    fn parse_factor(&mut self) -> Result<Expr, String> {
        let mut e = self.parse_unary()?;
        loop {
            let op = if self.match_token(&TokenKind::Star) { Op::Mul } else if self.match_token(&TokenKind::Slash) { Op::Div } else if self.match_token(&TokenKind::Percent) { Op::Mod } else { break };
            e = Expr::BinOp(op, Box::new(e), Box::new(self.parse_unary()?));
        } Ok(e)
    }
    fn parse_term(&mut self) -> Result<Expr, String> {
        let mut e = self.parse_factor()?;
        loop {
            let op = if self.match_token(&TokenKind::Plus) { Op::Add } else if self.match_token(&TokenKind::Minus) { Op::Sub } else { break };
            e = Expr::BinOp(op, Box::new(e), Box::new(self.parse_factor()?));
        } Ok(e)
    }
    fn parse_comp(&mut self) -> Result<Expr, String> {
        let mut e = self.parse_term()?;
        loop {
            let op = if self.match_token(&TokenKind::Eqeq) { Op::Eq } else if self.match_token(&TokenKind::Ne) { Op::Ne } else if self.match_token(&TokenKind::Lt) { Op::Lt }
            else if self.match_token(&TokenKind::Le) { Op::Le } else if self.match_token(&TokenKind::Gt) { Op::Gt } else if self.match_token(&TokenKind::Ge) { Op::Ge } else { break };
            e = Expr::Compare(op, Box::new(e), Box::new(self.parse_term()?));
        } Ok(e)
    }
    fn parse_not(&mut self) -> Result<Expr, String> { if self.match_token(&TokenKind::Not) { Ok(Expr::UnaryOp(Op::Not, Box::new(self.parse_not()?))) } else { self.parse_comp() } }
    fn parse_and(&mut self) -> Result<Expr, String> { let mut e = self.parse_not()?; while self.match_token(&TokenKind::And) { e = Expr::Logical(LogicOp::And, Box::new(e), Box::new(self.parse_not()?)); } Ok(e) }
    
    fn parse_expr(&mut self) -> Result<Expr, String> { 
        if self.match_token(&TokenKind::Lambda) {
            let mut p = Vec::new();
            if !self.match_token(&TokenKind::Colon) {
                loop { if let TokenKind::Name(pn) = &self.expect(TokenKind::Name("".into()), "expected param")?.kind { p.push(pn.clone()); } if !self.match_token(&TokenKind::Comma) { break; } }
                self.expect(TokenKind::Colon, "expected ':'")?;
            }
            return Ok(Expr::Lambda(p, Box::new(self.parse_expr()?)));
        }
        let mut e = self.parse_and()?; while self.match_token(&TokenKind::Or) { e = Expr::Logical(LogicOp::Or, Box::new(e), Box::new(self.parse_and()?)); } Ok(e) 
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, String> {
        self.expect(TokenKind::Newline, "expected newline")?; self.expect(TokenKind::Indent, "expected indent")?; self.skip_newlines();
        let mut b = Vec::new(); while self.peek().kind != TokenKind::Dedent && self.peek().kind != TokenKind::Eof { b.push(self.parse_stmt()?); self.skip_newlines(); }
        self.expect(TokenKind::Dedent, "expected dedent")?; Ok(b)
    }

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        if self.match_token(&TokenKind::Import) {
            let n = self.parse_dotted_name()?;
            self.expect(TokenKind::Newline, "expected newline")?; return Ok(Stmt::Import(n));
        }
        if self.match_token(&TokenKind::From) {
            let mod_n = self.parse_dotted_name()?;
            self.expect(TokenKind::Import, "expected 'import'")?; let mut names = Vec::new();
            loop {
                if let TokenKind::Name(n) = &self.expect(TokenKind::Name("".into()), "expected name")?.kind { names.push(n.clone()); }
                if !self.match_token(&TokenKind::Comma) { break; }
            }
            self.expect(TokenKind::Newline, "expected newline")?; return Ok(Stmt::FromImport(mod_n, names));
        }

        if self.match_token(&TokenKind::Def) {
            let n = if let TokenKind::Name(n) = &self.expect(TokenKind::Name("".into()), "expected name")?.kind { n.clone() } else { unreachable!() };
            self.expect(TokenKind::Lparen, "expected '('")?; let mut p = Vec::new();
            if !self.match_token(&TokenKind::Rparen) {
                loop { if let TokenKind::Name(pn) = &self.expect(TokenKind::Name("".into()), "expected param")?.kind { p.push(pn.clone()); } if !self.match_token(&TokenKind::Comma) { break; } }
                self.expect(TokenKind::Rparen, "expected ')'")?;
            }
            self.expect(TokenKind::Colon, "expected ':'")?; return Ok(Stmt::FunctionDef(n, p, self.parse_block()?));
        }
        if self.match_token(&TokenKind::Class) {
            let n = if let TokenKind::Name(n) = &self.expect(TokenKind::Name("".into()), "expected class name")?.kind { n.clone() } else { unreachable!() };
            self.expect(TokenKind::Colon, "expected ':'")?; return Ok(Stmt::ClassDef(n, self.parse_block()?));
        }
        if self.match_token(&TokenKind::Try) {
            self.expect(TokenKind::Colon, "expected ':'")?; let body = self.parse_block()?; self.skip_newlines();
            self.expect(TokenKind::Except, "expected 'except'")?;
            let mut exc_type = None; let mut exc_as = None;
            if let TokenKind::Name(n) = &self.peek().kind { exc_type = Some(n.clone()); self.pos += 1; if self.match_token(&TokenKind::As) {
                if let TokenKind::Name(a) = &self.expect(TokenKind::Name("".into()), "expected var")?.kind { exc_as = Some(a.clone()); }
            }}
            self.expect(TokenKind::Colon, "expected ':'")?; return Ok(Stmt::Try(body, exc_type, exc_as, self.parse_block()?));
        }
        if self.match_token(&TokenKind::Raise) { let e = self.parse_expr()?; self.expect(TokenKind::Newline, "expected newline")?; return Ok(Stmt::Raise(e)); }
        if self.match_token(&TokenKind::If) {
            let test = self.parse_expr()?; self.expect(TokenKind::Colon, "expected ':'")?; let body = self.parse_block()?; self.skip_newlines();
            let mut elifs = Vec::new();
            while self.match_token(&TokenKind::Elif) { let t = self.parse_expr()?; self.expect(TokenKind::Colon, "expected ':'")?; elifs.push((t, self.parse_block()?)); self.skip_newlines(); }
            let mut els = if self.match_token(&TokenKind::Else) { self.expect(TokenKind::Colon, "expected ':'")?; self.parse_block()? } else { vec![] };
            for (t, b) in elifs.into_iter().rev() { els = vec![Stmt::If(t, b, els)]; }
            return Ok(Stmt::If(test, body, els));
        }
        if self.match_token(&TokenKind::While) { let test = self.parse_expr()?; self.expect(TokenKind::Colon, "expected ':'")?; return Ok(Stmt::While(test, self.parse_block()?)); }
        if self.match_token(&TokenKind::For) {
            let v = if let TokenKind::Name(n) = &self.expect(TokenKind::Name("".into()), "expected var")?.kind { n.clone() } else { unreachable!() };
            self.expect(TokenKind::In, "expected 'in'")?; let iter = self.parse_expr()?; self.expect(TokenKind::Colon, "expected ':'")?; return Ok(Stmt::For(v, iter, self.parse_block()?));
        }
        if self.match_token(&TokenKind::Return) { if self.match_token(&TokenKind::Newline) { return Ok(Stmt::Return(None)); } let e = self.parse_expr()?; self.expect(TokenKind::Newline, "expected newline")?; return Ok(Stmt::Return(Some(e))); }
        if self.match_token(&TokenKind::Break) { self.expect(TokenKind::Newline, "expected newline")?; return Ok(Stmt::Break); }
        if self.match_token(&TokenKind::Continue) { self.expect(TokenKind::Newline, "expected newline")?; return Ok(Stmt::Continue); }
        if self.match_token(&TokenKind::Pass) { self.expect(TokenKind::Newline, "expected newline")?; return Ok(Stmt::Pass); }

        let expr = self.parse_expr()?;
        if self.match_token(&TokenKind::Equal) || self.match_token(&TokenKind::PlusEq) || self.match_token(&TokenKind::MinusEq) {
            let is_aug = self.prev().kind == TokenKind::PlusEq || self.prev().kind == TokenKind::MinusEq;
            let op = if self.prev().kind == TokenKind::PlusEq { Op::Add } else { Op::Sub };
            let parsed_val = self.parse_expr()?; self.expect(TokenKind::Newline, "expected newline")?;
            let final_val = if is_aug { Expr::BinOp(op, Box::new(expr.clone()), Box::new(parsed_val)) } else { parsed_val };

            return match expr {
                Expr::Name(n) => Ok(Stmt::Assign(n, final_val)),
                Expr::Subscript(o, i) => Ok(Stmt::AssignIndex(*o, *i, final_val)),
                Expr::Attribute(o, a) => Ok(Stmt::AssignAttr(*o, a, final_val)),
                _ => Err("SyntaxError: invalid assign target".into()),
            };
        }
        self.expect(TokenKind::Newline, "expected newline")?; Ok(Stmt::Expr(expr))
    }
    fn parse_module(&mut self) -> Result<Vec<Stmt>, String> { let mut b = Vec::new(); self.skip_newlines(); while self.peek().kind != TokenKind::Eof { b.push(self.parse_stmt()?); self.skip_newlines(); } Ok(b) }
}

// =========================================================================
// Environment & Values
// =========================================================================

#[derive(Clone)]
enum PyValue {
    None, Bool(bool), Int(i64), Float(f64), Str(String),
    Tuple(Vec<PyValue>), List(Rc<RefCell<Vec<PyValue>>>), Dict(Rc<RefCell<HashMap<String, PyValue>>>),
    Function { name: String, params: Vec<String>, body: Rc<Vec<Stmt>>, closure: Rc<RefCell<Env>> },
    Builtin(String, Rc<dyn Fn(&mut Runtime, Vec<PyValue>) -> Result<PyValue, PyValue>>),
    BuiltinMethod(Rc<RefCell<Vec<PyValue>>>, String),
    Class { name: String, methods: Rc<HashMap<String, PyValue>> },
    Instance { class_name: String, class_methods: Rc<HashMap<String, PyValue>>, attrs: Rc<RefCell<HashMap<String, PyValue>>> },
    BoundMethod { receiver: Box<PyValue>, func: Box<PyValue> },
    Exception(String, Box<PyValue>),
    Module(String, Rc<RefCell<Env>>), 
    
    // --- 新增：原生 File 物件以及它的專屬方法 ---
    File(Rc<RefCell<Option<File>>>),
    BuiltinMethodFile(Rc<RefCell<Option<File>>>, String),
}

impl PartialEq for PyValue {
    fn eq(&self, o: &Self) -> bool {
        match (self, o) { (PyValue::None, PyValue::None) => true, (PyValue::Bool(a), PyValue::Bool(b)) => a == b,
            (PyValue::Int(a), PyValue::Int(b)) => a == b, (PyValue::Float(a), PyValue::Float(b)) => a == b, (PyValue::Str(a), PyValue::Str(b)) => a == b, _ => false, }
    }
}

impl fmt::Display for PyValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PyValue::None => write!(f, "None"), PyValue::Bool(b) => write!(f, "{}", if *b { "True" } else { "False" }),
            PyValue::Int(i) => write!(f, "{}", i), PyValue::Float(n) => write!(f, "{}", n), PyValue::Str(s) => write!(f, "{}", s),
            PyValue::Tuple(t) => { let items: Vec<String> = t.iter().map(|v| match v { PyValue::Str(s) => format!("'{}'", s), _ => v.to_string() }).collect(); if items.len() == 1 { write!(f, "({},)", items[0]) } else { write!(f, "({})", items.join(", ")) } }
            PyValue::List(l) => { let items: Vec<String> = l.borrow().iter().map(|v| match v { PyValue::Str(s) => format!("'{}'", s), _ => v.to_string() }).collect(); write!(f, "[{}]", items.join(", ")) }
            PyValue::Dict(d) => { let items: Vec<String> = d.borrow().iter().map(|(k, v)| format!("'{}': {}", k, v)).collect(); write!(f, "{{{}}}", items.join(", ")) }
            PyValue::Class { name, .. } => write!(f, "<class '{}'>", name),
            PyValue::Instance { class_name, .. } => write!(f, "<{} object>", class_name),
            PyValue::BoundMethod { .. } => write!(f, "<bound method>"),
            PyValue::Exception(t, a) => write!(f, "{}({})", t, a),
            PyValue::Builtin(name, _) => write!(f, "<built-in function {}>", name),
            PyValue::BuiltinMethod(_, name) => write!(f, "<built-in method {}>", name),
            PyValue::Function { name, .. } => write!(f, "<function {}>", name),
            PyValue::Module(name, _) => write!(f, "<module '{}'>", name),
            PyValue::File(file) => if file.borrow().is_some() { write!(f, "<open file>") } else { write!(f, "<closed file>") },
            PyValue::BuiltinMethodFile(_, name) => write!(f, "<built-in method {} of file object>", name),
        }
    }
}

fn py_err<T>(typ: &str, msg: &str) -> Result<T, PyValue> { Err(PyValue::Exception(typ.to_string(), Box::new(PyValue::Str(msg.to_string())))) }
fn py_err_val(typ: &str, msg: &str) -> PyValue { PyValue::Exception(typ.to_string(), Box::new(PyValue::Str(msg.to_string()))) }

fn py_to_string(rt: &mut Runtime, val: PyValue) -> Result<String, PyValue> {
    if let PyValue::Instance { class_methods, .. } = &val {
        if let Some(m) = class_methods.get("__str__") {
            let bound = PyValue::BoundMethod { receiver: Box::new(val.clone()), func: Box::new(m.clone()) };
            let res = call_func(rt, bound, vec![])?;
            if let PyValue::Str(s) = res { return Ok(s); }
        }
    }
    Ok(val.to_string())
}

impl PyValue {
    fn is_truthy(&self) -> bool {
        match self { PyValue::None => false, PyValue::Bool(b) => *b, PyValue::Int(i) => *i != 0, PyValue::Float(f) => *f != 0.0, PyValue::Str(s) => !s.is_empty(),
            PyValue::Tuple(t) => !t.is_empty(), PyValue::List(l) => !l.borrow().is_empty(), PyValue::Dict(d) => !d.borrow().is_empty(), _ => true, }
    }
    fn as_num(&self) -> Result<f64, PyValue> { match self { PyValue::Int(i) => Ok(*i as f64), PyValue::Float(f) => Ok(*f), PyValue::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }), _ => py_err("TypeError", "expected number") } }
    fn as_key(&self) -> Result<String, PyValue> { match self { PyValue::Str(s) => Ok(s.clone()), PyValue::Int(i) => Ok(i.to_string()), _ => py_err("TypeError", "unhashable type") } }
}

struct Env { parent: Option<Rc<RefCell<Env>>>, vars: HashMap<String, PyValue> }
impl Env {
    fn new(parent: Option<Rc<RefCell<Env>>>) -> Rc<RefCell<Self>> { Rc::new(RefCell::new(Env { parent, vars: HashMap::new() })) }
    fn set(&mut self, n: &str, v: PyValue) { self.vars.insert(n.to_string(), v); }
    fn assign(&mut self, n: &str, v: PyValue) {
        if self.vars.contains_key(n) { self.vars.insert(n.to_string(), v); return; }
        if let Some(p) = &self.parent { if p.borrow().get_opt(n).is_some() { p.borrow_mut().assign(n, v); return; } }
        self.vars.insert(n.to_string(), v);
    }
    fn get_opt(&self, n: &str) -> Option<PyValue> { if let Some(v) = self.vars.get(n) { Some(v.clone()) } else if let Some(p) = &self.parent { p.borrow().get_opt(n) } else { None } }
    fn get(&self, n: &str) -> Result<PyValue, PyValue> { self.get_opt(n).ok_or_else(|| py_err_val("NameError", &format!("name '{}' is not defined", n))) }
}

// =========================================================================
// Evaluator & Module Loader
// =========================================================================

struct Runtime { sys_modules: HashMap<String, PyValue> }
enum ExecStatus { Continue, Return(PyValue), Break, ContinueLoop }

fn load_module(rt: &mut Runtime, name: &str) -> Result<PyValue, PyValue> {
    if let Some(m) = rt.sys_modules.get(name) { return Ok(m.clone()); }

    let path_base = name.replace('.', "/");
    let file_path = format!("{}.py", path_base);
    let pkg_path = format!("{}/__init__.py", path_base);

    let (path, src) = if let Ok(s) = fs::read_to_string(&file_path) {
        (file_path, s)
    } else if let Ok(s) = fs::read_to_string(&pkg_path) {
        (pkg_path, s)
    } else {
        return py_err("ImportError", &format!("No module named '{}'", name));
    };

    let tokens = lex_source(&src).map_err(|e| py_err_val("SyntaxError", &e))?;
    let mut parser = Parser::new(&tokens, &path);
    let ast = parser.parse_module().map_err(|e| py_err_val("SyntaxError", &e))?;

    let mod_env = Env::new(None);
    install_builtins(&mod_env);

    exec_block(rt, &mod_env, &ast)?;

    let module_val = PyValue::Module(name.to_string(), mod_env);
    rt.sys_modules.insert(name.to_string(), module_val.clone());
    Ok(module_val)
}

fn eval_expr(rt: &mut Runtime, env: &Rc<RefCell<Env>>, expr: &Expr) -> Result<PyValue, PyValue> {
    match expr {
        Expr::NoneVal => Ok(PyValue::None), Expr::Bool(b) => Ok(PyValue::Bool(*b)), Expr::Int(v) => Ok(PyValue::Int(*v)), Expr::Float(v) => Ok(PyValue::Float(*v)),
        Expr::String(v) => Ok(PyValue::Str(v.clone())), Expr::Name(n) => env.borrow().get(n),
        Expr::Tuple(items) => { let mut t = vec![]; for i in items { t.push(eval_expr(rt, env, i)?); } Ok(PyValue::Tuple(t)) }
        Expr::List(items) => { let mut l = vec![]; for i in items { l.push(eval_expr(rt, env, i)?); } Ok(PyValue::List(Rc::new(RefCell::new(l)))) }
        Expr::Dict(pairs) => { let mut d = HashMap::new(); for (k, v) in pairs { d.insert(eval_expr(rt, env, k)?.as_key()?, eval_expr(rt, env, v)?); } Ok(PyValue::Dict(Rc::new(RefCell::new(d)))) }
        Expr::Lambda(params, body_expr) => {
            let body = vec![Stmt::Return(Some(*body_expr.clone()))];
            Ok(PyValue::Function { name: "<lambda>".into(), params: params.clone(), body: Rc::new(body), closure: Rc::clone(env) })
        }
        Expr::ListComp(exp, var, iter, cond) => {
            let it = eval_expr(rt, env, iter)?;
            let items = match it { PyValue::List(l) => l.borrow().clone(), PyValue::Tuple(t) => t, PyValue::Str(s) => s.chars().map(|c| PyValue::Str(c.to_string())).collect(), _ => return py_err("TypeError", "not iterable") };
            let mut res = Vec::new(); let loc = Env::new(Some(Rc::clone(env)));
            for item in items {
                loc.borrow_mut().assign(var, item);
                let ok = if let Some(c) = cond { eval_expr(rt, &loc, c)?.is_truthy() } else { true };
                if ok { res.push(eval_expr(rt, &loc, exp)?); }
            }
            Ok(PyValue::List(Rc::new(RefCell::new(res))))
        }
        Expr::BinOp(op, l, r) => { let left_val = eval_expr(rt, env, l)?; let right_val = eval_expr(rt, env, r)?; apply_binop(rt, env, *op, left_val, right_val) }
        Expr::UnaryOp(op, operand) => { let v = eval_expr(rt, env, operand)?; match op { Op::Neg => match v { PyValue::Int(i) => Ok(PyValue::Int(-i)), _ => Ok(PyValue::Float(-v.as_num()?)) }, Op::Not => Ok(PyValue::Bool(!v.is_truthy())), _ => py_err("TypeError", "bad unary op") } }
        Expr::Compare(op, l, r) => { let left_val = eval_expr(rt, env, l)?; let right_val = eval_expr(rt, env, r)?; apply_comp(rt, env, *op, left_val, right_val) }
        Expr::Logical(op, l, r) => { let lv = eval_expr(rt, env, l)?; match op { LogicOp::And => if !lv.is_truthy() { Ok(lv) } else { eval_expr(rt, env, r) }, LogicOp::Or => if lv.is_truthy() { Ok(lv) } else { eval_expr(rt, env, r) } } }
        Expr::Call(func, args) => { let f = eval_expr(rt, env, func)?; let mut a = vec![]; for expr_a in args { a.push(eval_expr(rt, env, expr_a)?); } call_func(rt, f, a) }
        Expr::Attribute(obj, attr) => {
            let o = eval_expr(rt, env, obj)?;
            match &o {
                PyValue::Module(_, mod_env) => { mod_env.borrow().get(attr) }
                PyValue::Instance { class_methods, attrs, .. } => {
                    if let Some(v) = attrs.borrow().get(attr) { return Ok(v.clone()); }
                    if let Some(m) = class_methods.get(attr) { return Ok(PyValue::BoundMethod { receiver: Box::new(o.clone()), func: Box::new(m.clone()) }); }
                    py_err("AttributeError", &format!("object has no attribute '{}'", attr))
                }
                PyValue::List(l) if attr == "append" => Ok(PyValue::BuiltinMethod(Rc::clone(l), "append".into())),
                PyValue::File(f) => { // <--- 解析 File 物件的方法呼叫
                    match attr.as_str() {
                        "read" | "write" | "close" => Ok(PyValue::BuiltinMethodFile(Rc::clone(f), attr.clone())),
                        _ => py_err("AttributeError", &format!("file object has no attribute '{}'", attr))
                    }
                }
                _ => py_err("AttributeError", "object has no attribute")
            }
        }
        Expr::Subscript(obj, idx) => {
            let o = eval_expr(rt, env, obj)?; let i = eval_expr(rt, env, idx)?;
            match &o {
                PyValue::Tuple(t) => { let idx = match i { PyValue::Int(i) => i, _ => return py_err("TypeError", "index must be int") }; if idx < 0 || idx as usize >= t.len() { py_err("IndexError", "tuple index out of range") } else { Ok(t[idx as usize].clone()) } }
                PyValue::List(l) => { let idx = match i { PyValue::Int(i) => i, _ => return py_err("TypeError", "index must be int") }; let b = l.borrow(); if idx < 0 || idx as usize >= b.len() { py_err("IndexError", "list index out of range") } else { Ok(b[idx as usize].clone()) } }
                PyValue::Dict(d) => d.borrow().get(&i.as_key()?).cloned().ok_or_else(|| PyValue::Exception("KeyError".into(), Box::new(i.clone()))),
                _ => py_err("TypeError", "object is not subscriptable"),
            }
        }
    }
}

fn apply_binop(rt: &mut Runtime, _env: &Rc<RefCell<Env>>, op: Op, l: PyValue, r: PyValue) -> Result<PyValue, PyValue> {
    if op == Op::Add {
        if let PyValue::Instance { class_methods, .. } = &l {
            if let Some(m) = class_methods.get("__add__") { let bound = PyValue::BoundMethod { receiver: Box::new(l.clone()), func: Box::new(m.clone()) }; return call_func(rt, bound, vec![r]); }
        }
        if let (PyValue::Str(a), PyValue::Str(b)) = (&l, &r) { return Ok(PyValue::Str(format!("{}{}", a, b))); }
    }
    if let (PyValue::Int(a), PyValue::Int(b)) = (&l, &r) {
        return match op {
            Op::Add => Ok(PyValue::Int(a+b)), Op::Sub => Ok(PyValue::Int(a-b)), Op::Mul => Ok(PyValue::Int(a*b)),
            Op::Div => { if *b == 0 { return py_err("ZeroDivisionError", "division by zero"); } Ok(PyValue::Float((*a as f64)/(*b as f64))) },
            Op::Mod => { if *b == 0 { return py_err("ZeroDivisionError", "modulo by zero"); } Ok(PyValue::Int(a%b)) },
            _ => py_err("TypeError", "unsupported operand type(s)")
        };
    }
    let a = l.as_num()?; let b = r.as_num()?;
    match op {
        Op::Add => Ok(PyValue::Float(a+b)), Op::Sub => Ok(PyValue::Float(a-b)), Op::Mul => Ok(PyValue::Float(a*b)),
        Op::Div => { if b == 0.0 { return py_err("ZeroDivisionError", "division by zero"); } Ok(PyValue::Float(a/b)) },
        Op::Mod => Ok(PyValue::Float((a as i64 % b as i64) as f64)),
        _ => py_err("TypeError", "unsupported operand type(s)")
    }
}

fn apply_comp(_rt: &mut Runtime, _env: &Rc<RefCell<Env>>, op: Op, l: PyValue, r: PyValue) -> Result<PyValue, PyValue> {
    if let (PyValue::Str(a), PyValue::Str(b)) = (&l, &r) { return Ok(PyValue::Bool(match op { Op::Eq => a == b, Op::Ne => a != b, _ => false })); }
    let a = l.as_num()?; let b = r.as_num()?; Ok(PyValue::Bool(match op { Op::Eq => a == b, Op::Ne => a != b, Op::Lt => a < b, Op::Le => a <= b, Op::Gt => a > b, Op::Ge => a >= b, _ => false }))
}

fn exec_stmt(rt: &mut Runtime, env: &Rc<RefCell<Env>>, stmt: &Stmt) -> Result<ExecStatus, PyValue> {
    match stmt {
        Stmt::Expr(e) => { eval_expr(rt, env, e)?; Ok(ExecStatus::Continue) }
        Stmt::Assign(n, e) => { let v = eval_expr(rt, env, e)?; env.borrow_mut().assign(n, v); Ok(ExecStatus::Continue) }
        Stmt::AssignIndex(t, i, v) => {
            let obj = eval_expr(rt, env, t)?; let idx = eval_expr(rt, env, i)?; let val = eval_expr(rt, env, v)?;
            match &obj {
                PyValue::List(l) => { let i = match idx { PyValue::Int(i) => i, _ => return py_err("TypeError", "list indices must be integers") }; let mut b = l.borrow_mut(); if i < 0 || i as usize >= b.len() { return py_err("IndexError", "list assignment index out of range"); } b[i as usize] = val; }
                PyValue::Dict(d) => { d.borrow_mut().insert(idx.as_key()?, val); }
                PyValue::Tuple(_) => return py_err("TypeError", "tuple object does not support item assignment"),
                _ => return py_err("TypeError", "object does not support item assignment"),
            } Ok(ExecStatus::Continue)
        }
        Stmt::AssignAttr(obj, attr, val) => {
            let o = eval_expr(rt, env, obj)?; let v = eval_expr(rt, env, val)?;
            if let PyValue::Instance { attrs, .. } = &o { attrs.borrow_mut().insert(attr.clone(), v); Ok(ExecStatus::Continue) } else { py_err("AttributeError", "cannot assign attribute") }
        }
        Stmt::If(t, b, e) => { if eval_expr(rt, env, t)?.is_truthy() { exec_block(rt, env, b) } else { exec_block(rt, env, e) } }
        Stmt::While(t, b) => { while eval_expr(rt, env, t)?.is_truthy() { match exec_block(rt, env, b)? { ExecStatus::Return(v) => return Ok(ExecStatus::Return(v)), ExecStatus::Break => break, _ => {} } } Ok(ExecStatus::Continue) }
        Stmt::For(v, iter, b) => {
            let it = eval_expr(rt, env, iter)?;
            let items = match it { PyValue::List(l) => l.borrow().clone(), PyValue::Tuple(t) => t, PyValue::Str(s) => s.chars().map(|c| PyValue::Str(c.to_string())).collect(), _ => return py_err("TypeError", "object is not iterable") };
            for item in items { env.borrow_mut().assign(v, item); match exec_block(rt, env, b)? { ExecStatus::Return(ret) => return Ok(ExecStatus::Return(ret)), ExecStatus::Break => break, _ => {} } } Ok(ExecStatus::Continue)
        }
        Stmt::FunctionDef(n, p, b) => { env.borrow_mut().set(n, PyValue::Function { name: n.clone(), params: p.clone(), body: Rc::new(b.clone()), closure: Rc::clone(env) }); Ok(ExecStatus::Continue) }
        Stmt::ClassDef(n, b) => {
            let class_env = Env::new(Some(Rc::clone(env))); exec_block(rt, &class_env, b)?;
            let methods = class_env.borrow().vars.clone();
            env.borrow_mut().set(n, PyValue::Class { name: n.clone(), methods: Rc::new(methods) }); Ok(ExecStatus::Continue)
        }
Stmt::Try(body, _exc_type, exc_as, except_body) => {
            match exec_block(rt, env, body) {
                Err(exc) => {
                    let should_catch = match _exc_type {
                        Some(ref t) => { 
                            if let PyValue::Exception(exc_t, _) = &exc { 
                                // 修改處：如果腳本指定了 Exception，我們把它當成萬用捕捉
                                // 否則，必須精確匹配錯誤名稱 (例如 except ValueError:)
                                t == "Exception" || exc_t == t 
                            } else { 
                                false 
                            } 
                        },
                        None => true
                    };
                    
                    if should_catch {
                        let except_env = Env::new(Some(Rc::clone(env))); 
                        if let Some(var) = exc_as { except_env.borrow_mut().set(&var, exc); }
                        exec_block(rt, &except_env, except_body)
                    } else { 
                        Err(exc) 
                    }
                }
                Ok(status) => Ok(status)
            }
        }
        Stmt::Raise(e) => { Err(eval_expr(rt, env, e)?) }
        Stmt::Import(mod_name) => {
            let module = load_module(rt, mod_name)?;
            let bind_name = mod_name.split('.').last().unwrap();
            env.borrow_mut().assign(bind_name, module);
            Ok(ExecStatus::Continue)
        }
        Stmt::FromImport(mod_name, names) => {
            let module = load_module(rt, mod_name)?;
            if let PyValue::Module(_, mod_env) = module {
                for n in names {
                    let val = mod_env.borrow().get(n)?;
                    env.borrow_mut().assign(n, val);
                }
            }
            Ok(ExecStatus::Continue)
        }
        Stmt::Return(e) => Ok(ExecStatus::Return(if let Some(x) = e { eval_expr(rt, env, x)? } else { PyValue::None })),
        Stmt::Break => Ok(ExecStatus::Break), Stmt::Continue => Ok(ExecStatus::ContinueLoop), Stmt::Pass => Ok(ExecStatus::Continue),
    }
}

fn exec_block(rt: &mut Runtime, env: &Rc<RefCell<Env>>, stmts: &[Stmt]) -> Result<ExecStatus, PyValue> {
    for s in stmts { let st = exec_stmt(rt, env, s)?; if !matches!(st, ExecStatus::Continue) { return Ok(st); } } Ok(ExecStatus::Continue)
}

fn call_func(rt: &mut Runtime, func: PyValue, args: Vec<PyValue>) -> Result<PyValue, PyValue> {
    match func {
        PyValue::Builtin(_, f) => f(rt, args),
        PyValue::BuiltinMethod(obj, n) => { if n == "append" { obj.borrow_mut().push(args[0].clone()); Ok(PyValue::None) } else { py_err("AttributeError", "unknown method") } }
        
        // --- 執行 File 物件的 read / write / close 方法 ---
        PyValue::BuiltinMethodFile(f, method_name) => {
            let mut file_opt = f.borrow_mut();
            match method_name.as_str() {
                "read" => {
                    if let Some(file) = file_opt.as_mut() {
                        let mut s = String::new();
                        file.read_to_string(&mut s).map_err(|e| py_err_val("IOError", &e.to_string()))?;
                        Ok(PyValue::Str(s))
                    } else { py_err("ValueError", "I/O operation on closed file.") }
                }
                "write" => {
                    if let Some(file) = file_opt.as_mut() {
                        if args.is_empty() { return py_err("TypeError", "write() takes exactly one argument"); }
                        let s = py_to_string(rt, args[0].clone())?;
                        file.write_all(s.as_bytes()).map_err(|e| py_err_val("IOError", &e.to_string()))?;
                        Ok(PyValue::Int(s.len() as i64))
                    } else { py_err("ValueError", "I/O operation on closed file.") }
                }
                "close" => {
                    *file_opt = None; // 將 Option 設為 None 來釋放 File handle 並關閉檔案
                    Ok(PyValue::None)
                }
                _ => py_err("AttributeError", "unknown method")
            }
        }

        PyValue::Class { name, methods } => {
            let inst = PyValue::Instance { class_name: name, class_methods: Rc::clone(&methods), attrs: Rc::new(RefCell::new(HashMap::new())) };
            if let Some(init) = methods.get("__init__") { let mut a = vec![inst.clone()]; a.extend(args); call_func(rt, init.clone(), a)?; }
            Ok(inst)
        }
        PyValue::BoundMethod { receiver, func } => { let mut a = vec![*receiver]; a.extend(args); call_func(rt, *func, a) }
        PyValue::Function { name, params, body, closure } => {
            if args.len() != params.len() { return py_err("TypeError", &format!("{}() takes {} positional arguments but {} were given", name, params.len(), args.len())); }
            let local = Env::new(Some(closure)); for (p, a) in params.iter().zip(args) { local.borrow_mut().set(p, a); }
            match exec_block(rt, &local, &body)? { ExecStatus::Return(v) => Ok(v), _ => Ok(PyValue::None) }
        }
        _ => py_err("TypeError", "object is not callable"),
    }
}

// =========================================================================
// Builtins & Main
// =========================================================================

fn install_builtins(globals: &Rc<RefCell<Env>>) {
    let mut e = globals.borrow_mut();
    e.set("print", PyValue::Builtin("print".into(), Rc::new(|rt, a| { 
        let mut out = Vec::new();
        for val in a { out.push(py_to_string(rt, val.clone())?); }
        println!("{}", out.join(" ")); Ok(PyValue::None) 
    })));
    e.set("str", PyValue::Builtin("str".into(), Rc::new(|rt, a| {
        if a.len() != 1 { return py_err("TypeError", "str() takes exactly one argument"); }
        Ok(PyValue::Str(py_to_string(rt, a[0].clone())?))
    })));
    e.set("len", PyValue::Builtin("len".into(), Rc::new(|_, a| {
        if a.is_empty() { return py_err("TypeError", "len() takes exactly one argument (0 given)"); }
        match &a[0] { PyValue::Str(s) => Ok(PyValue::Int(s.len() as i64)), PyValue::List(l) => Ok(PyValue::Int(l.borrow().len() as i64)),
            PyValue::Tuple(t) => Ok(PyValue::Int(t.len() as i64)), PyValue::Dict(d) => Ok(PyValue::Int(d.borrow().len() as i64)), _ => py_err("TypeError", "object has no len()") }
    })));
    e.set("range", PyValue::Builtin("range".into(), Rc::new(|_, a| {
        if a.is_empty() { return py_err("TypeError", "range expected 1 argument, got 0"); }
        let end = match a[0] { PyValue::Int(i) => i, _ => return py_err("TypeError", "range() integer argument expected") };
        Ok(PyValue::List(Rc::new(RefCell::new((0..end).map(PyValue::Int).collect()))))
    })));
    e.set("Exception", PyValue::Builtin("Exception".into(), Rc::new(|_, a| {
        let arg = a.get(0).cloned().unwrap_or(PyValue::None);
        Ok(PyValue::Exception("Exception".into(), Box::new(arg)))
    })));

    // --- 新增：open() 內建函數 ---
    e.set("open", PyValue::Builtin("open".into(), Rc::new(|_, a| {
        if a.is_empty() { return py_err("TypeError", "open() expected at least 1 argument"); }
        let path = if let PyValue::Str(s) = &a[0] { s } else { return py_err("TypeError", "expected string as path"); };
        
        let mode = if a.len() > 1 {
            if let PyValue::Str(s) = &a[1] { s.clone() } else { return py_err("TypeError", "expected string as mode"); }
        } else {
            "r".to_string()
        };

        let mut opts = OpenOptions::new();
        match mode.as_str() {
            "r" => opts.read(true),
            "w" => opts.write(true).create(true).truncate(true),
            "a" => opts.write(true).create(true).append(true),
            _ => return py_err("ValueError", "invalid mode"),
        };

        let file = opts.open(path).map_err(|err| py_err_val("IOError", &err.to_string()))?;
        Ok(PyValue::File(Rc::new(RefCell::new(Some(file)))))
    })));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 { eprintln!("Usage: ./py5 <script.py>"); process::exit(1); }
    let src = fs::read_to_string(&args[1]).unwrap_or_else(|_| { eprintln!("cannot open {}", args[1]); process::exit(1); });

    let globals = Env::new(None); install_builtins(&globals);
    let mut rt = Runtime { sys_modules: HashMap::new() };

    let tokens = lex_source(&src).unwrap_or_else(|e| { eprintln!("SyntaxError: {}", e); process::exit(1); });
    let mut parser = Parser::new(&tokens, &args[1]);
    let module = parser.parse_module().unwrap_or_else(|e| { eprintln!("SyntaxError: {}", e); process::exit(1); });

    if let Err(exc) = exec_block(&mut rt, &globals, &module) { 
        eprintln!("Traceback (most recent call last):\n  {}", py_to_string(&mut rt, exc).unwrap_or_default()); 
        process::exit(1); 
    }
}