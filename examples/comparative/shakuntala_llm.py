#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
================================================================================
Kalidasa's Abhijnanasakuntalam Autoregressive Language Model (Python Reference)
महाकविकालिदासविरचितस्य अभिज्ञानशाकुन्तलस्य भाषा-प्रतिरूपम् (पायथन-तुलना)
================================================================================

This program is the official, well-documented Python equivalent of the pure
Devanagari Sankode implementation found in `examples/शाकुन्तल_सम्पूर्ण_प्रतिरूप.सङ्`.

It provides a line-by-line comparative reference to illustrate how:
1. Object-oriented data structures in Python correspond to Sankode's `संरचना` & `विधान`.
2. Python's dynamic runtime and garbage collection compare with Sankode's
   compile-time affine ownership (`स्वामित्व`), move semantics, and borrowing (`ऋण`).
3. Mathematical modeling (Laplace smoothing, cross-entropy loss, perplexity, and
   temperature-scaled softmax autoregression) is expressed across both languages.
4. File I/O and weight serialization operate equivalently.
"""

from __future__ import annotations
import math
import os
import sys
from typing import List

# Ensure UTF-8 output encoding across all operating systems
if sys.stdout.encoding != "utf-8":
    try:
        sys.stdout.reconfigure(encoding="utf-8")
    except AttributeError:
        pass


# ==============================================================================
# 1. Pseudo-Random Number Generator (यादृच्छिक-संख्या-यन्त्रम्)
# ==============================================================================
class RandomGenerator:
    """
    A 31-bit Linear Congruential Generator (LCG) replicating Sankode's PRNG.

    Formula:
        X_{n+1} = (a * X_n + c) mod m
        where a = 1664525, c = 1013904223, m = 2^31 - 1 = 2147483647

    Comparative Note:
        - In Sankode: Implemented via `संरचना यादृच्छिक_यन्त्र` and `विधान`.
          The mutable receiver `चलऋण स्व` (mutable borrow) guarantees that only
          one thread or caller mutates the PRNG seed at any point in time.
        - In Python: A standard class with mutable state `self.seed`.
    """

    def __init__(self, seed: int = 987654321) -> None:
        self.seed: int = seed

    def next_int(self, limit: int) -> int:
        """Generates a pseudo-random integer in [0, limit)."""
        if limit <= 1:
            return 0
        self.seed = (self.seed * 1664525 + 1013904223) % 2147483647
        result = self.seed % limit
        return abs(result)

    def random_float(self) -> float:
        """Generates a pseudo-random floating point number in [0.0, 1.0)."""
        val = self.next_int(10000)
        return float(val) / 10000.0


# ==============================================================================
# 2. Vocabulary & Character Tokenizer (शब्दावली)
# ==============================================================================
class Vocabulary:
    """
    Character-level tokenizer and bidirectional vocabulary indexer.

    Comparative Note:
        - In Sankode: Represents `संरचना शब्दावली` holding `वर्णाः: सूची` (dynamic list).
          String slices and characters are manipulated via native Devanagari intrinsics:
          `सूत्र_दैर्घ्यम्`, `सूत्र_वर्ण`, and `सूत्र_विभाजय`.
        - In Python: Uses a standard `list[str]` and dict for O(1) index lookups.
    """

    def __init__(self, tokens: List[str] | None = None) -> None:
        self.tokens: List[str] = [] if tokens is None else list(tokens)
        self.token_to_idx: dict[str, int] = {t: i for i, t in enumerate(self.tokens)}

    def add(self, char: str) -> int:
        """Adds a new character token to the vocabulary if not already present."""
        if char not in self.token_to_idx:
            idx = len(self.tokens)
            self.tokens.append(char)
            self.token_to_idx[char] = idx
            return idx
        return self.token_to_idx[char]

    def find(self, char: str) -> int:
        """Returns the integer index of the character, or -1 if unseen."""
        return self.token_to_idx.get(char, -1)

    def get(self, idx: int) -> str:
        """Retrieves the character token at index `idx`."""
        return self.tokens[idx]

    @property
    def size(self) -> int:
        """Total number of unique tokens in vocabulary (|V|)."""
        return len(self.tokens)


# ==============================================================================
# 3. Autoregressive Language Model (भाषा-प्रतिरूपम्)
# ==============================================================================
class LanguageModel:
    """
    Autoregressive bigram language model with Laplace smoothing and temperature sampling.

    Mathematical Formulation:
        1. Transition Probability Matrix P of shape |V| x |V|:
           P(w_t | w_{t-1}) = (Count(w_{t-1}, w_t) + 1.0) / (Count(w_{t-1}) + |V|)
           where add-1.0 is Laplace smoothing preventing zero-probability bottlenecks.
        2. Cross-Entropy Loss (Negative Log-Likelihood):
           L = - (1 / N) * sum_{t=1}^{N} ln P(w_t | w_{t-1})
        3. Perplexity:
           PPL = exp(L)
        4. Softmax Sampling with Temperature T:
           P_T(w_t | w_{t-1}) propto exp( ln P(w_t | w_{t-1}) / T )

    Comparative Note:
        - In Sankode:
          `संरचना भाषा_प्रतिरूप` stores `भार: सूची` (flattened 1D array of size |V|*|V|).
          Compile-time ownership ensures that `भार` cannot be accessed after being
          moved, and methods borrow the model via `चलऋण स्व` (mutable) or `ऋण स्व` (read-only).
        - In Python:
          Stores weights in a 1D `list[float]` to maintain exact memory-layout parity.
    """

    def __init__(self, vocab: Vocabulary, weights: List[float] | None = None) -> None:
        self.vocab: Vocabulary = vocab
        self.size: int = vocab.size
        total_elements = self.size * self.size
        if weights is not None:
            self.weights: List[float] = weights
        else:
            self.weights = [0.0] * total_elements

    def train(self, text: str) -> None:
        """
        Trains bigram transition counts on the input text corpus using Laplace add-one smoothing.
        """
        v = self.size
        counts = [0.0] * (v * v)
        row_sums = [0.0] * v

        # Count bigram transitions C(w_prev, w_next)
        text_len = len(text)
        for i in range(text_len - 1):
            c_prev = text[i]
            c_next = text[i + 1]
            u = self.vocab.find(c_prev)
            w = self.vocab.find(c_next)
            if u >= 0 and w >= 0:
                counts[u * v + w] += 1.0
                row_sums[u] += 1.0

        # Apply Laplace add-one smoothing & normalize rows to probabilities
        for r in range(v):
            denom = row_sums[r] + float(v)
            for c in range(v):
                idx = r * v + c
                self.weights[idx] = (counts[idx] + 1.0) / denom

    def compute_loss(self, text: str) -> float:
        """
        Computes negative log-likelihood cross-entropy loss over the corpus text.
        """
        v = self.size
        text_len = len(text)
        if text_len <= 1:
            return 0.0

        total_nll = 0.0
        valid_transitions = 0.0

        for i in range(text_len - 1):
            c_prev = text[i]
            c_next = text[i + 1]
            u = self.vocab.find(c_prev)
            w = self.vocab.find(c_next)
            if u >= 0 and w >= 0:
                prob = self.weights[u * v + w]
                if prob > 0.0:
                    total_nll -= math.log(prob)
                    valid_transitions += 1.0

        return total_nll / valid_transitions if valid_transitions > 0 else 0.0

    def save(self, filepath: str) -> None:
        """
        Serializes model vocabulary and transition weights to disk.
        Uses exact file format matching Sankode's `\n===भार===\n` delimiter.
        """
        os.makedirs(os.path.dirname(os.path.abspath(filepath)), exist_ok=True)

        vocab_str = "".join(self.vocab.tokens)
        weights_str = ",".join(f"{w:.6f}" for w in self.weights)
        payload = f"{vocab_str}\n===भार===\n{weights_str}"

        with open(filepath, "w", encoding="utf-8") as f:
            f.write(payload)

    @classmethod
    def load(cls, filepath: str) -> LanguageModel:
        """
        Loads vocabulary and weights from a serialized weights file on disk.
        """
        with open(filepath, "r", encoding="utf-8") as f:
            content = f.read()

        parts = content.split("\n===भार===\n")
        vocab_str = parts[0]
        weights_str = parts[1]

        tokens = list(vocab_str)
        vocab = Vocabulary(tokens)

        weights = [float(x.strip()) for x in weights_str.split(",") if x.strip()]
        return cls(vocab, weights)

    def generate(
        self,
        rng: RandomGenerator,
        prompt: str,
        length: int = 50,
        temperature: float = 0.70,
    ) -> str:
        """
        Autoregressively generates text from an initial prompt using temperature-scaled sampling.
        """
        if not prompt:
            return ""

        result = prompt
        curr_char = prompt[-1]
        v = self.size

        for _ in range(length):
            u = self.vocab.find(curr_char)
            if u < 0:
                # Unseen character fallback: uniform random sample
                next_idx = rng.next_int(v)
                curr_char = self.vocab.get(next_idx)
                result += curr_char
                continue

            # Compute temperature-scaled unnormalized probabilities: P^(1/T)
            base_idx = u * v
            scaled_exp = [0.0] * v
            exp_sum = 0.0

            for c in range(v):
                prob = self.weights[base_idx + c]
                val = math.exp(math.log(prob) / temperature)
                scaled_exp[c] = val
                exp_sum += val

            # Sample from categorical distribution via cumulative sum
            rand_point = rng.random_float() * exp_sum
            cum_sum = 0.0
            chosen_idx = v - 1

            for c in range(v):
                cum_sum += scaled_exp[c]
                if cum_sum >= rand_point:
                    chosen_idx = c
                    break

            next_char = self.vocab.get(chosen_idx)
            result += next_char
            curr_char = next_char

        return result


# ==============================================================================
# 4. Main Pipeline
# ==============================================================================
def main() -> None:
    print("================================================================")
    print("॥ पायथन-तन्त्रे अभिज्ञानशाकुन्तल-सम्पूर्ण-प्रतिरूपम् ॥")
    print("Kalidasa's Abhijnanasakuntalam LLM Pipeline (Python Reference)")
    print("================================================================")

    repo_root = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
    corpus_file = os.path.join(repo_root, "examples", "अभिज्ञानशाकुन्तलम्_मूलम्.पाठ")
    weights_file = os.path.join(repo_root, "examples", "शाकुन्तल_प्रतिरूप_python.भार")

    # Step 1: Read Corpus
    print("--- 1. Reading Corpus File (संचिकातः शाकुन्तल-पाठ-पठनम्) ---")
    if not os.path.exists(corpus_file):
        print(f"Error: Corpus file not found at {corpus_file}")
        sys.exit(1)

    with open(corpus_file, "r", encoding="utf-8") as f:
        corpus = f.read()

    print(f"Corpus Character Length (वर्णदैर्घ्यम्): {len(corpus)}")

    # Step 2: Build Vocabulary
    vocab = Vocabulary()
    for ch in corpus:
        vocab.add(ch)

    print(f"Vocabulary Size (विशिष्ट-वर्णानां सङ्ख्या): {vocab.size}")

    # Step 3: Train Model
    print("--- 2. Training Autoregressive Bigram Model (प्रतिरूपस्य प्रशिक्षणम्) ---")
    model = LanguageModel(vocab)
    model.train(corpus)
    print("Model training complete (प्रतिरूपस्य प्रशिक्षणं सम्पन्नम्)!")

    loss = model.compute_loss(corpus)
    perplexity = math.exp(loss)
    print(f"Cross-Entropy Loss (प्रशिक्षण-हानिः): {loss:.4f}")
    print(f"Perplexity (प्रतिरूपस्य संभ्रमः): {perplexity:.4f}")

    # Step 4: Save Weights
    print("--- 3. Saving Model Weights to Disk (संचिकायां भार-सञ्चयः) ---")
    model.save(weights_file)
    print(f"Weights successfully saved to: {weights_file}")

    # Step 5: Load Weights from Disk
    print("--- 4. Loading Model Weights from Disk (संचिकातः भारोद्धारः) ---")
    loaded_model = LanguageModel.load(weights_file)
    print("Weights successfully loaded from disk!")
    print(f"Loaded Vocabulary Size: {loaded_model.size}")

    # Step 6: Test Generative Inference
    print("--- 5. Testing Generative Inference (भाषा-सृजन-परीक्षणम्) ---")
    rng = RandomGenerator(seed=987654321)

    prompts = ["शकुन्तला", "या सृष्टिः", "दुष्यन्तः"]
    for i, prompt in enumerate(prompts, 1):
        print(f"[Generation {i}: Prompt '{prompt}']:")
        output = loaded_model.generate(rng, prompt=prompt, length=50, temperature=0.70)
        print(output)
        print()

    print("================================================================")
    print("॥ सम्पूर्ण-प्रक्रिया सफलीभूता (Full Pipeline Succeeded!) ॥")
    print("================================================================")


if __name__ == "__main__":
    main()