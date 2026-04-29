use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs;
use std::process;
use std::rc::Rc;

// =========================================================================
// Tokens & Lexer
// =========================================================================

#[derive(Debug, Clone, PartialEq)]
enum TokenKind {
    Eof, Newline, Indent, Dedent, Name(String), Int(i64), Float(f64), String(String),
    Def, If, Elif, Else, While, For, In, Return, Break, Continue, Pass,
    And, Or, Not, NoneVal, TrueVal, FalseVal,
    Lparen, Rparen, Lbracket, Rbracket, Lbrace, Rbrace,
    Comma, Colon, Dot, Plus, Minus, Star, Slash, Percent,
    Equal, PlusEq, MinusEq, Eqeq, Ne, Lt, Le, Gt, Ge,
}

#[derive(Debug, Clone)]
struct Token {
    kind: TokenKind,
    line: usize,
    col: usize,
}

fn lex_source(source: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut indent_stack = vec![0];
    let mut line_no = 1;

    for line in source.lines() {
        let mut col = 0;
        let mut indent = 0;
        let chars: Vec<char> = line.chars().collect();

        while col < chars.len() && (chars[col] == ' ' || chars[col] == '\t') {
            indent += if chars[col] == '\t' { 4 } else { 1 };
            col += 1;
        }

        let is_blank = col == chars.len() || chars[col] == '#';
        if is_blank { line_no += 1; continue; }

        let top = *indent_stack.last().unwrap();
        if indent > top {
            indent_stack.push(indent);
            tokens.push(Token { kind: TokenKind::Indent, line: line_no, col: 1 });
        } else {
            while indent < *indent_stack.last().unwrap() {
                indent_stack.pop();
                tokens.push(Token { kind: TokenKind::Dedent, line: line_no, col: 1 });
            }
            if indent != *indent_stack.last().unwrap() {
                return Err(format!("inconsistent indentation at line {}", line_no));
            }
        }

        let mut i = col;
        while i < chars.len() {
            let c = chars[i];
            if c == '#' { break; }
            if c.is_ascii_whitespace() { i += 1; continue; }

            if c.is_ascii_alphabetic() || c == '_' {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') { i += 1; }
                let text: String = chars[start..i].iter().collect();
                let kind = match text.as_str() {
                    "def" => TokenKind::Def, "if" => TokenKind::If, "elif" => TokenKind::Elif,
                    "else" => TokenKind::Else, "while" => TokenKind::While, "for" => TokenKind::For,
                    "in" => TokenKind::In, "return" => TokenKind::Return, "break" => TokenKind::Break,
                    "continue" => TokenKind::Continue, "pass" => TokenKind::Pass,
                    "and" => TokenKind::And, "or" => TokenKind::Or, "not" => TokenKind::Not,
                    "None" => TokenKind::NoneVal, "True" => TokenKind::TrueVal, "False" => TokenKind::FalseVal,
                    _ => TokenKind::Name(text),
                };
                tokens.push(Token { kind, line: line_no, col: start + 1 });
                continue;
            }

            if c.is_ascii_digit() {
                let start = i;
                let mut is_float = false;
                while i < chars.len() && chars[i].is_ascii_digit() { i += 1; }
                if i < chars.len() && chars[i] == '.' {
                    is_float = true; i += 1;
                    while i < chars.len() && chars[i].is_ascii_digit() { i += 1; }
                }
                let text: String = chars[start..i].iter().collect();
                let kind = if is_float { TokenKind::Float(text.parse().unwrap()) } else { TokenKind::Int(text.parse().unwrap()) };
                tokens.push(Token { kind, line: line_no, col: start + 1 });
                continue;
            }

            if c == '\'' || c == '"' {
                let quote = c;
                let start = i;
                let mut val = String::new();
                i += 1;
                while i < chars.len() && chars[i] != quote {
                    if chars[i] == '\\' {
                        i += 1;
                        if i == chars.len() { break; }
                        match chars[i] {
                            'n' => val.push('\n'), 't' => val.push('\t'),
                            '\\' => val.push('\\'), '\'' => val.push('\''),
                            '"' => val.push('"'), _ => val.push(chars[i]),
                        }
                    } else { val.push(chars[i]); }
                    i += 1;
                }
                if i == chars.len() { return Err(format!("unterminated string at line {}", line_no)); }
                i += 1;
                tokens.push(Token { kind: TokenKind::String(val), line: line_no, col: start + 1 });
                continue;
            }

            let start = i;
            let (kind, step) = if i + 1 < chars.len() && chars[i] == '=' && chars[i+1] == '=' { (TokenKind::Eqeq, 2) }
            else if i + 1 < chars.len() && chars[i] == '!' && chars[i+1] == '=' { (TokenKind::Ne, 2) }
            else if i + 1 < chars.len() && chars[i] == '<' && chars[i+1] == '=' { (TokenKind::Le, 2) }
            else if i + 1 < chars.len() && chars[i] == '>' && chars[i+1] == '=' { (TokenKind::Ge, 2) }
            else if i + 1 < chars.len() && chars[i] == '+' && chars[i+1] == '=' { (TokenKind::PlusEq, 2) }
            else if i + 1 < chars.len() && chars[i] == '-' && chars[i+1] == '=' { (TokenKind::MinusEq, 2) }
            else {
                let k = match c {
                    '(' => TokenKind::Lparen, ')' => TokenKind::Rparen,
                    '[' => TokenKind::Lbracket, ']' => TokenKind::Rbracket,
                    '{' => TokenKind::Lbrace, '}' => TokenKind::Rbrace,
                    ',' => TokenKind::Comma, ':' => TokenKind::Colon, '.' => TokenKind::Dot,
                    '+' => TokenKind::Plus, '-' => TokenKind::Minus, '*' => TokenKind::Star,
                    '/' => TokenKind::Slash, '%' => TokenKind::Percent, '=' => TokenKind::Equal,
                    '<' => TokenKind::Lt, '>' => TokenKind::Gt,
                    _ => return Err(format!("unexpected character '{}' at line {}", c, line_no)),
                };
                (k, 1)
            };
            tokens.push(Token { kind, line: line_no, col: start + 1 });
            i += step;
        }
        tokens.push(Token { kind: TokenKind::Newline, line: line_no, col: chars.len() + 1 });
        line_no += 1;
    }
    while indent_stack.len() > 1 { indent_stack.pop(); tokens.push(Token { kind: TokenKind::Dedent, line: line_no, col: 1 }); }
    tokens.push(Token { kind: TokenKind::Eof, line: line_no, col: 1 });
    Ok(tokens)
}

