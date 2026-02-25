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

| Binding power (`lbp`/`rbp`) |         Operadores textuales          |             Simbolos             | Asociatividad  |            Notas            |
| :-------------------------: | :-----------------------------------: | :------------------------------: | :------------: | :-------------------------: |
|            70/70            |             `neg`, `not`              |             `-`, `!`             |    Derecha     |     Operadores unarios      |
|            60/61            |             `mul`, `div`              |             `*`, `/`             |   Izquierda    | Aritméticos multiplicativos |
|            50/51            |             `add`, `sub`              |             `+`, `-`             |   Izquierda    |    Aritméticos aditivos     |
|            40/41            | `eq`, `neq`, `gt`, `lt`, `gte`, `lte` | `==`, `!=`, `>`, `<`, `>=`, `<=` | No asociativos |        Comparativos.        |
|            30/31            |                 `and`                 |               `&&`               |   Izquierda    |         Lógico AND          |
|            20/21            |                 `or`                  |              `\|\|`              |   Izquierda    |          Lógico OR          |
|            10/11            |                 `xor`                 |               `^^`               |   Izquierda    |         Lógico XOR          |

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

## Pratt Parsing

Pratt parsing = Top-Down Operator Precedence Parsing. La idea central no es parsear por gramática explícita, sino por potencia de enlace (binding power). Cada Token generado por el lexer puede comportarse como:

- _nud (null denotation)_, cuando aparece al inicio. ¿Qué significa este token si empieza una expresión?
- _led (left denotation)_, cuando aparece con algo a la izquierda. ¿Qué significa este token si ya hay una expresión a la izquierda?

Hay una sola función principal, `parse_expr(min_bp)`, la cual:

1. Lee un token, ejecutando su función `nud()` y obteniendo un `lhs`.
2. Mientras el siguiente token sea un operador y su binding power sea mayor que `min_bp`, se consume el operador y se ejecuta `led(lhs)`, obteniendo un nuevo `lhs`.

Reglas del binding power (usadas para modificar la tabla anterior), para `lbp` (left binding power) y `rbp` (right binding power).

- Para operadores asociativos a izquierda, `lbp = X, rbp = X + 1`.
- Para operadores asociativos a derecha, `lbp = X, rbp = X`.
- Para operadores no asociativos, `lbp = X, rbp = X + 1`, pero con restricción semántica (prohibimos encadenarlos).
