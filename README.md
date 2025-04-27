A tool to analyze English text

Installation:
```shell
cargo install booky
```

Commands

- `booky word`: show entries for a word

The `freq`, `proper` and `unknown` commands read from `stdin`, which can
be redirected from a file:

- `booky freq < [file]`: count word frequencies
- `booky proper < [file]`: list proper nouns (best guess)
- `booky unknown < [file]`: list unknown words