// =========================================================================
// AST Nodes
// =========================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
enum Op { Add, Sub, Mul, Div, Mod, Eq, Ne, Lt, Le, Gt, Ge, Neg, Not }

#[derive(Debug, Clone, Copy, PartialEq)]
enum LogicOp { And, Or }

#[derive(Debug, Clone)]
enum Expr {
    NoneVal, Bool(bool), Int(i64), Float(f64), String(String), Name(String),
    List(Vec<Expr>), Dict(Vec<(Expr, Expr)>),
    BinOp(Op, Box<Expr>, Box<Expr>),
    UnaryOp(Op, Box<Expr>),
    Compare(Op, Box<Expr>, Box<Expr>),
    Logical(LogicOp, Box<Expr>, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Attribute(Box<Expr>, String),
    Subscript(Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone)]
enum Stmt {
    Expr(Expr),
    Assign(String, Expr),
    AssignIndex(Expr, Expr, Expr), // target[idx] = val
    AssignAttr(Expr, String, Expr), // target.attr = val
    AugAssign(String, Op, Expr),
    If(Expr, Vec<Stmt>, Vec<Stmt>),
    While(Expr, Vec<Stmt>),
    For(String, Expr, Vec<Stmt>),
    FunctionDef(String, Vec<String>, Vec<Stmt>),
    Return(Option<Expr>),
    Break,
    Continue,
    Pass,
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
        if core::mem::discriminant(&self.peek().kind) == core::mem::discriminant(kind) {
            self.pos += 1; true
        } else { false }
    }

    fn expect(&mut self, kind: TokenKind, msg: &str) -> Result<&Token, String> {
        if core::mem::discriminant(&self.peek().kind) != core::mem::discriminant(&kind) {
            let tok = self.peek();
            Err(format!("{}:{}:{}: {}", self.filename, tok.line, tok.col, msg))
        } else { self.pos += 1; Ok(self.prev()) }
    }

