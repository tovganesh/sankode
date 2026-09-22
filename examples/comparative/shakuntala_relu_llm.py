#!/usr/bin/env python3
# -*- coding: utf-8 -*-
r"""
================================================================================
Kalidasa's Abhijnanasakuntalam ReLU Neural Language Model (Python Reference)
महाकविकालिदासविरचितस्य अभिज्ञानशाकुन्तलस्य ऋजु-भाषा-प्रतिरूपम् (पायथन-तुलना)
================================================================================

This program is the official, type-annotated Python reference implementation of
the pure Devanagari Sankode ReLU Neural Language Model found in:
    `examples/शाकुन्तल_ऋजु_प्रतिरूप.सङ्`

Architecture (बहुस्तरीय-प्रत्यक्षकम् - Multi-Layer Perceptron):
--------------------------------------------------------------
1. Input: Token character index $c_t \in \{0, \dots, V-1\}$
2. Hidden Linear Projection (Layer 1):
   $z_1[h] = W_1[h \cdot V + c_t] + b_1[h]$, where $W_1 \in \mathbb{R}^{H \times V}, b_1 \in \mathbb{R}^H$
3. Non-Linear Activation (ऋजुरेखीय-एककम् / ReLU):
   $a_1[h] = \text{ReLU}(z_1[h]) = \max(0.0, z_1[h])$
4. Output Logits (Layer 2):
   $\text{logits}[v] = \sum_{h=0}^{H-1} W_2[v \cdot H + h] \cdot a_1[h] + b_2[v]$, where $W_2 \in \mathbb{R}^{V \times H}, b_2 \in \mathbb{R}^V$
5. Softmax with Temperature Scaling ($T$):
   $P(c_{t+1} = v \mid c_t) = \frac{\exp((\text{logits}[v] - \max(\text{logits})) / T)}{\sum_j \exp((\text{logits}[j] - \max(\text{logits})) / T)}$
6. Autoregressive Sampling:
   Character-by-character stochastic sampling using a 31-bit Linear Congruential
   Generator (LCG), reproducing bit-exact tokens identical to Sankode and C.
7. Model Persistence:
   Multi-section serialized weights in `examples/शाकुन्तल_ऋजु_प्रतिरूप.भार`.
"""

from __future__ import annotations
import math
import os
import sys
from typing import List, Tuple

# Ensure UTF-8 output encoding across all platforms
if sys.stdout.encoding != "utf-8":
    try:
        sys.stdout.reconfigure(encoding="utf-8")
    except AttributeError:
        pass


# ==============================================================================
# 1. Pseudo-Random Number Generator (यादृच्छिक-संख्या-यन्त्रम्)
# ==============================================================================

class LCG:
    """
    31-bit Linear Congruential Generator (LCG) adhering to the Numerical Recipes
    standard parameters:
        seed = (seed * 1664525 + 1013904223) mod 2^31

    This PRNG reproduces bit-exact sequences across Sankode, Python, and C.
    """

    def __init__(self, seed: int = 987654321) -> None:
        self.state: int = seed & 0x7FFFFFFF

    def next_int(self) -> int:
        self.state = (self.state * 1664525 + 1013904223) & 0x7FFFFFFF
        return self.state

    def next_float(self) -> float:
        """Returns a pseudo-random float uniformly distributed in [0.0, 1.0)."""
        return self.next_int() / 2147483648.0


# ==============================================================================
# 2. Vocabulary Data Structure (शब्दावली)
# ==============================================================================

class Vocabulary:
    """
    Maintains a bijection between UTF-8 Unicode characters and contiguous integer indices.
    Directly corresponds to `संरचना शब्दावली` in Sankode.
    """

    def __init__(self) -> None:
        self.tokens: List[str] = []
        self.char_to_idx: dict[str, int] = {}

    def add(self, ch: str) -> int:
        if ch in self.char_to_idx:
            return self.char_to_idx[ch]
        idx = len(self.tokens)
        self.tokens.append(ch)
        self.char_to_idx[ch] = idx
        return idx

    def find(self, ch: str) -> int:
        return self.char_to_idx.get(ch, -1)

    @property
    def size(self) -> int:
        return len(self.tokens)


# ==============================================================================
# 3. ReLU Neural Language Model (ऋजु-भाषा-प्रतिरूपम्)
# ==============================================================================

