use std::rc::Rc;
use std::cell::RefCell;
use std::time::{SystemTime, UNIX_EPOCH};
use std::thread::sleep;
use std::time::Duration;
use std::process::Command;
use std::env;

use crate::{PyValue, Env, Runtime, py_err, py_err_val}; // 清除了未使用的
use crate::{lex_source, Parser, eval_expr, TokenKind};

pub(crate) fn load_native_module(name: &str) -> Option<PyValue> {
    match name {
        "math" => Some(load_math()),
        "time" => Some(load_time()),
        "os" => Some(load_os()),
        "sys" => Some(load_sys()),
        "json" => Some(load_json()),
        _ => None,
    }
}

// ==========================
// 1. Math Module
// ==========================
fn load_math() -> PyValue {
    let env = Env::new(None);
    env.borrow_mut().set("pi", PyValue::Float(std::f64::consts::PI));
    env.borrow_mut().set("sqrt", PyValue::Builtin("sqrt".into(), Rc::new(|_, a, _| {
        if a.is_empty() { return py_err("TypeError", "sqrt() takes exactly 1 argument"); }
        Ok(PyValue::Float(a[0].as_num()?.sqrt()))
    })));
    PyValue::Module("math".into(), env)
}

// ==========================
// 2. Time Module
// ==========================
fn load_time() -> PyValue {
    let env = Env::new(None);
    env.borrow_mut().set("time", PyValue::Builtin("time".into(), Rc::new(|_, _, _| {
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        Ok(PyValue::Float(since_the_epoch.as_secs_f64()))
    })));
    
    env.borrow_mut().set("sleep", PyValue::Builtin("sleep".into(), Rc::new(|_, a, _| {
        if a.is_empty() { return py_err("TypeError", "sleep() takes exactly 1 argument"); }
        let secs = a[0].as_num()?;
        sleep(Duration::from_secs_f64(secs));
        Ok(PyValue::None)
    })));
    PyValue::Module("time".into(), env)
}

// ==========================
// 3. OS Module
// ==========================
fn load_os() -> PyValue {
    let env = Env::new(None);
    env.borrow_mut().set("getenv", PyValue::Builtin("getenv".into(), Rc::new(|_, a, _| {
        if a.is_empty() { return py_err("TypeError", "getenv() takes at least 1 argument"); }
        let key = if let PyValue::Str(s) = &a[0] { s } else { return py_err("TypeError", "key must be string"); };
        match env::var(key) {
            Ok(val) => Ok(PyValue::Str(val)),
            Err(_) => {
                if a.len() > 1 { Ok(a[1].clone()) } else { Ok(PyValue::None) }
            }
        }
    })));

    env.borrow_mut().set("system", PyValue::Builtin("system".into(), Rc::new(|_, a, _| {
        if a.is_empty() { return py_err("TypeError", "system() takes exactly 1 argument"); }
        let cmd_str = if let PyValue::Str(s) = &a[0] { s } else { return py_err("TypeError", "command must be string"); };
        let status = Command::new("sh").arg("-c").arg(cmd_str).status()
            .map_err(|e| py_err_val("OSError", &e.to_string()))?;
        Ok(PyValue::Int(status.code().unwrap_or(1) as i64))
    })));
    PyValue::Module("os".into(), env)
}

// ==========================
// 4. Sys Module
// ==========================
fn load_sys() -> PyValue {
    let env = Env::new(None);
    // 把 Rust 抓到的 arguments 傳給 Python (略過第一個 ./py4 執行檔本身)
    let args: Vec<PyValue> = env::args().skip(1).map(PyValue::Str).collect();
    env.borrow_mut().set("argv", PyValue::List(Rc::new(RefCell::new(args))));
    
    env.borrow_mut().set("exit", PyValue::Builtin("exit".into(), Rc::new(|_, a, _| {
        let code = if a.is_empty() { 0 } else { a[0].as_num()? as i32 };
        std::process::exit(code);
    })));
    PyValue::Module("sys".into(), env)
}

// ==========================
// 5. JSON Module (利用 Token 轉換的魔法！)
// ==========================
fn load_json() -> PyValue {
    let env = Env::new(None);
    
    // json.loads: 把 JSON 字串當作 Python 表達式來解析！
    env.borrow_mut().set("loads", PyValue::Builtin("loads".into(), Rc::new(|rt, a, _| {
        if a.is_empty() { return py_err("TypeError", "loads() takes exactly 1 argument"); }
        let json_str = if let PyValue::Str(s) = &a[0] { s } else { return py_err("TypeError", "expected string"); };
        
        let mut tokens = lex_source(json_str).map_err(|e| py_err_val("ValueError", &e))?;
        // 魔法：把 JSON 的關鍵字轉換成 Python 的關鍵字！
        for t in &mut tokens {
            if let TokenKind::Name(n) = &t.kind {
                match n.as_str() {
                    "true" => t.kind = TokenKind::TrueVal,
                    "false" => t.kind = TokenKind::FalseVal,
                    "null" => t.kind = TokenKind::NoneVal,
                    _ => {}
                }
            }
        }
        
        let mut p = Parser::new(&tokens, "<json>");
        let ast = p.parse_expr().map_err(|e| py_err_val("ValueError", &format!("Invalid JSON: {}", e)))?;
        eval_expr(rt, &Env::new(None), &ast)
    })));

    // json.dumps: 遞迴把 PyValue 轉為 JSON 字串
    env.borrow_mut().set("dumps", PyValue::Builtin("dumps".into(), Rc::new(|rt, a, _| {
        if a.is_empty() { return py_err("TypeError", "dumps() takes exactly 1 argument"); }
        
        fn dump_val(rt: &mut Runtime, v: &PyValue) -> Result<String, PyValue> {
            match v {
                PyValue::None => Ok("null".to_string()),
                PyValue::Bool(b) => Ok(if *b { "true".to_string() } else { "false".to_string() }),
                PyValue::Int(i) => Ok(i.to_string()),
                PyValue::Float(f) => Ok(f.to_string()),
                PyValue::Str(s) => Ok(format!("\"{}\"", s.replace('"', "\\\""))),
                PyValue::List(l) => {
                    let mut items = Vec::new();
                    for item in l.borrow().iter() { items.push(dump_val(rt, item)?); }
                    Ok(format!("[{}]", items.join(", ")))
                }
                PyValue::Tuple(t) => {
                    let mut items = Vec::new();
                    for item in t.iter() { items.push(dump_val(rt, item)?); }
                    Ok(format!("[{}]", items.join(", "))) // JSON 沒有 tuple，轉成 Array
                }
                PyValue::Dict(d) => {
                    let mut items = Vec::new();
                    for (k, val) in d.borrow().iter() {
                        items.push(format!("\"{}\": {}", k, dump_val(rt, val)?));
                    }
                    Ok(format!("{{{}}}", items.join(", ")))
                }
                _ => py_err("TypeError", "Object of this type is not JSON serializable")
            }
        }
        Ok(PyValue::Str(dump_val(rt, &a[0])?))
    })));

    PyValue::Module("json".into(), env)
}