    fn skip_newlines(&mut self) { while self.match_token(&TokenKind::Newline) {} }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        let tok = self.peek().clone();
        match &tok.kind {
            TokenKind::NoneVal => { self.pos += 1; Ok(Expr::NoneVal) }
            TokenKind::TrueVal => { self.pos += 1; Ok(Expr::Bool(true)) }
            TokenKind::FalseVal => { self.pos += 1; Ok(Expr::Bool(false)) }
            TokenKind::Int(v) => { self.pos += 1; Ok(Expr::Int(*v)) }
            TokenKind::Float(v) => { self.pos += 1; Ok(Expr::Float(*v)) }
            TokenKind::String(v) => { self.pos += 1; Ok(Expr::String(v.clone())) }
            TokenKind::Name(n) => { self.pos += 1; let mut e = Expr::Name(n.clone()); self.parse_postfix(&mut e)?; Ok(e) }
            TokenKind::Lparen => {
                self.pos += 1; let mut e = self.parse_expr()?; self.expect(TokenKind::Rparen, "expected ')'")?;
                self.parse_postfix(&mut e)?; Ok(e)
            }
            TokenKind::Lbracket => {
                self.pos += 1; let mut items = Vec::new();
                if !self.match_token(&TokenKind::Rbracket) {
                    loop {
                        items.push(self.parse_expr()?);
                        if !self.match_token(&TokenKind::Comma) { break; }
                        if self.peek().kind == TokenKind::Rbracket { break; } // trailing comma
                    }
                    self.expect(TokenKind::Rbracket, "expected ']'")?;
                }
                let mut e = Expr::List(items); self.parse_postfix(&mut e)?; Ok(e)
            }
            TokenKind::Lbrace => {
                self.pos += 1; let mut pairs = Vec::new();
                if !self.match_token(&TokenKind::Rbrace) {
                    loop {
                        let k = self.parse_expr()?; self.expect(TokenKind::Colon, "expected ':' in dict")?;
                        let v = self.parse_expr()?; pairs.push((k, v));
                        if !self.match_token(&TokenKind::Comma) { break; }
                        if self.peek().kind == TokenKind::Rbrace { break; }
                    }
                    self.expect(TokenKind::Rbrace, "expected '}'")?;
                }
                let mut e = Expr::Dict(pairs); self.parse_postfix(&mut e)?; Ok(e)
            }
            _ => Err(format!("{}:{}:{}: expected expression", self.filename, tok.line, tok.col)),
        }
    }

    fn parse_postfix(&mut self, expr: &mut Expr) -> Result<(), String> {
        loop {
            if self.match_token(&TokenKind::Lparen) {
                let mut args = Vec::new();
                if !self.match_token(&TokenKind::Rparen) {
                    loop {
                        args.push(self.parse_expr()?);
                        if !self.match_token(&TokenKind::Comma) { break; }
                        if self.peek().kind == TokenKind::Rparen { break; }
                    }
                    self.expect(TokenKind::Rparen, "expected ')'")?;
                }
                *expr = Expr::Call(Box::new(expr.clone()), args);
            } else if self.match_token(&TokenKind::Dot) {
                if let TokenKind::Name(n) = &self.expect(TokenKind::Name(String::new()), "expected attribute")?.kind {
                    *expr = Expr::Attribute(Box::new(expr.clone()), n.clone());
                }
            } else if self.match_token(&TokenKind::Lbracket) {
                let index = self.parse_expr()?; self.expect(TokenKind::Rbracket, "expected ']'")?;
                *expr = Expr::Subscript(Box::new(expr.clone()), Box::new(index));
            } else { break; }
        }
        Ok(())
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        if self.match_token(&TokenKind::Minus) { Ok(Expr::UnaryOp(Op::Neg, Box::new(self.parse_unary()?))) }
        else { self.parse_primary() }
    }

    fn parse_factor(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_unary()?;
        loop {
            let op = if self.match_token(&TokenKind::Star) { Op::Mul } else if self.match_token(&TokenKind::Slash) { Op::Div }
            else if self.match_token(&TokenKind::Percent) { Op::Mod } else { break };
            expr = Expr::BinOp(op, Box::new(expr), Box::new(self.parse_unary()?));
        }
        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_factor()?;
        loop {
            let op = if self.match_token(&TokenKind::Plus) { Op::Add } else if self.match_token(&TokenKind::Minus) { Op::Sub } else { break };
            expr = Expr::BinOp(op, Box::new(expr), Box::new(self.parse_factor()?));
        }
        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_term()?;
        loop {
            let op = if self.match_token(&TokenKind::Eqeq) { Op::Eq } else if self.match_token(&TokenKind::Ne) { Op::Ne }
            else if self.match_token(&TokenKind::Lt) { Op::Lt } else if self.match_token(&TokenKind::Le) { Op::Le }
            else if self.match_token(&TokenKind::Gt) { Op::Gt } else if self.match_token(&TokenKind::Ge) { Op::Ge }
            else { break };
            expr = Expr::Compare(op, Box::new(expr), Box::new(self.parse_term()?));
        }
        Ok(expr)
    }

    fn parse_not(&mut self) -> Result<Expr, String> {
        if self.match_token(&TokenKind::Not) { Ok(Expr::UnaryOp(Op::Not, Box::new(self.parse_not()?))) }
        else { self.parse_comparison() }
    }

    fn parse_and(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_not()?;
        while self.match_token(&TokenKind::And) { expr = Expr::Logical(LogicOp::And, Box::new(expr), Box::new(self.parse_not()?)); }
        Ok(expr)
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_and()?;
        while self.match_token(&TokenKind::Or) { expr = Expr::Logical(LogicOp::Or, Box::new(expr), Box::new(self.parse_and()?)); }
        Ok(expr)
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, String> {
        self.expect(TokenKind::Newline, "expected newline after ':'")?;
        self.expect(TokenKind::Indent, "expected indented block")?;
        self.skip_newlines();
        let mut body = Vec::new();
        while self.peek().kind != TokenKind::Dedent && self.peek().kind != TokenKind::Eof {
            body.push(self.parse_stmt()?); self.skip_newlines();
        }
        self.expect(TokenKind::Dedent, "expected dedent")?;
        Ok(body)
    }

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        if self.match_token(&TokenKind::Def) {
            let n = if let TokenKind::Name(n) = &self.expect(TokenKind::Name("".into()), "expected func name")?.kind { n.clone() } else { unreachable!() };
            self.expect(TokenKind::Lparen, "expected '('")?;
            let mut params = Vec::new();
            if !self.match_token(&TokenKind::Rparen) {
                loop {
                    if let TokenKind::Name(p) = &self.expect(TokenKind::Name("".into()), "expected param")?.kind { params.push(p.clone()); }
                    if !self.match_token(&TokenKind::Comma) { break; }
                }
                self.expect(TokenKind::Rparen, "expected ')'")?;
            }
            self.expect(TokenKind::Colon, "expected ':'")?;
            return Ok(Stmt::FunctionDef(n, params, self.parse_block()?));
        }
        if self.match_token(&TokenKind::If) {
            let test = self.parse_expr()?; self.expect(TokenKind::Colon, "expected ':'")?; let body = self.parse_block()?;
            let mut curr_test = test; let mut curr_body = body; let mut elifs = Vec::new();
            self.skip_newlines();
            while self.match_token(&TokenKind::Elif) {
                let e_test = self.parse_expr()?; self.expect(TokenKind::Colon, "expected ':'")?;
                elifs.push((e_test, self.parse_block()?)); self.skip_newlines();
            }
            let orelse = if self.match_token(&TokenKind::Else) { self.expect(TokenKind::Colon, "expected ':'")?; self.parse_block()? } else { vec![] };
            // Fold elifs into nested Else
            let mut final_else = orelse;
            for (t, b) in elifs.into_iter().rev() { final_else = vec![Stmt::If(t, b, final_else)]; }
            return Ok(Stmt::If(curr_test, curr_body, final_else));
        }
        if self.match_token(&TokenKind::While) {
            let test = self.parse_expr()?; self.expect(TokenKind::Colon, "expected ':'")?;
            return Ok(Stmt::While(test, self.parse_block()?));
        }
        if self.match_token(&TokenKind::For) {
            let var = if let TokenKind::Name(n) = &self.expect(TokenKind::Name("".into()), "expected loop var")?.kind { n.clone() } else { unreachable!() };
            self.expect(TokenKind::In, "expected 'in'")?; let iter = self.parse_expr()?;
            self.expect(TokenKind::Colon, "expected ':'")?;
            return Ok(Stmt::For(var, iter, self.parse_block()?));
        }
        if self.match_token(&TokenKind::Return) {
            if self.match_token(&TokenKind::Newline) { return Ok(Stmt::Return(None)); }
            let expr = self.parse_expr()?; self.expect(TokenKind::Newline, "expected newline")?; return Ok(Stmt::Return(Some(expr)));
        }
        if self.match_token(&TokenKind::Break) { self.expect(TokenKind::Newline, "expected newline")?; return Ok(Stmt::Break); }
        if self.match_token(&TokenKind::Continue) { self.expect(TokenKind::Newline, "expected newline")?; return Ok(Stmt::Continue); }
        if self.match_token(&TokenKind::Pass) { self.expect(TokenKind::Newline, "expected newline")?; return Ok(Stmt::Pass); }

        let expr = self.parse_expr()?;
        if self.match_token(&TokenKind::Equal) {
            let val = self.parse_expr()?; self.expect(TokenKind::Newline, "expected newline")?;
            return match expr {
                Expr::Name(n) => Ok(Stmt::Assign(n, val)),
                Expr::Subscript(obj, idx) => Ok(Stmt::AssignIndex(*obj, *idx, val)),
                Expr::Attribute(obj, attr) => Ok(Stmt::AssignAttr(*obj, attr, val)),
                _ => Err("invalid assignment target".into()),
            };
        } else if self.match_token(&TokenKind::PlusEq) || self.match_token(&TokenKind::MinusEq) {
            let op = if self.prev().kind == TokenKind::PlusEq { Op::Add } else { Op::Sub };
            let val = self.parse_expr()?; self.expect(TokenKind::Newline, "expected newline")?;
            if let Expr::Name(n) = expr { return Ok(Stmt::AugAssign(n, op, val)); }
            return Err("augmented assignment only supports simple variables".into());
        }

        self.expect(TokenKind::Newline, "expected newline after expression")?;
        Ok(Stmt::Expr(expr))
    }

    fn parse_module(&mut self) -> Result<Vec<Stmt>, String> {
        let mut body = Vec::new(); self.skip_newlines();
        while self.peek().kind != TokenKind::Eof { body.push(self.parse_stmt()?); self.skip_newlines(); }
        Ok(body)
    }
}