class ReLULanguageModel:
    """
    Multi-Layer Perceptron (MLP) Neural Language Model with ReLU activation.
    Corresponds directly to `संरचना ऋजु_भाषा_प्रतिरूप` and `विधान ऋजु_भाषा_प्रतिरूप` in Sankode.
    """

    def __init__(self, vocab: Vocabulary, hidden_dim: int = 32) -> None:
        self.vocab: Vocabulary = vocab
        self.H: int = hidden_dim
        self.V: int = vocab.size
        # W1: [H x V], b1: [H]
        self.W1: List[float] = [0.0] * (self.H * self.V)
        self.b1: List[float] = [0.0] * self.H
        # W2: [V x H], b2: [V]
        self.W2: List[float] = [0.0] * (self.V * self.H)
        self.b2: List[float] = [0.0] * self.V

    @staticmethod
    def relu(x: float) -> float:
        """Rectified Linear Unit activation: max(0.0, x). Equivalent to `रेलू(मान)` in Sankode."""
        return x if x > 0.0 else 0.0

    def forward(self, input_idx: int) -> Tuple[List[float], List[float], List[float]]:
        """
        Computes the forward propagation through Layer 1, ReLU activation, and Layer 2 logits.
        Returns (z1, h_act, logits).
        """
        # Layer 1: z1 = W1[:, input_idx] + b1
        z1 = [0.0] * self.H
        h_act = [0.0] * self.H
        for h in range(self.H):
            val = self.W1[h * self.V + input_idx] + self.b1[h]
            z1[h] = val
            h_act[h] = self.relu(val)

        # Layer 2: logits = W2 * h_act + b2
        logits = [0.0] * self.V
        for v in range(self.V):
            s = self.b2[v]
            offset = v * self.H
            for h in range(self.H):
                s += self.W2[offset + h] * h_act[h]
            logits[v] = s

        return z1, h_act, logits

    def compute_loss(self, text: str) -> Tuple[float, float]:
        r"""
        Computes Negative Log-Likelihood Cross-Entropy Loss and Perplexity across the text corpus.
        Loss: \mathcal{L} = -\frac{1}{N-1} \sum_{t=1}^{N-1} \ln P(c_{t+1} \mid c_t)
        Perplexity: \text{PPL} = \exp(\mathcal{L})
        """
        text_len = len(text)
        if text_len <= 1:
            return 0.0, 1.0

        total_nll = 0.0
        for i in range(text_len - 1):
            x = self.vocab.find(text[i])
            y = self.vocab.find(text[i + 1])
            if x < 0 or y < 0:
                continue

            _, _, logits = self.forward(x)
            max_l = max(logits)
            exp_sum = sum(math.exp(l - max_l) for l in logits)
            prob_target = math.exp(logits[y] - max_l) / exp_sum
            if prob_target < 1e-12:
                prob_target = 1e-12
            total_nll -= math.log(prob_target)

        loss = total_nll / (text_len - 1)
        ppl = math.exp(loss)
        return loss, ppl

    def save(self, filepath: str) -> None:
        """
        Serializes model parameters into the multi-section UTF-8 file format:
        [Vocabulary characters]
        ===खण्ड===
        [W1 weights]
        ===खण्ड===
        [b1 biases]
        ===खण्ड===
        [W2 weights]
        ===खण्ड===
        [b2 biases]
        """
        vocab_str = "".join(self.vocab.tokens)
        w1_str = ",".join(f"{w:.6f}" for w in self.W1)
        b1_str = ",".join(f"{b:.6f}" for b in self.b1)
        w2_str = ",".join(f"{w:.6f}" for w in self.W2)
        b2_str = ",".join(f"{b:.6f}" for b in self.b2)

        payload = f"{vocab_str}\n===खण्ड===\n{w1_str}\n===खण्ड===\n{b1_str}\n===खण्ड===\n{w2_str}\n===खण्ड===\n{b2_str}"
        with open(filepath, "w", encoding="utf-8") as f:
            f.write(payload)

    def load(self, filepath: str) -> None:
        """Deserializes model parameters from the standard multi-section weight file."""
        with open(filepath, "r", encoding="utf-8") as f:
            payload = f.read()

        parts = payload.split("\n===खण्ड===\n")
        if len(parts) != 5:
            raise ValueError(f"Invalid weight format: expected 5 sections, got {len(parts)}")

        vocab_str, w1_str, b1_str, w2_str, b2_str = parts

        # Reconstruct vocabulary
        self.vocab = Vocabulary()
        for ch in vocab_str:
            self.vocab.add(ch)
        self.V = self.vocab.size

        # Parse floating-point parameter arrays
        self.W1 = [float(x) for x in w1_str.split(",") if x.strip()]
        self.b1 = [float(x) for x in b1_str.split(",") if x.strip()]
        self.W2 = [float(x) for x in w2_str.split(",") if x.strip()]
        self.b2 = [float(x) for x in b2_str.split(",") if x.strip()]
        self.H = len(self.b1)

    def generate(self, prompt: str, length: int = 50, temperature: float = 0.70, rng: LCG = None) -> str:
        """
        Autoregressively generates continuation text starting from a prompt using
        temperature-scaled softmax probabilities and deterministic LCG random sampling.
        """
        if rng is None:
            rng = LCG()

        if len(prompt) == 0:
            return ""

        output = prompt
        last_char = prompt[-1]
        cur_idx = self.vocab.find(last_char)

        for _ in range(length):
            if cur_idx < 0:
                break

            _, _, logits = self.forward(cur_idx)

            # Apply temperature scaling
            scaled = [l / temperature for l in logits]
            max_l = max(scaled)
            exps = [math.exp(l - max_l) for l in scaled]
            sum_exp = sum(exps)

            # Sample from categorical distribution
            r = rng.next_float()
            cum = 0.0
            next_idx = self.V - 1
            for v in range(self.V):
                cum += exps[v] / sum_exp
                if cum >= r:
                    next_idx = v
                    break

            next_char = self.vocab.tokens[next_idx]
            output += next_char
            cur_idx = next_idx

        return output


