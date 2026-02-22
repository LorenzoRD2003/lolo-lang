# Manual del lenguaje `lolo-lang`

## Introducción

Por ahora, el programa estará completamente contenido dentro de una sola función `main`. Un bloque es simplemente una lista de statements.

`program ::= main_block`
`main_block ::= "main" "{" block "}"`
`block ::= stmt*`

## Tipos de datos

- `i32` (enteros)
- `bool` (booleanos)

## Expresiones

Expresiones genéricas; su tipo concreto es decidido durante lowering a SSA IR.

- `expr ::= var | const | unary_expr | binary_expr`
- `unary_expr ::= neg | not`
- `binary_expr ::= arithmetic_expr | comparison_expr | logical_expr`

### Tipos de AST

- `var` / `const`: nodos genéricos. el tipo concreto será resuelto en SSA IR. Las variables deben empezar con una letra o `_`.
- `unary_expr` / `binary_expr`: contiene operandos deben ser compatibles según operación (i32/bool).

## Instrucciones (statements)

`stmt ::= let_stmt | return_stmt | if_stmt | print_stmt`

### Instrucción de asignación

Asignación simple; `var` creado en scope actual.

`let_stmt ::= "let" var "=" expr ";"`

### Instrucción de retorno

Solamente puede aparecer al final de `main` o de algún bloque (dentro de un branch). Es el **terminador** de un bloque.

`return_stmt ::= "return" expr ";"`

### Instrucción de branch

Genera bloques separados en SSA; phi nodes si se necesitan.

`if_stmt ::= "if" expr "{" block "}" [ "else" "{" block "}" ]`

### Instrucción de print

Es side-effect en SSA/IR, para que no lo elimine DCE.

`print_stmt ::= "print" expr ";"`

## Operaciones

### Operaciones unarias

- `neg ::= "neg" expr`
- `not ::= "not" expr`

### Operaciones binarias

Luego se implementarán sus versiones inline.

- Aritméticas: `arithmetic_expr ::= "add" expr expr | "sub" expr expr | "mul" expr expr | "div" expr expr`
- Comparaciones: `comparison_expr ::= "eq" expr expr | "neq" expr expr | "gt" expr expr | "lt" expr expr | "gte" expr expr | "lte" expr expr`
- Lógicas: `logical_expr ::= "and" expr expr | "or" expr expr | "xor" expr expr`

### Precedencia y asociatividad de operadores

Las expresiones pueden agruparse mediante paréntesis, lo que incrementa su precedencia.

| Nivel |              Operadores               | Asociatividad  |            Notas            |
| :---: | :-----------------------------------: | :------------: | :-------------------------: |
|   1   |             `neg`, `not`              |    Derecha     |     Operadores unarios      |
|   2   |             `mul`, `div`              |   Izquierda    | Aritméticos multiplicativos |
|   3   |             `add`, `sub`              |   Izquierda    |    Aritméticos aditivos     |
|   4   | `eq`, `neq`, `gt`, `lt`, `gte`, `lte` | No asociativos |        Comparativos.        |
|   5   |                 `and`                 |   Izquierda    |         Lógico AND          |
|   6   |                 `or`                  |   Izquierda    |          Lógico OR          |
|   7   |                 `xor`                 |   Izquierda    |         Lógico XOR          |

## Whitespaces

Los espacios entre tokens están permitidos, y no afectan la semántica del lenguaje. Tampoco forman parte del Span lógico del nodo (_únicamente delimitan tokens_).

## Reglas de lexing

Se aplica la siguiente jerarquía de reconocimiento:

1. EOF.
2. Whitespaces: No generan tokens, consumir caracteres hasta que deje de serlo
3. Delimitadores: Son triviales: un solo caracter
4. Literales: Suelen tener patrones claros. Un número empieza con un dígito, y un booleano es true/false.
5. Identificadores / keywords
6. Operadores: aun no tenemos, pero cuando sean +, -, ==, etc.
7. Error