// =========================================================================
// Environment & Values
// =========================================================================

#[derive(Clone)]
enum PyValue {
    None, Bool(bool), Int(i64), Float(f64), Str(String),
    List(Rc<RefCell<Vec<PyValue>>>),
    Dict(Rc<RefCell<HashMap<String, PyValue>>>),
    Function { name: String, params: Vec<String>, body: Rc<Vec<Stmt>>, closure: Rc<RefCell<Env>> },
    Builtin(String, Rc<dyn Fn(&mut Runtime, Vec<PyValue>) -> Result<PyValue, String>>),
    BuiltinMethod(Rc<RefCell<Vec<PyValue>>>, String), // Simple hack for list.append
}

impl PartialEq for PyValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (PyValue::None, PyValue::None) => true,
            (PyValue::Bool(a), PyValue::Bool(b)) => a == b,
            (PyValue::Int(a), PyValue::Int(b)) => a == b,
            (PyValue::Float(a), PyValue::Float(b)) => a == b,
            (PyValue::Str(a), PyValue::Str(b)) => a == b,
            _ => false,
        }
    }
}

impl fmt::Display for PyValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PyValue::None => write!(f, "None"), PyValue::Bool(b) => write!(f, "{}", if *b { "True" } else { "False" }),
            PyValue::Int(i) => write!(f, "{}", i), PyValue::Float(n) => write!(f, "{}", n),
            PyValue::Str(s) => write!(f, "{}", s),
            PyValue::List(l) => {
                let items: Vec<String> = l.borrow().iter().map(|v| match v { PyValue::Str(s) => format!("'{}'", s), _ => v.to_string() }).collect();
                write!(f, "[{}]", items.join(", "))
            }
            PyValue::Dict(d) => {
                let items: Vec<String> = d.borrow().iter().map(|(k, v)| format!("'{}': {}", k, v)).collect();
                write!(f, "{{{}}}", items.join(", "))
            }
            _ => write!(f, "<object>"),
        }
    }
}

