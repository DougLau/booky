## Word Entries

The dictionary `english.csv` is a special CSV format with extra rules:

* Value 1: **base word** : **word class** {.**attributes**}
* Values 2+: **irregular forms**

### Base Word

The base form of a word, usually in all lowercase.

### Word Class

This is one of nine basic word classes:

- __A__ Adjective
- __Av__ Adverb
- __C__ Conjunction
- __D__ Determiner
- __I__ Interjection
- __N__ Noun
- __P__ Preposition
- __Pn__ Pronoun
- __V__ Verb

It may be followed by a dot and **attributes**.

### Forms

If only the base form is provided (no irregular), regular forms will be
automatically generated for these four word classes:

- **Adjective**: Comparative (_er), Superlative (_est)
- **Noun**: Plural (_s)
- **Pronoun** Plural (_s)
- **Verb**: Present tense (_s), Present participle (_ing), Past tense (_ed)

There are rules for attaching the suffixes, but they're not perfect.
