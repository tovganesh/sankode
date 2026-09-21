# सङ्कोड (Sankode)

> **A Pure Devanagari-Native Systems Programming Language, Memory-Safe Compiler, Scripting Runtime, and Bespoke IDE.**

[![License](https://img.shields.io/badge/License-MIT%2FApache--2.0-blue.svg)](LICENSE)
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

# Run the hello world example
cargo run -p sankode-cli -- run examples/नमस्ते_जगत्.सङ्
```

---

## 📂 Project Architecture

```text
sankode/
├── crates/
│   ├── sankode-core/       # Devanagari numerals, tokens, AST, span tracking
│   ├── sankode-lexer/      # UTF-8 Devanagari tokenizer & Unicode NFKC normalization
│   ├── sankode-parser/     # Brace-free recursive descent & Pratt parser
│   ├── sankode-eval/       # Interactive interpreter & tree-walk evaluator
│   └── sankode-cli/        # CLI executable (`sankode run`, `sankode repl`)
├── docs/
│   └── implementation_plan.md  # Detailed language specification & design
└── examples/
    └── नमस्ते_जगत्.सङ्         # Hello World in pure Devanagari
```

---

## 📄 License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.
