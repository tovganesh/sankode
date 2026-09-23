# ॥ भाषा-प्रतिरूप-तुलनात्मक-बोधकः ॥
# Comparative Understanding: Sankode vs. Python vs. C

This directory contains equivalent implementations of the **Abhijnanasakuntalam Autoregressive Language Model (LLM)** across three distinct programming language paradigms:

1. **Pure Devanagari Sankode**: [`examples/शाकुन्तल_सम्पूर्ण_प्रतिरूप.सङ्`](../शाकुन्तल_सम्पूर्ण_प्रतिरूप.सङ्)
2. **Python Reference**: [`examples/comparative/shakuntala_llm.py`](shakuntala_llm.py)
3. **C Reference (C99/C11)**: [`examples/comparative/shakuntala_llm.c`](shakuntala_llm.c)

All three implementations execute the **identical algorithmic pipeline**, train on the same 3,391-character authentic Sanskrit corpus ([`examples/अभिज्ञानशाकुन्तलम्_मूलम्.पाठ`](../अभिज्ञानशाकुन्तलम्_मूलम्.पाठ)), use the same random seed (`987654321`), and produce **100% bit-exact numerical convergence and token generation**.

---

## 📊 Architectural & Paradigm Comparison Matrix

| Dimension | **सङ्कोड (Sankode)** | **Python** | **C (C99/C11)** |
|---|---|---|---|
| **Syntax & Literals** | Pure Devanagari script, Devanagari numerals (`०`-`९`), brace-free (`इति`), Danda terminator (`।`) | Indented blocks, colon (`:`), ASCII keywords and numerals | Curly braces (`{ }`), semicolons (`;`), ASCII keywords and numerals |
| **Memory Management** | **Compile-time Affine Ownership (`स्वामित्वम्`)**: Zero runtime pauses, deterministic cleanup, move semantics | **Runtime Garbage Collection**: Automated reference counting + cyclic GC sweeps | **Manual Memory**: Explicit `malloc()` / `free()`, prone to leaks and memory corruption |
| **Safety & Invariants** | **Compile-time Borrow Checker**: Either multiple read borrows (`ऋण`) OR one mutable borrow (`चलऋण`) | **Managed Runtime**: Memory safe but vulnerable to runtime type errors and mutable aliasing bugs | **Unchecked Raw Pointers**: Buffer overflow risks, dangling pointers, data races |
| **UTF-8 Representation** | **First-Class Native (`सूत्र`)**: UTF-8 character length, slicing, and tokenization intrinsics | **Unicode String (`str`)**: High-level character indexing with memory overhead | **Raw Byte Pointers (`char*`)**: Multi-byte Devanagari (3 bytes) requires custom decoding logic |
| **Cross-Entropy Loss** | **`२.७६२३`** | **`2.7623`** | **`2.7623`** |
| **Perplexity** | **`१५.८३६७`** | **`15.8367`** | **`15.8367`** |

---

## 🔍 Side-by-Side Code Comparison

### 1. Data Structures & Methods

#### Sankode:
```sankode
संरचना भाषा_प्रतिरूप
    शब्दा: शब्दावली।
    भार: सूची।
    आकार: पूर्ण६४।
इति

विधान भाषा_प्रतिरूप
    क्रिया प्रक्षालय(चलऋण स्व) -> रिक्त
        मान कुल_भार = स्व.आकार * स्व.आकार।
        स्व.भार = सूची_सृज(कुल_भार, ०.०)।
    इति
इति
```

#### Python:
```python
class LanguageModel:
    def __init__(self, vocab: Vocabulary) -> None:
        self.vocab: Vocabulary = vocab
        self.size: int = vocab.size
        self.weights: list[float] = [0.0] * (self.size * self.size)
```

