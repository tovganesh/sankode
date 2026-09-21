# सङ्कोड (Sankode)

> **A Pure Devanagari-Native Systems Programming Language, Memory-Safe Compiler, Scripting Runtime, and Bespoke IDE.**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Build](https://img.shields.io/badge/status-active%20development-orange.svg)]()

---

## 🌟 Vision

**Sankode** (सङ्कोड) is a compiled, strongly typed, memory-safe programming language where the syntax, keywords, identifiers, and literals are authored entirely in **Devanagari script**. 

- **Pure Devanagari**: Keywords (`क्रिया`, `मान`, `यदि`, `इति`), identifiers, and numerals (`०`, `१`, `२`...`९`).
- **No Curly Braces (`{ }`)**: Blocks are delimited using classical Sanskrit markers (`इति`) and statements with Danda (`।`).
- **Rust-like Memory Safety**: Compile-time affine ownership (`स्वामित्व`), borrowing (`ऋण`), and lifetimes (`आयुः`) with zero runtime garbage-collection pauses.
- **Dual Runtime**:
  - `sankode`: Native AOT compiled binary toolchain.
  - `sanskipt`: Dynamic Python-like scripting interpreter and interactive REPL (`सङ्वादक`).
- **Sankode Studio**: Minimalist IDE with integrated phonetic typing (Roman to Devanagari on the fly).

---

## 📜 नमस्ते जगत् (Hello World Example)

```sankode
॥ नमस्ते जगत् - सङ्कोडस्य प्रथमं कार्यक्रमम् ॥

क्रिया मुख्य() -> रिक्त
    मुद्रय("नमस्ते जगत्!")।
    मान गणना = १०।
    मुद्रय("गणना = ", गणना)।
इति
```

### Explanation:
- `क्रिया` (*Kriyā*): Declares a function.
- `मुख्य` (*Mukhya*): The main entry point.
- `-> रिक्त` (*Rikta*): Returns unit / void.
- `मुद्रय` (*Mudraya*): Built-in print procedure.
- `।` (*Purna Virama* / Danda): Statement terminator (no semicolons `;`).
- `इति` (*Iti*): Closes the function block (no curly braces `{ }`).
- `१०`: Devanagari numeral for `10`.

---

## 🚀 Quick Start

### Building from Source

Ensure you have Rust installed (1.80+):

```bash
# Build the compiler and runtime
cargo build --release

# Run the Sankode compiled hello world example
cargo run -p sankode-cli -- run examples/नमस्ते_जगत्.सङ्

# Run a Python-style Sanskipt script without main boilerplate
cargo run -p sanskipt -- examples/गणना_लिपि.सङ्स्कृ

# Launch the Sanskipt interactive REPL
cargo run -p sanskipt
```

---

## 📂 Project Architecture

```text
sankode/
├── crates/
│   ├── sankode-core/       # Devanagari numerals, tokens, AST, span tracking
│   ├── sankode-lexer/      # UTF-8 Devanagari tokenizer & Unicode NFKC normalization
│   ├── sankode-parser/     # Brace-free recursive descent & Pratt parser
│   ├── sankode-semantics/  # Static type checker & immutability analysis
│   ├── sankode-borrowck/   # Affine ownership, move semantics & borrow checker
│   ├── sankode-eval/       # Execution engine & tree-walk runtime
│   ├── sankode-cli/        # Sankode CLI (`sankode run`, `sankode check`, `sankode repl`)
│   ├── sankode-ime/        # Phonetic transliteration engine (Roman to Devanagari)
│   └── sanskipt/           # Python-inspired dynamic scripting runtime & REPL
├── docs/
│   └── implementation_plan.md  # Detailed language specification & design
└── examples/
    ├── नमस्ते_जगत्.सङ्     # Hello World in pure Devanagari
    ├── फिबोनाची.सङ्         # Recursive Fibonacci in pure Devanagari
    ├── स्वामित्व_प्रदर्शनम्.सङ् # Ownership & borrowing demonstration
    └── गणना_लिपि.सङ्स्कृ    # Python-style top-level Sanskipt script
```

---

## 📄 License

Licensed under the [MIT License](LICENSE).
