justfile grammar
================

Justfiles are processed by a mildly context-sensitive tokenizer
and a recursive descent parser. The grammar is LL(k), for an
unknown but hopefully reasonable value of k.

tokens
------

```
BACKTICK   = `[^`\n\r]*`
COMMENT    = #([^!].*)?$
DEDENT     = emitted when indentation decreases
EOF        = emitted at the end of the file
ESCAPE     = \n | \r | \t | \" | \\
INDENT     = emitted when indentation increases
NAME       = [a-zA-Z_][a-zA-Z0-9_-]*
NEWLINE    = \n|\r\n
RAW_STRING = '[^'\r\n]*'
STRING     = "(ESCAPE|[^"])*"
TEXT       = recipe text, only matches in a recipe body
```

grammar syntax
--------------

```
|   alternation
()  grouping
_?  option (0 or 1 times)
_*  repetition (0 or more times)
_+  repetition (1 or more times)
```

grammar
-------

```
justfile      : item* EOF

item          : recipe
              | alias
              | assignment
              | export
              | setting
              | eol
              | module

eol           : NEWLINE
              | COMMENT NEWLINE

alias         : 'alias' NAME ':=' NAME eol

assignment    : NAME ':=' expression eol

export        : 'export' assignment

setting       : 'set' 'shell' ':=' '[' string (',' string)* ','? ']' eol
              | 'set' 'module-experiment' ':=' true eol

expression    : value '+' expression
              | value

value         : NAME '(' sequence? ')'
              | STRING
              | RAW_STRING
              | BACKTICK
              | NAME
              | '(' expression ')'

string        : STRING
              | RAW_STRING

sequence      : expression ',' sequence
              | expression ','?

recipe        : '@'? NAME parameter* ('+' parameter)? ':' dependency* eol body?

parameter     : NAME
              | NAME '=' value

dependency    : NAME
              | '(' NAME expression* ')

body          : INDENT line+ DEDENT

line          : (TEXT | interpolation)* NEWLINE

interpolation : '{{' expression '}}'

module        : NAME '::' eol suite?

suite         : INDENT item* DEDENT
```