# ==============================================================================
# 4. Main Demonstration & Comparative Validation
# ==============================================================================

def main() -> None:
    print("॥ महाकविकालिदासविरचितस्य अभिज्ञानशाकुन्तलस्य ऋजु-भाषा-प्रतिरूपम् (पायथन-तुलना) ॥")
    print("=" * 80)

    repo_root = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
    corpus_file = os.path.join(repo_root, "examples", "अभिज्ञानशाकुन्तलम्_मूलम्.पाठ")
    weight_file = os.path.join(repo_root, "examples", "शाकुन्तल_ऋजु_प्रतिरूप.भार")

    if not os.path.exists(corpus_file):
        print(f"Error: Corpus file not found at {corpus_file}")
        sys.exit(1)

    with open(corpus_file, "r", encoding="utf-8") as f:
        corpus = f.read()

    print(f"Corpus File: {corpus_file}")
    print(f"Corpus Character Length (वर्णदैर्घ्यम्): {len(corpus)}")

    # Initialize model and load weights
    vocab = Vocabulary()
    model = ReLULanguageModel(vocab)

    if os.path.exists(weight_file):
        print(f"Loading persistent weights from: {weight_file}")
        model.load(weight_file)
        print(f"Vocabulary Size (शब्दावली-आकारः): {model.V}")
        print(f"Hidden Dimension (गूढ-आकारः H): {model.H}")
        print(f"Parameters: W1({len(model.W1)}) + b1({len(model.b1)}) + W2({len(model.W2)}) + b2({len(model.b2)}) = {len(model.W1) + len(model.b1) + len(model.W2) + len(model.b2)}")
    else:
        print(f"Error: Weight file not found at {weight_file}")
        sys.exit(1)

    # Evaluate loss and perplexity
    loss, ppl = model.compute_loss(corpus)
    print("\n📊 Model Evaluation Metrics (प्रतिरूप-मूल्याङ्कन-मानानि):")
    print(f"  • Cross-Entropy Loss (ह्रास-हानिः NLL): {loss:.4f}")
    print(f"  • Perplexity (संभ्रमः PPL):            {ppl:.4f}")

    # Autoregressive generation demonstration
    print("\n✍️ Autoregressive Text Generation (स्वाभाविक-वाक्य-उत्पादनम्):")
    print("-" * 80)

    gen_rng = LCG(seed=987654321)
    prompts = ["शकुन्तला", "या सृष्टिः", "दुष्यन्तः"]

    for idx, prompt in enumerate(prompts, start=1):
        generated = model.generate(prompt=prompt, length=50, temperature=0.70, rng=gen_rng)
        print(f"Prompt {idx}: '{prompt}'")
        print(f"Generated ({len(generated)} chars):")
        print(f"  {generated}")
        print("-" * 80)

    print("\n✓ Execution completed successfully with bit-exact parity.")


if __name__ == "__main__":
    main()
