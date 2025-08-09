A tool to analyze English text

Installation:
```shell
cargo install booky
```

### Lexicon

The `word` sub-command lists words from the built-in lexicon.

- Looks up all entries matching the provided word
- With no options, all entries are listed
- The `-f` option lists all known word forms
- The `-c` option filters words by class.  Provide a comma-separated list to
  specify classes:

Option | Word Class
-------|-----------
`A`    | Adjective
`Av`   | Adverb
`C`    | Conjunction
`D`    | Determiner
`I`    | Interjection
`N`    | Noun
`P`    | Preposition
`Pn`   | Pronoun
`V`    | Verb

### Reading a Text

The `read` sub-command reads UTF-8 text from `stdin`, which can be redirected
from a file.  With no additional options, a summary of token kinds is listed:

```
> booky read < Dr_Jeckyll_And_Mr_Hyde.txt

 3915 l Lexicon
    1 f Foreign
    4 o Ordinal
    0 r Roman
    2 n Number
   12 a Acronym
   37 p Proper
   16 s Symbol
    7 u Unknown
```

Comma-separated options can be added to list all tokens of a kind.

Option | Kind    | Description
-------|---------|--------------------------
`l`    | Lexicon | Found in built-in lexicon
`f`    | Foreign | Foreign words (non-English)
`o`    | Ordinal | Ordinal numbers (1st, 2nd, etc.)
`r`    | Roman   | Roman numerals (IV, LXI, etc.)
`n`    | Number  | Other words containing numbers
`a`    | Acronym | Acronyms / initialisms (ALL-CAPS)
`p`    | Proper  | Proper names / nouns
`s`    | Symbol  | Symbols / letters
`u`    | Unknown | Unknown (no other kind)
`A`    | All     | All kinds

### Highlighting

The `hl` sub-command adds highlighting to a text.