#### C:
```c
typedef struct {
    Vocabulary vocab;
    int size;
    double* weights;
} LanguageModel;

void model_init(LanguageModel* m, const Vocabulary* v) {
    m->vocab = *v;
    m->size = v->size;
    m->weights = (double*)calloc(m->size * m->size, sizeof(double));
}
```

---

### 2. File I/O & Model Persistence

All three implementations adhere to the shared weight serialization format:
`[Vocabulary Characters]
===भार===
[Comma-separated float weights]`

#### Sankode:
```sankode
क्रिया संचय(स्व, मार्ग: ऋण सूत्र) -> रिक्त
    मान शब्दावली_सूत्र = सूची_संयोग(स्व.शब्दा.वर्णाः, "")।
    मान भार_सूत्र = सूची_संयोग(स्व.भार, ",")।
    मान सम्पूर्ण = शब्दावली_सूत्र + "\n===भार===\n" + भार_सूत्र।
    संचिका_लेख(मार्ग, सम्पूर्ण)।
इति
```

#### Python:
```python
def save(self, filepath: str) -> None:
    vocab_str = "".join(self.vocab.tokens)
    weights_str = ",".join(f"{w:.6f}" for w in self.weights)
    payload = f"{vocab_str}\n===भार===\n{weights_str}"
    with open(filepath, "w", encoding="utf-8") as f:
        f.write(payload)
```

#### C:
```c
int model_save(const LanguageModel* m, const char* filepath) {
    FILE* f = utf8_fopen(filepath, "wb");
    for (int i = 0; i < m->vocab.size; i++) fputs(m->vocab.tokens[i], f);
    fputs("\n===भार===\n", f);
    for (int i = 0; i < m->size * m->size; i++) {
        if (i > 0) fputc(',', f);
        fprintf(f, "%.6f", m->weights[i]);
    }
    fclose(f);
    return 1;
}
```

---

## 🚀 Execution Instructions

### 1. Sankode (Compiled Binary & Scripting Runtime)
```bash
# Run with compiled toolchain
cargo run -p sankode-cli -- run examples/शाकुन्तल_सम्पूर्ण_प्रतिरूप.सङ्

# Run with dynamic Sanskript runner
cargo run -p sanskript -- examples/शाकुन्तल_सम्पूर्ण_प्रतिरूप.सङ्
```

### 2. Python Reference
```bash
python examples/comparative/shakuntala_llm.py
```

### 3. C Reference
```bash
# On Windows (MSVC):
cl /utf-8 /O2 examples/comparative/shakuntala_llm.c /Fe:examples/comparative/shakuntala_llm.exe
./examples/comparative/shakuntala_llm.exe

# On Linux / macOS (GCC or Clang):
gcc -O2 examples/comparative/shakuntala_llm.c -lm -o examples/comparative/shakuntala_llm
./examples/comparative/shakuntala_llm
```

---

## 🎯 Verification & Output Parity

All three programs produce identical generative inference:

| Prompt | Sample Generation (Length = 50, Temperature = 0.70) |
|---|---|
| `"शकुन्तला"` | `शकुन्तलान श्रदावमश्यानुन्रचिकुच्कभिथिताभढभि वि चङ॥ पितिदं` |
| `"या सृष्टिः"` | `या सृष्टिः॥ स्रि स्तनाि कलि विनानुरामि चटचञञछआछा वि पप्वेष्ल` |
| `"दुष्यन्तः"` | `दुष्यन्तःङैडविजन्मठमन् भपि वगजडवत्ये द्यियेन तिनेिर्पिना वि` |

---

## 🧠 ReLU Neural Network Language Model (ऋजु-भाषा-प्रतिरूपम्)

In addition to the classical Bigram Markov Transition model, this directory also provides full **Multi-Layer Perceptron (MLP) Neural Network Language Models** featuring non-linear **ReLU activation (`रेलू`)**:

1. **Pure Devanagari Sankode**: [`examples/शाकुन्तल_ऋजु_प्रतिरूप.सङ्`](../शाकुन्तल_ऋजु_प्रतिरूप.सङ्)
2. **Python Reference**: [`examples/comparative/shakuntala_relu_llm.py`](shakuntala_relu_llm.py)
3. **C Reference (C99/C11)**: [`examples/comparative/shakuntala_relu_llm.c`](shakuntala_relu_llm.c)

### Mathematical Architecture
- **Layer 1**: $z_1 = W_1[:, c_t] + b_1$, where $W_1 \in \mathbb{R}^{32 \times 54}, b_1 \in \mathbb{R}^{32}$
- **ReLU Activation**: $h = \text{ReLU}(z_1) = \max(0.0, z_1)$
- **Layer 2**: $\text{logits} = W_2 h + b_2$, where $W_2 \in \mathbb{R}^{54 \times 32}, b_2 \in \mathbb{R}^{54}$
- **Softmax Temperature Scaling**: $P(c_{t+1} = v \mid c_t) = \frac{\exp((\text{logits}[v] - \max(\text{logits})) / T)}{\sum_j \exp((\text{logits}[j] - \max(\text{logits})) / T)}$
- **Loss & Perplexity**: Negative Log-Likelihood Cross-Entropy & $e^{\mathcal{L}}$

### 📊 Metric Parity Across All 3 Implementations

| Metric | **सङ्कोड (Sankode)** | **Python** | **C (C99/C11)** |
|---|---|---|---|
| **Model Type** | Multi-Layer Perceptron | Multi-Layer Perceptron | Multi-Layer Perceptron |
| **Hidden Dimension ($H$)** | ३२ (32) | 32 | 32 |
| **Total Parameters** | ३,५४२ (3,542) | 3,542 | 3,542 |
| **Cross-Entropy Loss (NLL)** | **`२.५६५५`** | **`2.5655`** | **`2.5655`** |
| **Perplexity ($e^{\mathcal{L}}$)** | **`१३.००६९`** | **`13.0069`** | **`13.0069`** |
| **Prompt 1 (`"शकुन्तला"`) Output** | `शकुन्तला पिजित्विषुतिरास्चितुम्तितस्यातुच दानेनिस्यतीश्तस्` | `शकुन्तला पिजित्विषुतिरास्चितुम्तितस्यातुच दानेनिस्यतीश्तस्` | `शकुन्तला पिजित्विषुतिरास्चितुम्तितस्यातुच दानेनिस्यतीश्तस्` |

### Execution Commands:
```bash
# 1. Sankode
cargo run -p sankode-cli -- run examples/शाकुन्तल_ऋजु_प्रतिरूप.सङ्

# 2. Python
python examples/comparative/shakuntala_relu_llm.py

# 3. C
cl /utf-8 /O2 examples/comparative/shakuntala_relu_llm.c /Fe:examples/comparative/shakuntala_relu_llm.exe
./examples/comparative/shakuntala_relu_llm.exe
```

---

## ⚡ Laya: Fast Non-Autoregressive System 1 Decision Engine (लयः - द्रुत-निर्णय-यन्त्रम्)

