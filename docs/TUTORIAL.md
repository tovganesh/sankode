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
10. [साधनानि (Tooling: Sankode, Sanskript & Studio)](#१०-साधनानि-tooling-sankode-sanskript--studio)
11. [व्यावहारिक-उदाहरणानि (Practical Examples)](#११-व्यावहारिक-उदाहरणानि-practical-examples)
12. [उन्नत-विषयाः: सूचयः, गणितम्, बृहद्-भाषा-प्रतिरूपम् (Advanced: Lists, Math & LLMs)](#१२-उन्नत-विषयाः-सूचयः-गणितम्-बृहद्-भाषा-प्रतिरूपम्-advanced-lists-math--llms)

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
- **Sanskript CLI**: Pass `--ime` to automatically transliterate source files on the fly:
  ```bash
  cargo run -p sanskript -- --ime script_in_roman.txt
  ```
- **Sanskript REPL**: Type `:ime` inside the REPL to toggle live Roman-to-Devanagari typing.
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

## १०. साधनानि (Tooling: Sankode, Sanskript & Studio)

The Sankode repository provides three execution modes:

### 10.1 The Native Compiler Toolchain (`sankode`)
Validates types, checks ownership, and compiles/executes programs at native machine speed via the Ahead-Of-Time (AOT) C99 backend:
```bash
# Verify static types and borrowing rules without running
cargo run -p sankode-cli -- check examples/नमस्ते_जगत्.सङ्

# Build an optimized native executable (10x-50x faster than Python):
cargo run -p sankode-cli -- build examples/शाकुन्तल_ऋजु_प्रतिरूप.सङ् -o target/shakuntala_relu.exe
./target/shakuntala_relu.exe

# Run with on-the-fly AOT native compilation:
cargo run -p sankode-cli -- run --native examples/शाकुन्तल_ऋजु_प्रतिरूप.सङ्

# Emit clean, standalone C99 source code:
cargo run -p sankode-cli -- build examples/शाकुन्तल_ऋजु_प्रतिरूप.सङ् --emit-c

# Run via interpreter:
cargo run -p sankode-cli -- run examples/नमस्ते_जगत्.सङ्
```

### 10.2 The Dynamic Scripting Engine (`sanskript`)
A Python-inspired interpreter that executes top-level code directly:
```bash
# Execute a script
cargo run -p sanskript -- examples/गणना_लिपि.सङ्स्कृ

# Execute a one-liner command
cargo run -p sanskript -- -c 'मान क = ५। मुद्रय(क * २)।'

# Interactive REPL shell
cargo run -p sanskript
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
- **Dual Execution**: Run in either compiled `sankode` mode (with strict type & borrow checking) or rapid `sanskript` mode.
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

---

## १२. उन्नत-विषयाः: सूचयः, गणितम्, बृहद्-भाषा-प्रतिरूपम् (Advanced: Lists, Math & LLMs)

Sankode supports dynamic data structures and mathematical built-in functions suitable for tensor math, scientific computation, and machine learning models without sacrificing type safety or ownership invariants.

### १२.१ सूचयः (Lists & Arrays: `सूची` / `[T]`)

Sankode provides first-class dynamic lists with Devanagari bracket syntax:

```sankode
॥ सूची-निर्माणं सूचकीकरणं च ॥
मान सारणी = [१०, २०, ३०, ४०]।
मुद्रय("प्रथमः मानः = ", सारणी[०])।
सारणी[०] = ९९।
मुद्रय("परिवर्तितः मानः = ", सारणी[०])।
```

#### सूची-सम्बद्धाः क्रियाः (List Built-in Procedures)
- `सूची_सृज(आकार: पूर्ण६४, प्रारम्भिक_मान: T) -> सूची`: Allocates a list of given length filled with an initial element.
- `सूची_दैर्घ्यम्(सू: सूची) -> पूर्ण६४`: Returns the count of elements in the list.
- `सूची_संयोजय(सू: सूची, मान: T) -> रिक्त`: Appends an element to the end of the list.
- `सूची_संयोग(सू: सूची, विभाजक: सूत्र) -> सूत्र`: Joins elements of a list into a single delimited string.

```sankode
मान विकार्य आव्यूह = सूची_सृज(१०, ०.०)।
सूची_संयोजय(आव्यूह, ५.२)।
मुद्रय("दैर्घ्यम् = ", सूची_दैर्घ्यम्(आव्यूह))।
```

### १२.२ सूत्र-परीक्षणं संचिका-व्यापारश्च (String & File I/O Built-ins)
Strings in Sankode are UTF-8 sequences that can be measured, indexed, split, and persisted:
- `सूत्र_दैर्घ्यम्(वाक्य: सूत्र) -> पूर्ण६४`: Returns the Unicode character count of the string.
- `सूत्र_वर्ण(वाक्य: सूत्र, सूचक: पूर्ण६४) -> सूत्र`: Extracts the character at the specified index as a single-character string.
- `सूत्र_अंश(वाक्य: सूत्र, आरम्भ: पूर्ण६४, समाप्ति: पूर्ण६४) -> सूत्र`: Returns the substring slice from `आरम्भ` to `समाप्ति` (exclusive).
- `सूत्र_विभाजय(वाक्य: सूत्र, विभाजक: सूत्र) -> सूची`: Splits a string by delimiter into a list of strings (or characters if delimiter is empty).
- `संख्या_पाठ(वाक्य: सूत्र) -> अंश६४`: Parses a Devanagari or ASCII numeric string into a float.
- `सूत्र_रूप(मान: T) -> सूत्र`: Converts any value into its string representation.

#### तार्किक-गणितीय-सक्रियकाः (Mathematical & Neural Activation Built-ins)
- `रेलू(मान: अंश६४) -> अंश६४`: Rectified Linear Unit non-linear activation (returns $\max(0.0, \text{मान})$).
- `महत्तम(अ: अंश६४, ब: अंश६४) -> अंश६४`: Returns the maximum of two floating-point values.
- `न्यूनतम(अ: अंश६४, ब: अंश६४) -> अंश६४`: Returns the minimum of two floating-point values.

#### संचिका-क्रियाः (File I/O Procedures)
- `संचिका_पठ(मार्ग: सूत्र) -> सूत्र`: Reads the entire UTF-8 file contents into a string.
- `संचिका_लेख(मार्ग: सूत्र, विषय: सूत्र) -> रिक्त` (alias: `संचिका_लिख`): Writes text to a file, creating parent directories if needed.
- `संचिका_विद्यते(मार्ग: सूत्र) -> सत्यम्`: Checks whether a file exists on disk.

### १२.३ गणित-क्रियाः (Mathematical Built-in Procedures)
For loss calculation, probabilities, and normalizations:
- `लॉग(क्ष: अंश६४) -> अंश६४`: Natural logarithm (ln(x)).
- `घाताङ्क(क्ष: अंश६४) -> अंश६४`: Exponential function (exp(x)).
- `वर्गमूल(क्ष: अंश६४) -> अंश६४`: Square root (sqrt(x)).
- `पूर्णाङ्क(क्ष: अंश६४) -> पूर्ण६४`: Casts float (`अंश६४`) to integer (`पूर्ण६४`).
- `अंशाङ्क(क्ष: पूर्ण६४) -> अंश६४`: Casts integer (`पूर्ण६४`) to float (`अंश६४`).

### १२.४ बृहद्-भाषा-प्रतिरूपम् (Building an LLM from Scratch: Abhijnanasakuntalam)

In `examples/शाकुन्तल_भाषा_प्रतिरूप.सङ्`, Sankode implements a complete character-level autoregressive language model trained from scratch on Mahakavi Kalidasa's classical drama *Abhijnanasakuntalam* (*अभिज्ञानशाकुन्तलम्*):

1. **यादृच्छिक_यन्त्र (PRNG)**: A 64-bit Linear Congruential Generator producing pseudo-random floats in [0.0, 1.0).
2. **शब्दावली (Tokenizer & Vocabulary)**: Parses the corpus into unique characters, builds forward and reverse token maps.
3. **भाषा_प्रतिरूप (Autoregressive Language Model)**:
   - Allocates a dynamic transition matrix (2D tensor `सूची_सृज(आकार * आकार, ०.०)`).
   - Trains on bigram sequences with Laplace add-one smoothing.
   - Computes cross-entropy loss (negative log-likelihood) and Perplexity metric.
   - Generates novel Sanskrit verse tokens using temperature-controlled softmax probability sampling.

#### Running the Basic LLM (4 Verses):
```bash
cargo run -p sankode-cli -- run examples/शाकुन्तल_भाषा_प्रतिरूप.सङ्
```

### १२.५ सम्पूर्ण-शाकुन्तल-प्रतिरूपम् (Full Pipeline: File Corpus, Weight Persistence & Inference)

The advanced pipeline in `examples/शाकुन्तल_सम्पूर्ण_प्रतिरूप.सङ्` demonstrates production-style machine learning workflow:
1. **Corpus File**: Reads 3,391 characters of Kalidasa's *Abhijnanasakuntalam* from `examples/अभिज्ञानशाकुन्तलम्_मूलम्.पाठ` using `संचिका_पठ`.
2. **Model Training**: Trains autoregressive bigram transition weights over a 54-token vocabulary.
3. **Weight Persistence**: Serializes the trained float matrix using `सूची_संयोग` and saves weights to `examples/शाकुन्तल_प्रतिरूप.भार` using `संचिका_लेख`.
4. **Weight Deserialization**: Loads the model back from disk using `संचिका_पठ`, `सूत्र_विभाजय`, and `संख्या_पाठ`.
5. **Generative Inference**: Runs temperature-sampled text generation on prompts (`"शकुन्तला"`, `"या सृष्टिः"`, `"दुष्यन्तः"`).

#### Running the Full Pipeline:
```bash
# Run with compiled binary toolchain
cargo run -p sankode-cli -- run examples/शाकुन्तल_सम्पूर्ण_प्रतिरूप.सङ्

# Or run with dynamic Sanskript runner
cargo run -p sanskript -- examples/शाकुन्तल_सम्पूर्ण_प्रतिरूप.सङ्
```


### १२.७ ऋजु-भाषा-प्रतिरूपम् (ReLU Neural Network Language Model)

Extending beyond linear n-gram transitions, `examples/शाकुन्तल_ऋजु_प्रतिरूप.सङ्` implements a complete **Multi-Layer Perceptron (MLP)** character-level neural language model:

1. **Layer 1 Projection**: Multiplies the input character embedding by weight matrix `भार_१` ($H \times V$) and adds bias `पक्षपात_१` ($H$).
2. **ReLU Activation (`रेलू`)**: Applies the native `रेलू` built-in procedure to extract sparse, non-linear hidden representations.
3. **Layer 2 Logits**: Projects the hidden state $h$ to vocabulary logits using `भार_२` ($V \times H$) and bias `पक्षपात_२` ($V$).
4. **Softmax Temperature Scaling**: Computes normalized categorical probabilities with configurable sampling temperature.
5. **Loss & Perplexity**: Evaluates Negative Log-Likelihood cross-entropy loss (`२.५६५५`) and perplexity (`१३.००६९`).
6. **Multi-Section Weight Persistence**: Saves and restores model parameters to/from `examples/शाकुन्तल_ऋजु_प्रतिरूप.भार` using `===खण्ड===` delimiters.

```bash
# Verify static safety
cargo run -p sankode-cli -- check examples/शाकुन्तल_ऋजु_प्रतिरूप.सङ्

# Run with compiled toolchain
cargo run -p sankode-cli -- run examples/शाकुन्तल_ऋजु_प्रतिरूप.सङ्

# Run with Sanskript dynamic script runner
cargo run -p sanskript -- examples/शाकुन्तल_ऋजु_प्रतिरूप.सङ्
```

### १२.८ लयः - प्रणाली-१ द्रुत-निर्णय-यन्त्रम् (Laya: Fast System 1 Decision Engine)

Inspired by **[NandhaKishorM/laya](https://github.com/NandhaKishorM/laya)**, this advanced example demonstrates a high-performance, non-autoregressive **System 1 Decision Engine** written in 100% authentic Devanagari Sankode.

```sankode
॥ लयः - बहुभाषिक-प्रणाली-१ द्रुत-निर्णय-यन्त्रम् ॥
मान मार्गदर्शक = मार्गक(पूर्वनिर्धारित: "बहुभाषिक")।
मान यन्त्र = द्रुत_यन्त्र(तापमानम्: ०.८)।

मान मार्ग_फल = मार्गदर्शक.दिश(ऋण सन्देश)।
मान उ१ = यन्त्र.गणना(ऋण प्र_विभाग, ऋण सन्देश)।
मान उ२ = यन्त्र.गणना(ऋण प्र_क्रोध, ऋण सन्देश)।
मान उ३ = यन्त्र.गणना(ऋण प्र_आवश्यक, ऋण सन्देश)।
```

#### १. दर्शनम् (Cognitive Architecture: System 1 vs. System 2)
- **System 1 (प्रणाली १)**: Fast, intuitive, non-autoregressive decision making (< 1 ms in compiled Sankode, ~33 ms in neural checkpoints). Evaluates structured questions in a single forward pass without multi-token decoding or hallucination.
- **System 2 (प्रणाली २)**: Deliberate, multi-step autoregressive text generation (such as the Shakuntala LLM).

#### २. उप-मिलीसेकण्ड-लिपि-मार्गणम् (Sub-millisecond Script Routing)
The `मार्गक` (Router) examines Unicode script blocks (Devanagari `U+0900..U+097F` vs. Latin) in microseconds:
- **`बहुभाषिक` (Multilingual / mmBERT)**: Automatically selected whenever non-Latin Devanagari characters are detected, avoiding the catastrophic collapse of English-only models on Indic text.
- **`आङ्ग्ल` (English / ModernBERT)**: Selected for Latin-script text.
- Outputs rich telemetry explaining the decision reason, detected script, and character fraction.

#### ३. त्रिविध-प्रश्नाः (Typed Decisions Framework)
1. **`विकल्प` (Choice - QType 0)**: Multi-class categorical decision against semantic rubrics (e.g. support ticket intent: `शुल्क_व्यवस्था`, `तान्त्रिक_सहायता`, `सेवा_त्यागः`, `सामान्य_जिज्ञासा`).
2. **`क्रमाङ्क` (Score - QType 1)**: Calibrated ordinal grading on a 0..3 scale (e.g. frustration level: `०_शान्त`, `१_चिन्तित`, `२_कुपित`, `३_अत्यन्त_क्रुद्ध`).
3. **`नौल` (Noul - QType 2)**: Boolean determination (e.g. `अत्यावश्यकम्` [is urgent?], `त्याग_भयः` [churn threat?], `शुल्क_याचना` [refund requested?]).

#### ४. सम्भावना-समायोजनम् (Softmax Calibration & Brier Score)
Evaluates affinity scores across options in a single pass, applies temperature-scaled Softmax:
$$P(i) = \frac{\exp(z_i / T)}{\sum_j \exp(z_j / T)}$$
and computes the strictly proper calibration score (**Brier Score**):
$$\text{Brier} = (1 - P_{\text{max}})^2$$

#### ५. चालन-निर्देशाः (Running & Compiling)
```bash
# Run with interpreter
cargo run -p sankode-cli -- run examples/लय_द्रुत_निर्णय.सङ्

# Compile to optimized native binary (AOT)
cargo run -p sankode-cli -- build examples/लय_द्रुत_निर्णय.सङ् -o target/laya_engine.exe
./target/laya_engine.exe

# Run comparative Python and C implementations
python examples/comparative/laya_decision_engine.py
./target/laya_c.exe
```

### १२.६ तुलनात्मक-बोधः (Comparative Implementations: Sankode vs. Python vs. C)

To facilitate comparative understanding for engineers coming from C and Python backgrounds, the complete LLM implementation has also been authored in **Python** and **C99/C11** inside [`examples/comparative/`](../examples/comparative/):

- **Pure Devanagari Sankode**: [`examples/शाकुन्तल_सम्पूर्ण_प्रतिरूप.सङ्`](../examples/शाकुन्तल_सम्पूर्ण_प्रतिरूप.सङ्)
- **Python Reference**: [`examples/comparative/shakuntala_llm.py`](../examples/comparative/shakuntala_llm.py)
- **C Reference**: [`examples/comparative/shakuntala_llm.c`](../examples/comparative/shakuntala_llm.c)
- **Detailed Comparative Analysis & Matrix**: [`examples/comparative/README.md`](../examples/comparative/README.md)

#### Key Architectural Observations

1. **Memory Safety & Ownership (`स्वामित्वम्`)**:
   - **Sankode**: Compile-time affine ownership system prevents memory leaks and data races with zero garbage collection pauses.
   - **Python**: Relies on reference counting and a cyclic garbage collector, trading predictable execution latency for programmer convenience.
   - **C**: Manual allocation via `calloc()` and `free()`, requiring rigorous manual tracking of memory lifetimes.

2. **Native Unicode Strings (`सूत्रम्`)**:
   - Devanagari Sanskrit characters consist of 3-byte UTF-8 sequences.
   - **Sankode** provides first-class Unicode character indexing, slicing, splitting (`सूत्र_विभाजय`), and join operations (`सूची_संयोग`).
   - **C** requires explicit UTF-8 byte boundary decoding (`utf8_char_len`) and platform-specific wide-character file opening APIs (`_wfopen` on Windows).

3. **Bit-Exact Algorithmic Parity**:
   - All three implementations share the identical 31-bit Linear Congruential Generator (` बीज = (बीज * १६६४५२५ + १०१३९०४२२३) % २१४७४८३६४८ `), Laplace transition smoothing, Negative Log-Likelihood cross-entropy loss, and temperature-scaled softmax generation.
   - Loss: **`२.७६२३`** (Sankode) = **`2.7623`** (Python) = **`2.7623`** (C).
   - Perplexity: **`१५.८३६७`** (Sankode) = **`15.8367`** (Python) = **`15.8367`** (C).


---

## 🎓 Next Steps & Community

To contribute, report issues, or explore the source code:
- **Repository**: [github.com/tovganesh/sankode](https://github.com/tovganesh/sankode)
- **License**: MIT License
- **Issues & Discussions**: Open a GitHub Issue or Pull Request.
