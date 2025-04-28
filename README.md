A tool to analyze English text

Installation:
```shell
cargo install booky
```

### Commands

> booky word _{word}_

Show entries for _{word}_.

The `freq`, `proper` and `unknown` commands read UTF-8 text from `stdin`,
which can be redirected from a file:

> booky freq < _{file}_

Count frequencies of words in _{file}_.

> booky proper < _{file}_

List proper nouns in _{file}_ (best guess).

> booky unknown < _{file}_

List unknown words in _{file}_.