impl PyValue {
    fn is_truthy(&self) -> bool {
        match self {
            PyValue::None => false, PyValue::Bool(b) => *b,
            PyValue::Int(i) => *i != 0, PyValue::Float(f) => *f != 0.0,
            PyValue::Str(s) => !s.is_empty(), PyValue::List(l) => !l.borrow().is_empty(),
            PyValue::Dict(d) => !d.borrow().is_empty(), _ => true,
        }
    }
    fn as_number(&self) -> Result<f64, String> {
        match self { PyValue::Int(i) => Ok(*i as f64), PyValue::Float(f) => Ok(*f),
            PyValue::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }), _ => Err("expected number".into()),
        }
    }
    fn as_dict_key(&self) -> Result<String, String> {
        match self { PyValue::Str(s) => Ok(s.clone()), PyValue::Int(i) => Ok(i.to_string()),
            _ => Err("unhashable type for dict key".into())
        }
    }
}

struct Env { parent: Option<Rc<RefCell<Env>>>, vars: HashMap<String, PyValue> }
impl Env {
    fn new(parent: Option<Rc<RefCell<Env>>>) -> Rc<RefCell<Self>> { Rc::new(RefCell::new(Env { parent, vars: HashMap::new() })) }
    fn set(&mut self, name: &str, value: PyValue) { self.vars.insert(name.to_string(), value); }
    fn assign(&mut self, name: &str, value: PyValue) {
        if self.vars.contains_key(name) { self.vars.insert(name.to_string(), value); return; }
        if let Some(parent) = &self.parent { if parent.borrow().get_opt(name).is_some() { parent.borrow_mut().assign(name, value); return; } }
        self.vars.insert(name.to_string(), value);
    }
    fn get_opt(&self, name: &str) -> Option<PyValue> {
        if let Some(val) = self.vars.get(name) { return Some(val.clone()); }
        if let Some(p) = &self.parent { return p.borrow().get_opt(name); }
        None
    }
    fn get(&self, name: &str) -> Result<PyValue, String> { self.get_opt(name).ok_or_else(|| format!("name '{}' is not defined", name)) }
}

