A tool to analyze English text

Installation:
```shell
cargo install booky
```

### Commands

> booky word _{word}_

Show entries for _{word}_.

The `freq` command (and others) read UTF-8 text from `stdin`, which can be
redirected from a file:

> booky freq < _{file}_

Count frequencies of words in _{file}_.

- `acronym`: List acronyms / initialisms
- `foreign`: List foreign words (non-English)
- `num`: List words containing numbers
- `ordinal`: List ordinal numbers (1st, 2nd, etc.)
- `proper`: List proper nouns (best guess)
- `roman`: List roman numerals
- `unknown`: List unknown words
