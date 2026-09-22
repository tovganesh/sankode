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

## 📖 Language Tutorial & Documentation

Start learning Sankode in minutes with our comprehensive guide:
👉 **[Read the Complete Sankode Tutorial (सङ्कोड-बोधकः)](docs/TUTORIAL.md)**

It covers:
- **Phonetic Typing**: Write Devanagari using standard English keyboard transliteration (`sankode-ime`).
- **Syntax**: Pure Devanagari numerals (`०`-`९`), zero curly braces (`{ }`), statement Dandas (`।`).
- **Data Types & Variables**: `पूर्ण६४`, `अंश६४`, `सूत्र`, `द्वैध`, `रिक्त`, immutable `मान` vs mutable `विकार्य`.
- **Functions & Control Flow**: `क्रिया`, `प्रति`, `यदि` / `अन्यथा`, and `यावत्` loops.
- **Structures & Methods**: `संरचना`, `विधान`, `स्व`, and `चलऋण स्व`.
- **Affine Ownership**: Rust-like move semantics, immutable borrowing (`ऋण`), and mutable borrowing (`चलऋण`).
- **Dual Runtime & Studio**: Compiled `sankode`, dynamic `sanskipt` REPL, and web `sankode-studio`.

---

## 🚀 Quick Start

### Building from Source

Ensure you have Rust installed (1.80+):

```bash
# Build the compiler and runtime
cargo build --release

# Run the Sankode compiled hello world example
cargo run -p sankode-cli -- run examples/नमस्ते_जगत्.सङ्

# Run the ReLU Neural Network Language Model (MLP with non-linear activation)
cargo run -p sankode-cli -- run examples/शाकुन्तल_ऋजु_प्रतिरूप.सङ्

# Run the full-text Abhijnanasakuntalam LLM (file corpus, save/load weights, inference)
cargo run -p sankode-cli -- run examples/शाकुन्तल_सम्पूर्ण_प्रतिरूप.सङ्

# Run the Python comparative equivalents
python examples/comparative/shakuntala_relu_llm.py
python examples/comparative/shakuntala_llm.py

# Run the C comparative equivalent (MSVC on Windows / GCC on Linux)
cl /utf-8 /O2 examples/comparative/shakuntala_llm.c /Fe:examples/comparative/shakuntala_llm.exe
./examples/comparative/shakuntala_llm.exe

# Run the basic Abhijnanasakuntalam LLM example (4 verses)
cargo run -p sankode-cli -- run examples/शाकुन्तल_भाषा_प्रतिरूप.सङ्

# Run a Python-style Sanskipt script without main boilerplate
cargo run -p sanskipt -- examples/गणना_लिपि.सङ्स्कृ

# Launch the Sanskipt interactive REPL
cargo run -p sanskipt

# Launch the Sankode Studio IDE (वेधशाला) in your browser
cargo run -p sankode-studio
# OR via CLI: cargo run -p sankode-cli -- studio
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
│   ├── sankode-cli/        # Sankode CLI (`run`, `check`, `repl`, `studio`)
│   ├── sankode-ime/        # Phonetic transliteration engine (Roman to Devanagari)
│   ├── sanskipt/           # Python-inspired dynamic scripting runtime & REPL
│   └── sankode-studio/     # Embedded web-based visual IDE & dev server
├── docs/
│   ├── TUTORIAL.md             # Complete language tutorial & feature reference
│   └── implementation_plan.md  # Detailed language specification & design
└── examples/
    ├── नमस्ते_जगत्.सङ्     # Hello World in pure Devanagari
    ├── फिबोनाची.सङ्         # Recursive Fibonacci in pure Devanagari
    ├── comparative/             # Comparative LLM implementations
    │   ├── README.md            # Paradigm comparison matrix (Sankode vs Python vs C)
    │   ├── shakuntala_llm.py    # Type-annotated Python Bigram equivalent
    │   ├── shakuntala_llm.c     # Standalone C99/C11 Bigram equivalent
    │   ├── shakuntala_relu_llm.py # Python Multi-Layer Perceptron with ReLU
    │   └── shakuntala_relu_llm.c  # C99/C11 Multi-Layer Perceptron with ReLU
    ├── स्वामित्व_प्रदर्शनम्.सङ् # Ownership & borrowing demonstration
    ├── शाकुन्तल_भाषा_प्रतिरूप.सङ् # Basic Autoregressive LLM (4 verses)
    ├── शाकुन्तल_सम्पूर्ण_प्रतिरूप.सङ् # Full-text LLM: File I/O, Save/Load Weights & Inference
    ├── शाकुन्तल_ऋजु_प्रतिरूप.सङ् # ReLU Neural Language Model (MLP with non-linear activation)
    ├── अभिज्ञानशाकुन्तलम्_मूलम्.पाठ # Authentic Sanskrit corpus (Acts 1-7, 3,391 chars)
    ├── शाकुन्तल_प्रतिरूप.भार # Serialized Bigram model weights
    ├── शाकुन्तल_ऋजु_प्रतिरूप.भार # Serialized ReLU Neural model weights (W1, b1, W2, b2)
    └── गणना_लिपि.सङ्स्कृ    # Python-style top-level Sanskipt script
```

---

## 📄 License

Licensed under the [MIT License](LICENSE).
