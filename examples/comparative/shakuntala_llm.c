/*
================================================================================
Kalidasa's Abhijnanasakuntalam Autoregressive Language Model (C Reference)
महाकविकालिदासविरचितस्य अभिज्ञानशाकुन्तलस्य भाषा-प्रतिरूपम् (सी-तुलना)
================================================================================

This program is the official, well-documented C99/C11 equivalent of the pure
Devanagari Sankode implementation found in `examples/शाकुन्तल_सम्पूर्ण_प्रतिरूप.सङ्`.

Comparative Architecture & Memory Model Notes:
---------------------------------------------
1. Manual Memory vs. Affine Ownership:
   - In C: All dynamic buffers (`weights`, `corpus`, `tokens`) require explicit
     `malloc()` and `free()`. Failure to free causes memory leaks; dangling
     pointers cause undefined behavior or security exploits.
   - In Sankode: Memory is governed by compile-time affine ownership (`स्वामित्व`).
     When a `संरचना` or `सूची` goes out of scope, it is deterministically freed
     with zero garbage collection pauses and zero use-after-free risks.

2. UTF-8 Devanagari Strings:
   - In C: Strings are raw `char*` byte arrays. Devanagari characters span 3 bytes
     each in UTF-8 (`0xE0 0xA4/0xA5 ...`). We implement a custom UTF-8 character
     boundary decoder (`utf8_char_len`).
   - In Sankode: UTF-8 strings (`सूत्र`) and character tokens are native, first-class
     types with compile-time immutability and built-in intrinsics (`सूत्र_दैर्घ्यम्`,
     `सूत्र_वर्ण`, `सूत्र_विभाजय`).

3. Borrow Checking vs. Raw Pointers:
   - In C: Passing pointers (`LanguageModel*`) offers zero aliasing guarantees.
     Multiple pointers can simultaneously mutate shared state, leading to race conditions.
   - In Sankode: The borrow checker enforces XOR aliasing: either multiple shared
     immutable borrows (`ऋण`), OR exactly one exclusive mutable borrow (`चलऋण`).
*/

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>

#ifdef _WIN32
#include <windows.h>
#endif

#define MAX_TOKENS 128
#define MAX_TOKEN_BYTES 8
#define SEED_INIT 987654321LL
#define DELIMITER "\n===भार===\n"

/* Cross-platform UTF-8 file open (supporting Unicode Devanagari paths on Windows) */
static FILE* utf8_fopen(const char* filename, const char* mode) {
#ifdef _WIN32
    wchar_t wfilename[1024];
    wchar_t wmode[32];
    MultiByteToWideChar(CP_UTF8, 0, filename, -1, wfilename, 1024);
    MultiByteToWideChar(CP_UTF8, 0, mode, -1, wmode, 32);
    return _wfopen(wfilename, wmode);
#else
    return fopen(filename, mode);
#endif
}


/* =============================================================================
 * 1. UTF-8 Utility Functions (सूत्र-परीक्षण-साधनानि)
 * =============================================================================
 * Devanagari characters in Unicode NFKC are encoded as 3-byte sequences in UTF-8.
 * Standard ASCII characters (spaces, punctuation) are 1 byte.
 */
static int utf8_char_len(unsigned char c) {
    if ((c & 0x80) == 0x00) return 1; /* 0xxxxxxx: ASCII */
    if ((c & 0xE0) == 0xC0) return 2; /* 110xxxxx */
    if ((c & 0xF0) == 0xE0) return 3; /* 1110xxxx: Devanagari letters & matras */
    if ((c & 0xF8) == 0xF0) return 4; /* 11110xxx */
    return 1;
}

/* =============================================================================
 * 2. Pseudo-Random Number Generator (यादृच्छिक-संख्या-यन्त्रम्)
 * =============================================================================
 * 31-bit Linear Congruential Generator matching Sankode's exact parameters.
 */
typedef struct {
    long long seed;
} RandomGenerator;

void rng_init(RandomGenerator* rng, long long seed) {
    rng->seed = seed;
}

int rng_next_int(RandomGenerator* rng, int limit) {
    if (limit <= 1) return 0;
    rng->seed = (rng->seed * 1664525LL + 1013904223LL) % 2147483647LL;
    long long res = rng->seed % limit;
    if (res < 0) res = -res;
    return (int)res;
}

double rng_random_float(RandomGenerator* rng) {
    int val = rng_next_int(rng, 10000);
    return (double)val / 10000.0;
}

/* =============================================================================
 * 3. Vocabulary & Character Tokenizer (शब्दावली)
 * =============================================================================
 */
typedef struct {
    char tokens[MAX_TOKENS][MAX_TOKEN_BYTES];
    int size;
} Vocabulary;

