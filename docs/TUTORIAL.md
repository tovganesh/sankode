# ॥ सङ्कोड-बोधकः ॥ (Sankode Tutorial)

> **A Comprehensive Guide to Learning and Using the Sankode Programming Language**

Welcome to **Sankode** (सङ्कोड) — a systems and scripting programming language designed from first principles with **classical Sanskrit** keywords, pure **Devanagari numerals** (`०`-`९`), a **brace-free syntax** (no `{ }`), and a **Rust-inspired affine ownership memory model**.

---

## विषयसूची (Table of Contents)

1. [दर्शनम् (Philosophy & Core Invariants)](#१-दर्शनम्-philosophy--core-invariants)
2. [लेखन-पद्धतिः (How to Write: Phonetic Input & IME)](#२-लेखन-पद्धतिः-how-to-write-phonetic-input--ime)
3. [मूलभूताः नियमाः (Syntax Fundamentals)](#३-मूलभूताः-नियमाः-syntax-fundamentals)
4. [दत्तप्रकाराः (Data Types)](#४-दत्तप्रकाराः-data-types)
5. [चराः विकार्यता च (Variables & Mutability)](#५-चराः-विकार्यता-च-variables--mutability)
6. [क्रियाः (Functions)](#६-क्रियाः-functions)
7. [नियन्त्रण-प्रवाहः (Control Flow: If & Loops)](#७-नियन्त्रण-प्रवाहः-control-flow-if--loops)
8. [संरचना विधानं च (Structs & Methods)](#८-संरचना-विधानं-च-structs--methods)
9. [स्वामित्वम् ऋणं च (Ownership & Borrowing)](#९-स्वामित्वम्-ऋणं-च-ownership--borrowing)
10. [साधनानि (Tooling: Sankode, Sanskipt & Studio)](#१०-साधनानि-tooling-sankode-sanskipt--studio)
11. [व्यावहारिक-उदाहरणानि (Practical Examples)](#११-व्यावहारिक-उदाहरणानि-practical-examples)

---

## १. दर्शनम् (Philosophy & Core Invariants)

Sankode is built on four non-negotiable architectural principles:

| सिद्धान्तः (Principle) | विवरणम् (Description) |
|---|---|
| **शुद्ध-देवनागरी (Pure Devanagari)** | All keywords, identifiers, string literals, and numerals (`०`, `१`, `२`...`९`) are in Devanagari. ASCII digits (`0-9`) are rejected by the compiler. |
| **बन्धनाभावः (Zero Braces `{ }`)** | Blocks never use curly braces `{ }`. Blocks begin with declarations and close deterministically with `इति` (*iti*). |
| **दण्ड-विरामः (Danda Termination)** | Statements terminate with a single Sanskrit Danda `।` (Purna Virama). Semicolons (`;`) are forbidden. Comments are wrapped with double Dandas `॥ ... ॥`. |
| **स्मृति-सुरक्षा (Affine Memory Safety)** | Variables have strict ownership semantics. Value passing moves ownership unless explicitly borrowed via `ऋण` (*rina* - immutable) or `चलऋण` (*chalarina* - mutable). |

---

## २. लेखन-पद्धतिः (How to Write: Phonetic Input & IME)

You can write Sankode using any standard Devanagari keyboard (InScript or Google Input Tools). Additionally, Sankode includes **`sankode-ime`**, a high-performance phonetic transliterator based on ITRANS:

### Quick Transliteration Reference:

| Roman Input | Devanagari Output | Meaning / Role |
|---|---|---|
| `kriya` | `क्रिया` | Function declaration |
| `man` | `मान` | Variable declaration |
| `vikarya` | `विकार्य` | Mutable modifier |
| `yadi` | `यदि` | If branch |
| `anyathayadi` | `अन्यथायदि` | Else-if branch |
| `anyatha` | `अन्यथा` | Else branch |
| `yavat` | `यावत्` | While loop |
| `prati` | `प्रति` | Return statement |
| `iti` | `इति` | Block terminator |
| `mudraya` | `मुद्रय` | Standard print statement |
| `samrachana` | `संरचना` | Struct declaration |
| `vidhana` | `विधान` | Impl block |
| `sva` | `स्व` | Self (`this` / `self`) |
| `rina` | `ऋण` | Immutable borrow (`&`) |
| `chalarina` | `चलऋण` | Mutable borrow (`&mut`) |
| `|` | `।` | Statement terminator |
| `||` | `॥` | Comment delimiter |
| `0 1 2 ... 9` | `० १ २ ... ९` | Devanagari digits |

### Using Phonetic Typing in Tools:
- **Sanskipt CLI**: Pass `--ime` to automatically transliterate source files on the fly:
  ```bash
  cargo run -p sanskipt -- --ime script_in_roman.txt
  ```
- **Sanskipt REPL**: Type `:ime` inside the REPL to toggle live Roman-to-Devanagari typing.
- **Sankode Studio**: Click the **Phonetic IME (सक्रिय)** toggle in the top bar to type phonetic Roman and have it converted in real time.

---

## ३. मूलभूतताः नियमाः (Syntax Fundamentals)

### 3.1 Statements and Danda (`।`)
Every executable statement must end with a single Danda `।`:
```sankode
मान वयः = २५।
मुद्रय(वयः)।
```

### 3.2 Comments (`॥ ... ॥`)
Comments are delimited by double Dandas `॥`:
```sankode
॥ एषः सङ्केते टिप्पणी अस्ति ॥
मान क = १०। ॥ अत्र क-चरस्य मानं १० अस्ति ॥
```

### 3.3 Blocks and `इति`
Code blocks do not use `{ }`. They are closed with `इति`:
```sankode
क्रिया मुख्य() -> रिक्त
    मुद्रय("आरम्भः")।
इति
```

---

## ४. दत्तप्रकाराः (Data Types)

Sankode is strongly and statically typed:

| प्रकारः (Type) | विवरणम् (Description) | उदाहरणम् (Example) |
|---|---|---|
| `पूर्ण६४` | 64-bit signed integer | `४२`, `-१००`, `०` |
| `अंश६४` | 64-bit IEEE-754 floating point | `३.१४१५९`, `-०.५`, `१०.०` |
| `सूत्र` | UTF-8 String literal | `"नमस्ते जगत्"`, `"सङ्कोडः"` |
| `द्वैध` | Boolean (`सत्यम्` or `मिथ्या`) | `सत्यम्` (true), `मिथ्या` (false) |
| `रिक्त` | Unit type (equivalent to Rust `()`) | Return type of void functions |

---

## ५. चराः विकार्यता च (Variables & Mutability)

By default, all variable bindings in Sankode are **अविकार्य (immutable)**.

### 5.1 Immutable Variables (`मान`)
```sankode
मान क = १०।
॥ क = २०। <-- प्रकारदोषः! अविकार्य-चरस्य मानं परिवर्तयितुं न शक्यते ॥
```

### 5.2 Mutable Variables (`मान विकार्य`)
To allow a variable to be modified, declare it with `विकार्य`:
```sankode
मान विकार्य गणना = ०।
गणना = गणना + १।
मुद्रय("गणना = ", गणना)।
```

---

## ६. क्रियाः (Functions)

Functions are declared with `क्रिया`, take typed parameters, specify return types with `->`, and end with `इति`. Values are returned using `प्रति`.

```sankode
क्रिया योजय(क: पूर्ण६४, ख: पूर्ण६४) -> पूर्ण६४
    प्रति क + ख।
इति

क्रिया मुख्य() -> रिक्त
    मान परिणाम = योजय(१५, २५)।
    मुद्रय("योगः = ", परिणाम)। ॥ योगः = ४० ॥
इति
```

If a function does not return a value, its return type is `रिक्त`:
```sankode
क्रिया अभिवादनम्(नाम: सूत्र) -> रिक्त
    मुद्रय("शुभदिनम्, ", नाम)।
इति
```

---

## ७. नियन्त्रण-प्रवाहः (Control Flow: If & Loops)

### 7.1 सशर्त-शाखा (Conditionals: `यदि` / `अन्यथायदि` / `अन्यथा`)
```sankode
मान अङ्क = ७५।

यदि अङ्क >= ९०
    मुद्रय("उत्कृष्टम् (A)")।
अन्यथायदि अङ्क >= ७०
    मुद्रय("प्रथम-श्रेणी (B)")।
अन्यथा
    मुद्रय("सामान्यम्")।
इति
```

### 7.2 चक्रम् (Loops: `यावत्`)
The `यावत्` construct loops as long as the condition evaluates to `सत्यम्`:
```sankode
मान विकार्य सूचक = १।

यावत् सूचक <= ५
    मुद्रय("सङ्ख्या = ", सूचक)।
    सूचक = सूचक + १।
इति
```

---

## ८. संरचना विधानं च (Structs & Methods)

Sankode supports user-defined data structures (`संरचना`) and associated method blocks (`विधान`).

### 8.1 Defining a Struct (`संरचना`)
```sankode
संरचना बिन्दु
    क्ष: अंश६४।
    य: अंश६४।
इति
```

### 8.2 Implementing Methods (`विधान`)
Methods can take immutable `स्व` (self) or mutable `चलऋण स्व` (mutable reference to self):
```sankode
विधान बिन्दु
    ॥ Getter / inspection method ॥
    क्रिया दूरता(स्व) -> अंश६४
        प्रति स्व.क्ष + स्व.य।
    इति

    ॥ Mutating method ॥
    क्रिया स्थानान्तरय(चलऋण स्व, अक्ष: अंश६४, अय: अंश६४) -> रिक्त
        स्व.क्ष = स्व.क्ष + अक्ष।
        स्व.य = स्व.य + अय।
    इति
इति
```

### 8.3 Instantiation and Method Calls
Fields are initialized using named fields (`क्षेत्र: मान`):
```sankode
क्रिया मुख्य() -> रिक्त
    मान विकार्य ब = बिन्दु(क्ष: ३.०, य: ४.०)।
    मुद्रय("प्रारम्भिक-दूरता = ", ब.दूरता())।

    ब.स्थानान्तरय(२.०, १.०)।
    मुद्रय("नूतन-स्थानम् = ", ब)।
इति
```

---

## ९. स्वामित्वम् ऋणं च (Ownership & Borrowing)

Sankode incorporates Rust's affine type system to guarantee memory safety at compile time without a garbage collector:

### 9.1 Move Semantics (स्वामित्व-हस्तान्तरणम्)
Assigning a non-copy value or passing it to another function transfers ownership:
```sankode
क्रिया उपभोक्ता(दत्त: बिन्दु) -> रिक्त
    मुद्रय("दत्तं गृहीतम्: ", दत्त)।
इति

क्रिया मुख्य() -> रिक्त
    मान ब१ = बिन्दु(क्ष: १.०, य: २.०)।
    उपभोक्ता(ब१)।
    ॥ उपभोक्ता(ब१)। <-- स्वामित्वदोषः! 'ब१' चरस्य स्वामित्वं पूर्वमेव हस्तान्तरितम् ॥
इति
```

### 9.2 Borrowing (ऋणग्रहणम्)
Instead of moving ownership, you can borrow values:
- `ऋण चर`: Read-only, immutable borrow.
- `चलऋण चर`: Read-write, mutable borrow.

**Borrow Rules**:
1. You may have any number of immutable references (`ऋण`).
2. OR you may have exactly one mutable reference (`चलऋण`).
3. You can never have both simultaneously!

```sankode
क्रिया मुख्य() -> रिक्त
    मान विकार्य धनम् = १०००।
    मान साक्षी = ऋण धनम्।
    मुद्रय("दृष्टं धनम् = ", साक्षी)।
इति
```

---

## १०. साधनानि (Tooling: Sankode, Sanskipt & Studio)

The Sankode repository provides three execution modes:

### 10.1 The Native Compiler Toolchain (`sankode`)
Validates types, checks ownership, and executes compiled programs:
```bash
# Verify static types and borrowing rules without running
cargo run -p sankode-cli -- check examples/नमस्ते_जगत्.सङ्

# Run program
cargo run -p sankode-cli -- run examples/नमस्ते_जगत्.सङ्
```

### 10.2 The Dynamic Scripting Engine (`sanskipt`)
A Python-inspired interpreter that executes top-level code directly:
```bash
# Execute a script
cargo run -p sanskipt -- examples/गणना_लिपि.सङ्स्कृ

# Execute a one-liner command
cargo run -p sanskipt -- -c 'मान क = ५। मुद्रय(क * २)।'

# Interactive REPL shell
cargo run -p sanskipt
```

Inside the REPL:
- Type `:ime` to switch between English keyboard phonetic typing and direct Devanagari.
- Type `विराम` or `exit` to exit.

### 10.3 Sankode Studio IDE (`sankode-studio`)
A lightweight, modern web IDE with built-in runtime engines:
```bash
cargo run -p sankode-studio
# Alternatively:
cargo run -p sankode-cli -- studio
```
Features:
- **Dual Execution**: Run in either compiled `sankode` mode (with strict type & borrow checking) or rapid `sanskipt` mode.
- **Deep Diagnostics**: Inspect the live Token stream and AST tree for any program.
- **Phonetic Transliteration**: Type `kriya mukhya() |` and see it transform into `क्रिया मुख्य() ।` in real time.

---

## ११. व्यावहारिक-उदाहरणानि (Practical Examples)

### उदाहरणम् १: नमस्ते जगत् (Hello World)
```sankode
॥ संचिका: नमस्ते_जगत्.सङ् ॥

क्रिया मुख्य() -> रिक्त
    मुद्रय("नमस्ते जगत्!")।
    मान गणना = १०।
    मुद्रय("गणना = ", गणना)।
इति
```

### उदाहरणम् २: फिबोनाची-अनुक्रमः (Recursive Fibonacci)
```sankode
॥ संचिका: फिबोनाची.सङ् ॥

क्रिया फिबोनाची(संख्या: पूर्ण६४) -> पूर्ण६४
    यदि संख्या <= ०
        प्रति ०।
    इति
    यदि संख्या == १
        प्रति १।
    इति
    प्रति फिबोनाची(संख्या - १) + फिबोनाची(संख्या - २)।
इति

क्रिया मुख्य() -> रिक्त
    मुद्रय("--- फिबोनाची-श्रेणी ---")।
    मान विकार्य न = ०।
    यावत् न <= ७
        मुद्रय("फिबोनाची(", न, ") = ", फिबोनाची(न))।
        न = न + १।
    इति
इति
```

### उदाहरणम् ३: बिन्दु-संरचना एवं स्थानान्तरणम् (Vector / Point Math)
```sankode
॥ संचिका: बिन्दु_संरचना.सङ् ॥

संरचना बिन्दु
    क्ष: अंश६४।
    य: अंश६४।
इति

विधान बिन्दु
    क्रिया दूरता(स्व) -> अंश६४
        प्रति स्व.क्ष + स्व.य।
    इति

    क्रिया स्थानान्तरय(चलऋण स्व, अक्ष: अंश६४, अय: अंश६४) -> रिक्त
        स्व.क्ष = स्व.क्ष + अक्ष।
        स्व.य = स्व.य + अय।
    इति
इति

क्रिया मुख्य() -> रिक्त
    मान विकार्य ब = बिन्दु(क्ष: ३.५, य: ४.५)।
    मुद्रय("प्रारम्भिकः बिन्दुः = ", ब)।
    ब.स्थानान्तरय(१.५, २.०)।
    मुद्रय("नूतन-बिन्दुः = ", ब)।
    मुद्रय("नूतना दूरता = ", ब.दूरता())।
इति
```

---

## 🎓 Next Steps & Community

To contribute, report issues, or explore the source code:
- **Repository**: [github.com/tovganesh/sankode](https://github.com/tovganesh/sankode)
- **License**: MIT License
- **Issues & Discussions**: Open a GitHub Issue or Pull Request.
