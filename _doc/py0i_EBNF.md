根據您提供的 `py0i.rs` 原始碼，我為這個微型 Python 直譯器 (Subset of Python) 整理出了對應的 EBNF (Extended Backus-Naur Form) 語法。

這裡將語法分為 **語法規則 (Parser Rules)** 與 **詞法規則 (Lexical Rules)** 兩部分：

### 語法規則 (Parser Rules)

```ebnf
module      ::= (stmt | NEWLINE)* EOF

block       ::= NEWLINE INDENT NEWLINE* (stmt NEWLINE*)+ DEDENT

stmt        ::= function_def
              | if_stmt
              | while_stmt
              | return_stmt
              | pass_stmt
              | assign_stmt
              | expr_stmt

function_def::= "def" NAME "(" [NAME ("," NAME)*] ")" ":" block
if_stmt     ::= "if" expr ":" block ["else" ":" block]
while_stmt  ::= "while" expr ":" block
return_stmt ::= "return" [expr] NEWLINE
pass_stmt   ::= "pass" NEWLINE
assign_stmt ::= NAME "=" expr NEWLINE
expr_stmt   ::= expr NEWLINE

expr        ::= comparison
comparison  ::= term (comp_op term)*
comp_op     ::= "==" | "!=" | "<" | "<=" | ">" | ">="

term        ::= factor (("+" | "-") factor)*
factor      ::= unary (("*" | "/" | "%") unary)*

unary       ::= "-" unary 
              | primary

primary     ::= atom postfix*

atom        ::= INT 
              | FLOAT 
              | STRING 
              | NAME 
              | "(" expr ")"

postfix     ::= "(" [expr ("," expr)*] ")"   (* 函數呼叫 Call *)
              | "." NAME                     (* 屬性存取 Attribute *)
              | "[" expr "]"                 (* 索引存取 Subscript *)
```

---

### 詞法規則 (Lexical Rules / Tokens)

以下規則描述了 Lexer (`lex_source` 函數) 是如何解析字元流的：

```ebnf
NAME    ::= [a-zA-Z_] [a-zA-Z0-9_]*
          (* 排除關鍵字: def, if, else, while, return, pass *)

INT     ::= [0-9]+
FLOAT   ::= [0-9]+ "." [0-9]*

STRING  ::= "'" [^']* "'" 
          | '"' [^"]* '"'
          (* 支援跳脫字元: \n, \t, \\, \', \" *)

NEWLINE ::= '\n' | '\r\n'  (* 包含跳過空白與 # 註解後的換行 *)
INDENT  ::= (* 縮排層級增加 (Space=1, Tab=4) *)
DEDENT  ::= (* 縮排層級減少 *)
EOF     ::= (* 檔案結尾 *)
```

### 語法特性解析 (對應 Rust 程式碼細節)

1. **運算子優先級 (Operator Precedence)**：
   由高到低精準對應了原始碼中的遞迴下降解析器：
   `postfix (呼叫/屬性/索引)` > `unary (負號)` > `factor (* / %)` > `term (+ -)` > `comparison (== != < <= > >=)`
2. **區塊與縮排 (Block & Indentation)**：
   必須以 `:` 結尾，隨後需要一個 `NEWLINE`，接著增加縮排 `INDENT`，包含一到多個陳述式 (Statement)，最後以 `DEDENT` 結束。區塊內部的空行會被 `skip_newlines()` 自動忽略。
3. **沒有複合賦值或多重賦值**：
   `py0i` 只支援最基礎的 `NAME = expr` (例如 `a = 1`)，不支援 `a = b = 1` 或是 `a += 1`。
4. **Trailing Comma (尾隨逗號)**：
   根據 `parse_postfix` 與 `parse_stmt` 中迴圈的寫法 (`if !self.match_token(&TokenKind::Comma) { break; }`)，參數或陣列若有逗號，後面**必須**接下一個元素，不支援尾隨逗號 (例如 `foo(a, b,)` 會產生錯誤)。