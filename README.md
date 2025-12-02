# Typst2Latex 

Convert Typst documents written using the unequivocal-ams template to human-friendly Latex. This uses a custom Typst parser to produce a parse tree in which useful environements (e.g. theorem) are preserved. A backend (currently pandoc only) is then used to convert the leafs of the parse tree (typically math formulas). This relies on certain assumptions about the structure of your Typst code, see [Assumptions](#assumptions).

## Features

- Preserves figure and theorem-like environnements.
- Distinguish between references and citations using an optionnal bibtex file.
- Special directives for ignoring or adding code during conversion:
```typst
\\ BEGIN NO TEX
Typst content to ignore
\\ END NO TEX

\* BEGIN TEX
Latex content to add 
END TEX */
```
And much more to come!

## Roadmap

TODO

## Installation 

Installation is done using cargo: 

```bash
cargo install --git https://github.com/aualbert/typst2latex
```

Nix users can also install or run the program directly:

```bash
nix run github.com/aualbert/typst2latex
```

## Running

Typst2Latex requires Pandoc to be installed. See the list of options provided by the program. Typically:

```bash
typst2latex main.typ -b refs.bib
```

## Building

Building is done using cargo:

```bash
git clone https://github.com/aualbert/typst2latex
cd typst2latex
cargo build
```

For Nix users, a flake is provided. Activate it using `nix develop`.

## Assumptions

- Theorem-like environnement can be given a title using the following syntax:
```typst
\#theorem[my_title⏎
my_content
]
```

- Body arguments in functions should use brackets as much as possible, e.g. `\#figure([#grid ...])` instead of `\#figure(grid ...)`. 
