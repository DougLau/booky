## Word Entries

The dictionary `english.csv` is a special CSV format with extra rules:

* Value 1: **base word** : **word class** {.**attributes**}
* Values 2+: **irregular forms**

### Base Word

The base form of a word, usually in all lowercase.

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
|------|----------------------------------|---------
| `a`  | Auxiliary verb                   | _cannot_
| `c`  | Countable noun (default)         | _chair_
| `u`  | Uncountable noun (mass)          | _furniture_
| `g`  | Group (collective noun)          | _water_
| `n`  | Proper (name) noun               | _Monday_
| `p`  | Plurale Tantum                   | _pants_, _scissors_
| `i`  | Intransitive verb or preposition |
| `t`  | Transitive verb or preposition   |
| `j`  | Conjunctive preposition          |
| `v`  | "Deverbal" prepositions          | _excluding_

### Forms

If only the base form is provided (no irregular), regular forms will be
automatically generated for these four word classes:

- **Adjective**: Comparative (\_*er*), Superlative (\_*est*)
- **Noun**: Plural (\_*s*)
- **Pronoun**: Plural (\_*s*)
- **Verb**: Present tense (\_*s*), Present participle (\_*ing*),
            Past tense (\_*ed*)

There are rules for attaching the suffixes, but they're not perfect.