void vocab_init(Vocabulary* v) {
    v->size = 0;
    memset(v->tokens, 0, sizeof(v->tokens));
}

int vocab_find(const Vocabulary* v, const char* token) {
    for (int i = 0; i < v->size; i++) {
        if (strcmp(v->tokens[i], token) == 0) {
            return i;
        }
    }
    return -1;
}

int vocab_add(Vocabulary* v, const char* token) {
    int existing = vocab_find(v, token);
    if (existing >= 0) return existing;
    if (v->size >= MAX_TOKENS) return -1;

    strncpy(v->tokens[v->size], token, MAX_TOKEN_BYTES - 1);
    v->tokens[v->size][MAX_TOKEN_BYTES - 1] = '\0';
    v->size++;
    return v->size - 1;
}

const char* vocab_get(const Vocabulary* v, int idx) {
    if (idx >= 0 && idx < v->size) {
        return v->tokens[idx];
    }
    return "";
}

/* =============================================================================
 * 4. Autoregressive Language Model (भाषा-प्रतिरूपम्)
 * =============================================================================
 */
typedef struct {
    Vocabulary vocab;
    int size;          /* |V| */
    double* weights;   /* Flattened 2D transition probability matrix of size |V| * |V| */
} LanguageModel;

void model_init(LanguageModel* m, const Vocabulary* v) {
    m->vocab = *v;
    m->size = v->size;
    int total = m->size * m->size;
    m->weights = (double*)calloc(total > 0 ? total : 1, sizeof(double));
}

void model_free(LanguageModel* m) {
    if (m->weights) {
        free(m->weights);
        m->weights = NULL;
    }
    m->size = 0;
}

/*
 * Trains the autoregressive bigram transition model on UTF-8 corpus text.
 * Uses Laplace add-one smoothing to prevent zero-probability transitions.
 */
void model_train(LanguageModel* m, const char* corpus) {
    int v = m->size;
    int total = v * v;
    double* counts = (double*)calloc(total, sizeof(double));
    double* row_sums = (double*)calloc(v, sizeof(double));

    const unsigned char* ptr = (const unsigned char*)corpus;
    char prev_tok[MAX_TOKEN_BYTES] = {0};
    int has_prev = 0;

    while (*ptr) {
        int len = utf8_char_len(*ptr);
        char curr_tok[MAX_TOKEN_BYTES] = {0};
        memcpy(curr_tok, ptr, len);
        curr_tok[len] = '\0';

        if (has_prev) {
            int u = vocab_find(&m->vocab, prev_tok);
            int w = vocab_find(&m->vocab, curr_tok);
            if (u >= 0 && w >= 0) {
                counts[u * v + w] += 1.0;
                row_sums[u] += 1.0;
            }
        }

        memcpy(prev_tok, curr_tok, MAX_TOKEN_BYTES);
        has_prev = 1;
        ptr += len;
    }

    /* Laplace Smoothing & Normalization */
    for (int r = 0; r < v; r++) {
        double denom = row_sums[r] + (double)v;
        for (int c = 0; c < v; c++) {
            int idx = r * v + c;
            m->weights[idx] = (counts[idx] + 1.0) / denom;
        }
    }

    free(counts);
    free(row_sums);
}

/*
 * Computes cross-entropy loss (Negative Log-Likelihood) over the corpus.
 */
double model_compute_loss(const LanguageModel* m, const char* corpus) {
    int v = m->size;
    const unsigned char* ptr = (const unsigned char*)corpus;
    char prev_tok[MAX_TOKEN_BYTES] = {0};
    int has_prev = 0;

    double total_nll = 0.0;
    double count = 0.0;

    while (*ptr) {
        int len = utf8_char_len(*ptr);
        char curr_tok[MAX_TOKEN_BYTES] = {0};
        memcpy(curr_tok, ptr, len);
        curr_tok[len] = '\0';

        if (has_prev) {
            int u = vocab_find(&m->vocab, prev_tok);
            int w = vocab_find(&m->vocab, curr_tok);
            if (u >= 0 && w >= 0) {
                double prob = m->weights[u * v + w];
                if (prob > 0.0) {
                    total_nll -= log(prob);
                    count += 1.0;
                }
            }
        }

        memcpy(prev_tok, curr_tok, MAX_TOKEN_BYTES);
        has_prev = 1;
        ptr += len;
    }

    return count > 0.0 ? total_nll / count : 0.0;
}

/*
 * Saves vocabulary and weights to disk in format matching Sankode.
 */