// =========================================================================
// Evaluator
// =========================================================================

struct Runtime { globals: Rc<RefCell<Env>> }

enum ExecStatus { Continue, Return(PyValue), Break, ContinueLoop }

fn eval_expr(rt: &mut Runtime, env: &Rc<RefCell<Env>>, expr: &Expr) -> Result<PyValue, String> {
    match expr {
        Expr::NoneVal => Ok(PyValue::None), Expr::Bool(b) => Ok(PyValue::Bool(*b)),
        Expr::Int(v) => Ok(PyValue::Int(*v)), Expr::Float(v) => Ok(PyValue::Float(*v)),
        Expr::String(v) => Ok(PyValue::Str(v.clone())), Expr::Name(n) => env.borrow().get(n),
        Expr::List(items) => {
            let mut list = Vec::new();
            for item in items { list.push(eval_expr(rt, env, item)?); }
            Ok(PyValue::List(Rc::new(RefCell::new(list))))
        }
        Expr::Dict(pairs) => {
            let mut dict = HashMap::new();
            for (k_expr, v_expr) in pairs {
                let k = eval_expr(rt, env, k_expr)?.as_dict_key()?;
                let v = eval_expr(rt, env, v_expr)?;
                dict.insert(k, v);
            }
            Ok(PyValue::Dict(Rc::new(RefCell::new(dict))))
        }
        Expr::BinOp(op, l_expr, r_expr) => {
            let l = eval_expr(rt, env, l_expr)?; let r = eval_expr(rt, env, r_expr)?; apply_binop(*op, l, r)
        }
        Expr::UnaryOp(op, operand) => {
            let v = eval_expr(rt, env, operand)?;
            match op {
                Op::Neg => match v { PyValue::Int(i) => Ok(PyValue::Int(-i)), _ => Ok(PyValue::Float(-v.as_number()?)) },
                Op::Not => Ok(PyValue::Bool(!v.is_truthy())),
                _ => Err("unsupported unary operator".into()),
            }
        }
        Expr::Compare(op, l_expr, r_expr) => {
            let l = eval_expr(rt, env, l_expr)?; let r = eval_expr(rt, env, r_expr)?; apply_compare(*op, l, r)
        }
        Expr::Logical(op, l_expr, r_expr) => {
            let l = eval_expr(rt, env, l_expr)?;
            match op {
                LogicOp::And => if !l.is_truthy() { Ok(l) } else { eval_expr(rt, env, r_expr) },
                LogicOp::Or => if l.is_truthy() { Ok(l) } else { eval_expr(rt, env, r_expr) },
            }
        }
        Expr::Call(func_expr, args_exprs) => {
            let func = eval_expr(rt, env, func_expr)?;
            let mut args = Vec::new(); for a in args_exprs { args.push(eval_expr(rt, env, a)?); }
            call_function(rt, func, args)
        }
        Expr::Attribute(val_expr, attr) => {
            let val = eval_expr(rt, env, val_expr)?;
            match val {
                PyValue::List(l) if attr == "append" => Ok(PyValue::BuiltinMethod(Rc::clone(&l), "append".into())),
                _ => Err(format!("unsupported attribute access: {}", attr))
            }
        }
        Expr::Subscript(val_expr, idx_expr) => {
            let val = eval_expr(rt, env, val_expr)?; let idx = eval_expr(rt, env, idx_expr)?;
            match val {
                PyValue::List(l) => {
                    let i = match idx { PyValue::Int(i) => i, _ => return Err("list indices must be int".into()) };
                    let b = l.borrow();
                    if i < 0 || i as usize >= b.len() { Err("list index out of range".into()) } else { Ok(b[i as usize].clone()) }
                }
                PyValue::Dict(d) => {
                    let k = idx.as_dict_key()?;
                    d.borrow().get(&k).cloned().ok_or_else(|| format!("KeyError: '{}'", k))
                }
                _ => Err("unsupported subscript target".into()),
            }
        }
    }
}

