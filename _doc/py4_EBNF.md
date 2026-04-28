這是一個非常棒的挑戰！為了讓語法「更完整」且更接近真正的 Python，我擴充了以下核心功能：

1. **資料結構**：加入 **List (串列)** `[1, 2]` 與 **Dict (字典)** `{"a": 1}` 的字面值與操作。
2. **控制流**：加入 **`for ... in ...`** 迴圈、**`elif`**、**`break`** 與 **`continue`**。
3. **邏輯運算**：加入 **`and`**, **`or`**, **`not`** (支援短路求值 Short-circuiting)。
4. **進階賦值**：支援索引賦值 `a[0] = 1` 字典賦值 `d["k"] = 2` 以及複合賦值 `a += 1`, `a -= 1`。
5. **內建功能**：加入 `range()` (為了配合 for 迴圈) 以及 List 的 `.append()` 方法支援。

---

### 1. 擴充後的 EBNF 語法

```ebnf
module      ::= (stmt | NEWLINE)* EOF

block       ::= NEWLINE INDENT NEWLINE* (stmt NEWLINE*)+ DEDENT

stmt        ::= function_def
              | if_stmt
              | while_stmt
              | for_stmt
              | return_stmt
              | break_stmt
              | continue_stmt
              | pass_stmt
              | assign_stmt
              | expr_stmt

function_def::= "def" NAME "(" [NAME ("," NAME)* [","]] ")" ":" block
if_stmt     ::= "if" expr ":" block ("elif" expr ":" block)* ["else" ":" block]
while_stmt  ::= "while" expr ":" block
for_stmt    ::= "for" NAME "in" expr ":" block
return_stmt ::= "return" [expr] NEWLINE
break_stmt  ::= "break" NEWLINE
continue_stmt::="continue" NEWLINE
pass_stmt   ::= "pass" NEWLINE
assign_stmt ::= target ("=" | "+=" | "-=") expr NEWLINE
expr_stmt   ::= expr NEWLINE

target      ::= NAME 
              | primary "[" expr "]"
              | primary "." NAME

expr        ::= logical_or
logical_or  ::= logical_and ("or" logical_and)*
logical_and ::= logical_not ("and" logical_not)*
logical_not ::= "not" logical_not | comparison

comparison  ::= term (comp_op term)*
comp_op     ::= "==" | "!=" | "<" | "<=" | ">" | ">="

term        ::= factor (("+" | "-") factor)*
factor      ::= unary (("*" | "/" | "%") unary)*

unary       ::= "-" unary | primary

primary     ::= atom postfix*

atom        ::= INT | FLOAT | STRING | NAME 
              | "None" | "True" | "False"
              | "[" [expr ("," expr)* [","]] "]"               (* List *)
              | "{" [dict_pair ("," dict_pair)* [","]] "}"     (* Dict *)
              | "(" expr ")"

dict_pair   ::= expr ":" expr

postfix     ::= "(" [expr ("," expr)* [","]] ")"   (* 呼叫 Call *)
              | "." NAME                     (* 屬性 Attribute *)
              | "[" expr "]"                 (* 索引 Subscript *)
```