int model_save(const LanguageModel* m, const char* filepath) {
    FILE* f = utf8_fopen(filepath, "wb");
    if (!f) return 0;

    /* Write vocabulary characters */
    for (int i = 0; i < m->vocab.size; i++) {
        fputs(m->vocab.tokens[i], f);
    }
    fputs(DELIMITER, f);

    /* Write comma-separated weights */
    int total = m->size * m->size;
    for (int i = 0; i < total; i++) {
        if (i > 0) fputc(',', f);
        fprintf(f, "%.6f", m->weights[i]);
    }

    fclose(f);
    return 1;
}

/*
 * Loads vocabulary and weights from disk.
 */
int model_load(LanguageModel* m, const char* filepath) {
    FILE* f = utf8_fopen(filepath, "rb");
    if (!f) return 0;

    fseek(f, 0, SEEK_END);
    long fsize = ftell(f);
    fseek(f, 0, SEEK_SET);

    char* buffer = (char*)malloc(fsize + 1);
    fread(buffer, 1, fsize, f);
    buffer[fsize] = '\0';
    fclose(f);

    char* delim_pos = strstr(buffer, DELIMITER);
    if (!delim_pos) {
        free(buffer);
        return 0;
    }

    *delim_pos = '\0';
    char* vocab_part = buffer;
    char* weights_part = delim_pos + strlen(DELIMITER);

    /* Reconstruct vocabulary */
    Vocabulary v;
    vocab_init(&v);
    unsigned char* vptr = (unsigned char*)vocab_part;
    while (*vptr) {
        int len = utf8_char_len(*vptr);
        char tok[MAX_TOKEN_BYTES] = {0};
        memcpy(tok, vptr, len);
        tok[len] = '\0';
        vocab_add(&v, tok);
        vptr += len;
    }

    model_init(m, &v);

    /* Parse comma-separated weights */
    int total = m->size * m->size;
    char* token = strtok(weights_part, ",\r\n");
    int idx = 0;
    while (token && idx < total) {
        m->weights[idx++] = atof(token);
        token = strtok(NULL, ",\r\n");
    }

    free(buffer);
    return 1;
}

/*
 * Generates text from a prompt using temperature-controlled softmax sampling.
 */
void model_generate(
    LanguageModel* m,
    RandomGenerator* rng,
    const char* prompt,
    int length,
    double temperature,
    char* out_buf,
    size_t out_buf_size
) {
    if (!prompt || strlen(prompt) == 0) {
        out_buf[0] = '\0';
        return;
    }

    strcpy(out_buf, prompt);

    /* Extract the last UTF-8 character from prompt */
    int p_len = (int)strlen(prompt);
    int last_char_start = 0;
    int cur = 0;
    while (cur < p_len) {
        last_char_start = cur;
        cur += utf8_char_len((unsigned char)prompt[cur]);
    }
    char curr_tok[MAX_TOKEN_BYTES] = {0};
    strncpy(curr_tok, prompt + last_char_start, cur - last_char_start);
    curr_tok[cur - last_char_start] = '\0';

    int v = m->size;
    double* scaled_exp = (double*)malloc(v * sizeof(double));

    for (int step = 0; step < length; step++) {
        int u = vocab_find(&m->vocab, curr_tok);
        if (u < 0) {
            int next_idx = rng_next_int(rng, v);
            const char* next_tok = vocab_get(&m->vocab, next_idx);
            strncat(out_buf, next_tok, out_buf_size - strlen(out_buf) - 1);
            strncpy(curr_tok, next_tok, MAX_TOKEN_BYTES);
            continue;
        }

        int base_idx = u * v;
        double exp_sum = 0.0;
        for (int c = 0; c < v; c++) {
            double prob = m->weights[base_idx + c];
            double val = exp(log(prob) / temperature);
            scaled_exp[c] = val;
            exp_sum += val;
        }

        double rand_point = rng_random_float(rng) * exp_sum;
        double cum_sum = 0.0;
        int chosen_idx = v - 1;

        for (int c = 0; c < v; c++) {
            cum_sum += scaled_exp[c];
            if (cum_sum >= rand_point) {
                chosen_idx = c;
                break;
            }
        }

        const char* next_tok = vocab_get(&m->vocab, chosen_idx);
        strncat(out_buf, next_tok, out_buf_size - strlen(out_buf) - 1);
        strncpy(curr_tok, next_tok, MAX_TOKEN_BYTES);
    }

    free(scaled_exp);
}

/* =============================================================================
 * 5. Main Pipeline
 * =============================================================================
 */
