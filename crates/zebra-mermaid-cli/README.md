# zebra-mermaid-cli

Command line Mermaid-to-SVG renderer using the pure Rust `zebra-mermaid` library.
The renderer is aligned with **Pele 1.13.0** and supports 33 diagram types, built-in
RaTeX math, CSS output, and concrete colors for native SVG consumers.

```sh
cargo install zebra-mermaid-cli --version 0.1.0 --locked

zebra-mermaid render diagram.mmd -o diagram.svg
zebra-mermaid render diagram.mmd --theme dark -o dark.svg
zebra-mermaid render diagram.mmd --color-mode css -o web.svg
zebra-mermaid check diagram.mmd --format json
zebra-mermaid batch ./diagrams --out-dir ./svg --jobs 4
```

The installed executable is named `zebra-mermaid`. Use `-` for standard input;
output files are written atomically, and `--force` allows replacing an existing
file. Run `zebra-mermaid --help` for font, layout, resource-limit and link/image
options. Rust 1.96 or later is required. JavaScript and Node are not required.

The full regression suite and versioned reference fixtures are maintained in the
Git repository. Full original-test acceptance remains incomplete; see the
[implementation status](https://github.com/entropysniper-app/zebra-mermaid/blob/main/docs/IMPLEMENTATION-STATUS.md).

MIT licensed; see `LICENSE` and `THIRD_PARTY_NOTICES.md`.
[Source and rendering gallery](https://github.com/entropysniper-app/zebra-mermaid).
