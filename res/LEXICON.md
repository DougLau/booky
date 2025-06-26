## Lexicon Entries

The lexicon `english.csv` uses a variation of the comma separated value (CSV)
format.  Each line contains one **lexeme**:

* Value 1: **lemma** : **word class** {.**attributes**}
* Values 2+: **irregular inflected forms**

Example:

```
go:V,goes,going,went,gone
```

### Lemma (Base Word)

The base or canonical form of a word, usually in all lowercase.

### Word Class

Codes for one of nine basic word classes:

| Code | Class        |
|------|--------------|
| `A`  | Adjective    |
| `Av` | Adverb       |
| `C`  | Conjunction  |
| `D`  | Determiner   |
| `I`  | Interjection |
| `N`  | Noun         |
| `P`  | Preposition  |
| `Pn` | Pronoun      |
| `V`  | Verb         |

### Attributes

If any attributes are provided, they are preceded by a dot `.`:

| Code | Description                      | Examples
|------|----------------------------------|----------------------
| `s`  | Singulare Tantum                 | _dust_, _information_
| `p`  | Plurale Tantum                   | _pants_, _scissors_
| `n`  | Proper (name) noun               | _Monday_
| `a`  | Auxiliary verb                   | _cannot_
| `i`  | Intransitive verb or preposition |
| `t`  | Transitive verb or preposition   |

### Inflected Forms

Forms are modifications of the **lemma**, indicating tense, number, etc.
These forms can be **regular** or **irregular**.  If no irregular forms are
provided, regular forms will be automatically generated for these four word
classes:

- **Adjective**: Comparative (*er*), Superlative (*est*)
- **Noun**: Plural (*s*)
- **Pronoun**: Plural (*s*)
- **Verb**: Present tense (*s*), Present participle (*ing*),
            Past tense (*ed*)

The suffixes are attached to lemmas based on special rules.

Irregular forms can be abbreviated with a hyphen (\-) at the beginning.  This
includes the lemma, trimmed to the first letter after the hyphen.
For example:

```
alumnus:N,-ni
```