int main(void) {
#ifdef _WIN32
    SetConsoleOutputCP(CP_UTF8);
#endif

    printf("================================================================\n");
    printf("॥ सी-तन्त्रे अभिज्ञानशाकुन्तल-सम्पूर्ण-प्रतिरूपम् ॥\n");
    printf("Kalidasa's Abhijnanasakuntalam LLM Pipeline (C Reference)\n");
    printf("================================================================\n");

    const char* corpus_file = "examples/अभिज्ञानशाकुन्तलम्_मूलम्.पाठ";
    const char* weights_file = "examples/शाकुन्तल_प्रतिरूप_c.भार";

    /* Step 1: Read Corpus */
    printf("--- 1. Reading Corpus File (संचिकातः शाकुन्तल-पाठ-पठनम्) ---\n");
    FILE* f = utf8_fopen(corpus_file, "rb");
    if (!f) {
        fprintf(stderr, "Error opening corpus file: %s\n", corpus_file);
        return 1;
    }
    fseek(f, 0, SEEK_END);
    long fsize = ftell(f);
    fseek(f, 0, SEEK_SET);

    char* corpus = (char*)malloc(fsize + 1);
    fread(corpus, 1, fsize, f);
    corpus[fsize] = '\0';
    fclose(f);

    /* Normalize CRLF to LF */
    char* wptr = corpus;
    for (char* rptr = corpus; *rptr; rptr++) {
        if (*rptr != '\r') {
            *wptr++ = *rptr;
        }
    }
    *wptr = '\0';

    /* Count UTF-8 characters */
    int char_count = 0;
    unsigned char* ptr = (unsigned char*)corpus;
    while (*ptr) {
        ptr += utf8_char_len(*ptr);
        char_count++;
    }
    printf("Corpus Character Length (वर्णदैर्घ्यम्): %d\n", char_count);

    /* Step 2: Build Vocabulary */
    Vocabulary vocab;
    vocab_init(&vocab);
    ptr = (unsigned char*)corpus;
    while (*ptr) {
        int len = utf8_char_len(*ptr);
        char tok[MAX_TOKEN_BYTES] = {0};
        memcpy(tok, ptr, len);
        tok[len] = '\0';
        vocab_add(&vocab, tok);
        ptr += len;
    }
    printf("Vocabulary Size (विशिष्ट-वर्णानां सङ्ख्या): %d\n", vocab.size);

    /* Step 3: Train Model */
    printf("--- 2. Training Autoregressive Bigram Model (प्रतिरूपस्य प्रशिक्षणम्) ---\n");
    LanguageModel model;
    model_init(&model, &vocab);
    model_train(&model, corpus);
    printf("Model training complete (प्रतिरूपस्य प्रशिक्षणं सम्पन्नम्)!\n");

    double loss = model_compute_loss(&model, corpus);
    double perplexity = exp(loss);
    printf("Cross-Entropy Loss (प्रशिक्षण-हानिः): %.4f\n", loss);
    printf("Perplexity (प्रतिरूपस्य संभ्रमः): %.4f\n", perplexity);

    /* Step 4: Save Weights */
    printf("--- 3. Saving Model Weights to Disk (संचिकायां भार-सञ्चयः) ---\n");
    if (model_save(&model, weights_file)) {
        printf("Weights successfully saved to: %s\n", weights_file);
    } else {
        fprintf(stderr, "Failed to save weights to: %s\n", weights_file);
    }

    /* Step 5: Load Weights */
    printf("--- 4. Loading Model Weights from Disk (संचिकातः भारोद्धारः) ---\n");
    LanguageModel loaded_model;
    if (model_load(&loaded_model, weights_file)) {
        printf("Weights successfully loaded from disk!\n");
        printf("Loaded Vocabulary Size: %d\n", loaded_model.size);
    } else {
        fprintf(stderr, "Failed to load weights from: %s\n", weights_file);
        return 1;
    }

    /* Step 6: Test Generative Inference */
    printf("--- 5. Testing Generative Inference (भाषा-सृजन-परीक्षणम्) ---\n");
    RandomGenerator rng;
    rng_init(&rng, SEED_INIT);

    const char* prompts[] = {"शकुन्तला", "या सृष्टिः", "दुष्यन्तः"};
    char gen_buf[2048];

    for (int i = 0; i < 3; i++) {
        printf("[Generation %d: Prompt '%s']:\n", i + 1, prompts[i]);
        model_generate(&loaded_model, &rng, prompts[i], 50, 0.70, gen_buf, sizeof(gen_buf));
        printf("%s\n\n", gen_buf);
    }

    printf("================================================================\n");
    printf("॥ सम्पूर्ण-प्रक्रिया सफलीभूता (Full Pipeline Succeeded!) ॥\n");
    printf("================================================================\n");

    /* Free all allocated memory */
    free(corpus);
    model_free(&model);
    model_free(&loaded_model);

    return 0;
}