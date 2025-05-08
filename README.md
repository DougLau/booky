A tool to analyze English text

Installation:
```shell
cargo install booky
```

### Categorizing Words

The `cat` sub-command reads UTF-8 text from `stdin`, which can be redirected
from a file.  With no additional options, a summary of words in each category
is listed:

```
> booky cat < _{file}_

14907 d Dictionary
    9 o Ordinal
   11 r Roman
    0 n Number
   31 a Acronym
    9 f Foreign
  938 p Proper
    9 l Letter
  928 u Unknown
```

Command-line options can be added to list all words for a category.

Option | Category   | Description
-------|------------|-----------------------------
d      | Dictionary | Found in built-in dictionary
o      | Ordinal    | Ordinal numbers (1st, 2nd, etc.)
r      | Roman      | Roman numerals (IV, LXI, etc.)
n      | Number     | Other words containing numbers
a      | Acronym    | Acronyms / initialisms (ALL-CAPS)
f      | Foreign    | Foreign words (non-English)
p      | Proper     | Proper names / nouns
l      | Letter     | Single-letter "words"
u      | Unknown    | Unknown (no other category)
A      | all        | All categories

### Dictionary

The `dict` sub-command lists words from the built-in dictionary.

```
> booky dict words

word words
word words wording worded
```

With no options, all dictionary entries are listed.  Using `-f` lists all
known word forms.
