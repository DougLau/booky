A tool to analyze English text

Installation:
```shell
cargo install booky
```

### Grouping Words by Kind

The `kind` sub-command reads UTF-8 text from `stdin`, which can be redirected
from a file.  With no additional options, a summary of words of each kind is
listed:

```
> booky kind < Dr_Jeckyll_And_Mr_Hyde.txt

 3850 d Dictionary
    1 f Foreign
    4 o Ordinal
    1 r Roman
    1 n Number
    9 a Acronym
   32 p Proper
    1 s Symbol
    6 u Unknown
```

Command-line options can be added to list all words of a kind.

Option | Kind       | Description
-------|------------|-----------------------------
`-d`   | Dictionary | Found in built-in dictionary
`-f`   | Foreign    | Foreign words (non-English)
`-o`   | Ordinal    | Ordinal numbers (1st, 2nd, etc.)
`-r`   | Roman      | Roman numerals (IV, LXI, etc.)
`-n`   | Number     | Other words containing numbers
`-a`   | Acronym    | Acronyms / initialisms (ALL-CAPS)
`-p`   | Proper     | Proper names / nouns
`-s`   | Symbol     | Symbols / letters
`-u`   | Unknown    | Unknown (no other kind)
`-A`   | All        | All kinds

### Dictionary

The `dict` sub-command lists words from the built-in dictionary.

```
> booky dict words

word words
word words wording worded
```

- With no options, all dictionary entries are listed
- Using `-f` lists all known word forms