fn apply_binop(op: Op, l: PyValue, r: PyValue) -> Result<PyValue, String> {
    if op == Op::Add { if let (PyValue::Str(a), PyValue::Str(b)) = (&l, &r) { return Ok(PyValue::Str(format!("{}{}", a, b))); } }
    if let (PyValue::Int(a), PyValue::Int(b)) = (&l, &r) {
        return match op { Op::Add => Ok(PyValue::Int(a + b)), Op::Sub => Ok(PyValue::Int(a - b)), Op::Mul => Ok(PyValue::Int(a * b)),
            Op::Div => Ok(PyValue::Float((*a as f64) / (*b as f64))), Op::Mod => Ok(PyValue::Int(a % b)), _ => Err("bad binop".into()) };
    }
    let a = l.as_number()?; let b = r.as_number()?;
    match op { Op::Add => Ok(PyValue::Float(a + b)), Op::Sub => Ok(PyValue::Float(a - b)), Op::Mul => Ok(PyValue::Float(a * b)),
        Op::Div => Ok(PyValue::Float(a / b)), Op::Mod => Ok(PyValue::Float((a as i64 % b as i64) as f64)), _ => Err("bad binop".into()) }
}

fn apply_compare(op: Op, l: PyValue, r: PyValue) -> Result<PyValue, String> {
    if let (PyValue::Str(a), PyValue::Str(b)) = (&l, &r) {
        return Ok(PyValue::Bool(match op { Op::Eq => a == b, Op::Ne => a != b, _ => false }));
    }
    let a = l.as_number()?; let b = r.as_number()?;
    Ok(PyValue::Bool(match op { Op::Eq => a == b, Op::Ne => a != b, Op::Lt => a < b, Op::Le => a <= b, Op::Gt => a > b, Op::Ge => a >= b, _ => false }))
}

fn exec_stmt(rt: &mut Runtime, env: &Rc<RefCell<Env>>, stmt: &Stmt) -> Result<ExecStatus, String> {
    match stmt {
        Stmt::Expr(expr) => { eval_expr(rt, env, expr)?; Ok(ExecStatus::Continue) }
        Stmt::Assign(name, val_expr) => { let val = eval_expr(rt, env, val_expr)?; env.borrow_mut().assign(name, val); Ok(ExecStatus::Continue) }
        Stmt::AssignIndex(target, idx_expr, val_expr) => {
            let t = eval_expr(rt, env, target)?; let idx = eval_expr(rt, env, idx_expr)?; let val = eval_expr(rt, env, val_expr)?;
            match t {
                PyValue::List(l) => {
                    let i = match idx { PyValue::Int(i) => i, _ => return Err("list index must be int".into()) };
                    let mut b = l.borrow_mut();
                    if i < 0 || i as usize >= b.len() { return Err("list assignment index out of range".into()); }
                    b[i as usize] = val;
                }
                PyValue::Dict(d) => { d.borrow_mut().insert(idx.as_dict_key()?, val); }
                _ => return Err("unsupported item assignment target".into()),
            }
            Ok(ExecStatus::Continue)
        }
        Stmt::AssignAttr(_, _, _) => Err("Attribute assignment not implemented".into()),
        Stmt::AugAssign(name, op, val_expr) => {
            let current = env.borrow().get(name)?; let val = eval_expr(rt, env, val_expr)?;
            let new_val = apply_binop(*op, current, val)?; env.borrow_mut().assign(name, new_val);
            Ok(ExecStatus::Continue)
        }
        Stmt::If(test, body, orelse) => {
            if eval_expr(rt, env, test)?.is_truthy() { exec_block(rt, env, body) } else { exec_block(rt, env, orelse) }
        }
        Stmt::While(test, body) => {
            while eval_expr(rt, env, test)?.is_truthy() {
                match exec_block(rt, env, body)? {
                    ExecStatus::Return(v) => return Ok(ExecStatus::Return(v)),
                    ExecStatus::Break => break,
                    _ => {}
                }
            }
            Ok(ExecStatus::Continue)
        }
        Stmt::For(var, iter_expr, body) => {
            let iter = eval_expr(rt, env, iter_expr)?;
            let items = match iter {
                PyValue::List(l) => l.borrow().clone(),
                PyValue::Str(s) => s.chars().map(|c| PyValue::Str(c.to_string())).collect(),
                _ => return Err("object is not iterable".into()),
            };
            for item in items {
                env.borrow_mut().assign(var, item);
                match exec_block(rt, env, body)? {
                    ExecStatus::Return(v) => return Ok(ExecStatus::Return(v)),
                    ExecStatus::Break => break,
                    _ => {}
                }
            }
            Ok(ExecStatus::Continue)
        }
        Stmt::FunctionDef(name, params, body) => {
            env.borrow_mut().set(name, PyValue::Function { name: name.clone(), params: params.clone(), body: Rc::new(body.clone()), closure: Rc::clone(env) });
            Ok(ExecStatus::Continue)
        }
        Stmt::Return(expr_opt) => {
            let val = if let Some(expr) = expr_opt { eval_expr(rt, env, expr)? } else { PyValue::None };
            Ok(ExecStatus::Return(val))
        }
        Stmt::Break => Ok(ExecStatus::Break),
        Stmt::Continue => Ok(ExecStatus::ContinueLoop),
        Stmt::Pass => Ok(ExecStatus::Continue),
    }
}