Inspired by **[NandhaKishorM/laya](https://github.com/NandhaKishorM/laya)**, this section demonstrates a high-performance **System 1 Decision Engine** across Sankode, Python, and C.

### Philosophy & Architecture
1. **Dual-Process Theory (Kahneman)**:
   - **System 1 (प्रणाली १)**: Fast, intuitive, non-autoregressive (< 1 ms in Sankode, ~33 ms in neural checkpoints), zero hallucination, direct output across typed questions.
   - **System 2 (प्रणाली २)**: Deliberate, autoregressive multi-token generation (like Shakuntala LLM).
2. **Sub-millisecond Script Routing (`Router` / `मार्गक`)**:
   - Inspects Unicode script blocks (Devanagari `U+0900..U+097F` vs Latin).
   - Routes requests instantly to `बहुभाषिक` (multilingual / mmBERT) or `आङ्ग्ल` (English / ModernBERT) with human-readable rationale.
3. **Typed Decisions (`त्रिविध-प्रश्नाः`)**:
   - **`विकल्प` (Choice)**: Multi-class categorical decision against semantic rubrics (e.g. support ticket department).
   - **`क्रमाङ्क` (Score)**: Calibrated ordinal rating on a 0..3 scale (e.g. frustration level: calm=0, concerned=1, annoyed=2, furious=3).
   - **`नौल` (Noul)**: Boolean determination (e.g. `अत्यावश्यकम्` [is_urgent], `त्याग_भयः` [churn_risk], `शुल्क_याचना` [refund_requested]).
4. **Single Forward Pass & Proper Scoring**:
   - Evaluates all questions in a single forward pass without step-by-step token decoding.
   - Softmax with temperature scaling: $P(i) = rac{\exp(z_i / T)}{\sum_j \exp(z_j / T)}$.
   - Proper scoring metric (Brier Score: $(1 - P_{\text{max}})^2$).

### Implementations
1. **Pure Devanagari Sankode**: [`examples/लय_द्रुत_निर्णय.सङ्`](../लय_द्रुत_निर्णय.सङ्)
2. **Python Reference**: [`examples/comparative/laya_decision_engine.py`](laya_decision_engine.py)
3. **C Reference (C99/C11)**: [`examples/comparative/laya_decision_engine.c`](laya_decision_engine.c)

### 📊 Verification & Prediction Parity

All 3 implementations produce identical decision outputs and calibrated confidence across production triage tickets:

| Ticket State | Router Decision | Department (`choice`) | Frustration (`score`) | Urgent (`noul`) | Churn Risk (`noul`) | Refund (`noul`) |
|---|---|---|---|---|---|---|
| **Ticket 1 (Devanagari Sanskrit/Hindi)**: `"नमस्ते। मम लेखे द्विगुणीकृतं शुल्कं गृहीतम्! कृपया मम धनं शीघ्रं प्रतिप्रेषयन्तु..."` | **`बहुभाषिक`** (96.19% Devanagari) | `शुल्क_व्यवस्था` (99.98%) | `३_अत्यन्त_क्रुद्ध` (91.25%) | `सत्यम्` (94.66%) | `सत्यम्` (94.66%) | `सत्यम्` (99.99%) |
| **Ticket 2 (English Latin)**: `"Hello support team, we were billed twice on invoice 4411. Please refund..."` | **`आङ्ग्ल`** (100% Roman) | `शुल्क_व्यवस्था` (95.95%) | `२_कुपित` (52.00%) | `सत्यम्` (99.75%) | `सत्यम्` (94.66%) | `सत्यम्` (94.66%) |
| **Ticket 3 (Technical Outage)**: `"Critical outage on production cluster: API server returning 500 error..."` | **`आङ्ग्ल`** (100% Roman) | `तान्त्रिक_सहायता` (99.9999%) | `१_चिन्तित` (26.56%) | `सत्यम्` (99.75%) | `मिथ्या` (56.22%) | `मिथ्या` (56.22%) |

### Execution Commands:
```bash
# 1. Sankode Interpreter
cargo run -p sankode-cli -- run examples/लय_द्रुत_निर्णय.सङ्

# 2. Sankode Native AOT Compilation
cargo run -p sankode-cli -- build examples/लय_द्रुत_निर्णय.सङ् -o target/laya_system1.exe
./target/laya_system1.exe

# 3. Python Reference
python examples/comparative/laya_decision_engine.py

# 4. C Reference
# On Windows (MSVC):
cl /utf-8 /O2 examples/comparative/laya_decision_engine.c /Fe:target/laya_c.exe
./target/laya_c.exe

# On Linux / macOS (GCC / Clang):
gcc -O2 examples/comparative/laya_decision_engine.c -lm -o target/laya_c
./target/laya_c
```