fn exec_block(rt: &mut Runtime, env: &Rc<RefCell<Env>>, stmts: &[Stmt]) -> Result<ExecStatus, String> {
    for stmt in stmts {
        let status = exec_stmt(rt, env, stmt)?;
        if !matches!(status, ExecStatus::Continue) { return Ok(status); }
    }
    Ok(ExecStatus::Continue)
}

fn call_function(rt: &mut Runtime, func: PyValue, args: Vec<PyValue>) -> Result<PyValue, String> {
    match func {
        PyValue::Builtin(_, fn_ptr) => fn_ptr(rt, args),
        PyValue::BuiltinMethod(obj, name) => {
            if name == "append" {
                if args.len() != 1 { return Err("append() takes 1 argument".into()); }
                obj.borrow_mut().push(args[0].clone()); Ok(PyValue::None)
            } else { Err("unknown method".into()) }
        }
        PyValue::Function { name, params, body, closure } => {
            if args.len() != params.len() { return Err(format!("{}() expected {} args, got {}", name, params.len(), args.len())); }
            let local = Env::new(Some(closure));
            for (p, a) in params.iter().zip(args) { local.borrow_mut().set(p, a); }
            match exec_block(rt, &local, &body)? { ExecStatus::Return(v) => Ok(v), _ => Ok(PyValue::None) }
        }
        _ => Err("object is not callable".into()),
    }
}

// =========================================================================
// Builtins & Main
// =========================================================================

fn install_builtins(globals: &Rc<RefCell<Env>>) {
    let mut env = globals.borrow_mut();
    env.set("print", PyValue::Builtin("print".into(), Rc::new(|_, args| {
        println!("{}", args.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" ")); Ok(PyValue::None)
    })));
    env.set("len", PyValue::Builtin("len".into(), Rc::new(|_, args| {
        if args.len() != 1 { return Err("len() expects 1 arg".into()); }
        match &args[0] {
            PyValue::Str(s) => Ok(PyValue::Int(s.len() as i64)),
            PyValue::List(l) => Ok(PyValue::Int(l.borrow().len() as i64)),
            PyValue::Dict(d) => Ok(PyValue::Int(d.borrow().len() as i64)),
            _ => Err("len() unsupported type".into()),
        }
    })));
    env.set("range", PyValue::Builtin("range".into(), Rc::new(|_, args| {
        if args.len() != 1 { return Err("range() expects 1 arg".into()); }
        let end = match args[0] { PyValue::Int(i) => i, _ => return Err("range() requires int".into()) };
        let items: Vec<PyValue> = (0..end).map(PyValue::Int).collect();
        Ok(PyValue::List(Rc::new(RefCell::new(items))))
    })));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 { eprintln!("Usage: ./py4 <script.py>"); process::exit(1); }
    let source = fs::read_to_string(&args[1]).unwrap_or_else(|_| { eprintln!("cannot open {}", args[1]); process::exit(1); });

    let globals = Env::new(None); install_builtins(&globals);
    let mut rt = Runtime { globals: Rc::clone(&globals) };

    let tokens = lex_source(&source).unwrap_or_else(|e| { eprintln!("Lex Error: {}", e); process::exit(1); });
    let mut parser = Parser::new(&tokens, &args[1]);
    let module = parser.parse_module().unwrap_or_else(|e| { eprintln!("Parse Error: {}", e); process::exit(1); });

    if let Err(e) = exec_block(&mut rt, &globals, &module) { eprintln!("Runtime Error: {}", e); process::exit(1); }
}