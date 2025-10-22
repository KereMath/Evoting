#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <pbc/pbc.h>
#include <gmp.h>
#include <emscripten.h>
#include "sha512.h"

// Global pairing parameters
static pairing_t pairing;
static element_t g1, g2, h1;
static mpz_t prime_order;
static int initialized = 0;

// KoR Proof structure
typedef struct {
    char *c;
    char *s1;
    char *s2;
    char *s3;
} KoRProof;

// PrepareBlindSign output structure
typedef struct {
    char *com;
    char *com_i;
    char *h;
    KoRProof proof;
    char *o_value;
} PrepareBlindSignOutput;

// Helper: element to hex string
static char* element_to_hex(element_t elem) {
    int len = element_length_in_bytes(elem);
    unsigned char *buf = (unsigned char*)malloc(len);
    element_to_bytes(buf, elem);

    char *hex = (char*)malloc(len * 2 + 1);
    for (int i = 0; i < len; i++) {
        sprintf(hex + i * 2, "%02x", buf[i]);
    }
    hex[len * 2] = '\0';

    free(buf);
    return hex;
}

// Helper: mpz to hex string
static char* mpz_to_hex(mpz_t num) {
    char *hex = mpz_get_str(NULL, 16, num);
    return hex;
}

// Helper: hex string to mpz
static void hex_to_mpz(mpz_t rop, const char *hex_str, mpz_t modulus) {
    if (mpz_set_str(rop, hex_str, 16) != 0) {
        fprintf(stderr, "Error: Invalid hex string\n");
        mpz_set_ui(rop, 0);
        return;
    }
    mpz_mod(rop, rop, modulus);
}

// Helper: Hash to G1
static void hash_to_g1(element_t out, element_t in) {
    char *hex = element_to_hex(in);
    element_from_hash(out, hex, strlen(hex));
    free(hex);
}

// Helper: Hash to Zr
static void hash_to_zr(element_t out, const char **strings, int count) {
    // Concatenate all strings
    size_t total_len = 0;
    for (int i = 0; i < count; i++) {
        total_len += strlen(strings[i]);
    }

    char *concat = (char*)malloc(total_len + 1);
    concat[0] = '\0';
    for (int i = 0; i < count; i++) {
        strcat(concat, strings[i]);
    }

    // SHA-512 hash
    unsigned char digest[SHA512_DIGEST_LENGTH];
    SHA512((unsigned char*)concat, strlen(concat), digest);

    // Convert to Zr
    mpz_t tmp;
    mpz_init(tmp);
    mpz_import(tmp, SHA512_DIGEST_LENGTH, 1, 1, 0, 0, digest);
    mpz_mod(tmp, tmp, prime_order);
    element_set_mpz(out, tmp);

    mpz_clear(tmp);
    free(concat);
}

// Compute KoR Proof (Algorithm 5)
static KoRProof compute_kor(
    element_t com,
    element_t com_i,
    element_t g1_elem,
    element_t h1_elem,
    element_t h_elem,
    mpz_t o_i,
    mpz_t did,
    mpz_t o
) {
    KoRProof proof;

    // Generate random r1, r2, r3 ∈ Zr
    element_t r1, r2, r3;
    element_init_Zr(r1, pairing);
    element_init_Zr(r2, pairing);
    element_init_Zr(r3, pairing);
    element_random(r1);
    element_random(r2);
    element_random(r3);

    // com_i' = g1^r1 · h1^r2
    element_t com_i_prime, g1_r1, h1_r2;
    element_init_G1(com_i_prime, pairing);
    element_init_G1(g1_r1, pairing);
    element_init_G1(h1_r2, pairing);
    element_pow_zn(g1_r1, g1_elem, r1);
    element_pow_zn(h1_r2, h1_elem, r2);
    element_mul(com_i_prime, g1_r1, h1_r2);

    // com' = g1^r3 · h^r2
    element_t com_prime, g1_r3, h_r2;
    element_init_G1(com_prime, pairing);
    element_init_G1(g1_r3, pairing);
    element_init_G1(h_r2, pairing);
    element_pow_zn(g1_r3, g1_elem, r3);
    element_pow_zn(h_r2, h_elem, r2);
    element_mul(com_prime, g1_r3, h_r2);

    // c = Hash(g1, h, h1, com, com', com_i, com_i')
    const char *hash_inputs[7];
    hash_inputs[0] = element_to_hex(g1_elem);
    hash_inputs[1] = element_to_hex(h_elem);
    hash_inputs[2] = element_to_hex(h1_elem);
    hash_inputs[3] = element_to_hex(com);
    hash_inputs[4] = element_to_hex(com_prime);
    hash_inputs[5] = element_to_hex(com_i);
    hash_inputs[6] = element_to_hex(com_i_prime);

    element_t c;
    element_init_Zr(c, pairing);
    hash_to_zr(c, hash_inputs, 7);

    // s1 = r1 - c·o_i (mod q)
    mpz_t c_mpz, r1_mpz, r2_mpz, r3_mpz, s1_mpz, s2_mpz, s3_mpz;
    mpz_inits(c_mpz, r1_mpz, r2_mpz, r3_mpz, s1_mpz, s2_mpz, s3_mpz, NULL);

    element_to_mpz(c_mpz, c);
    element_to_mpz(r1_mpz, r1);
    element_to_mpz(r2_mpz, r2);
    element_to_mpz(r3_mpz, r3);

    mpz_mul(s1_mpz, c_mpz, o_i);
    mpz_sub(s1_mpz, r1_mpz, s1_mpz);
    mpz_mod(s1_mpz, s1_mpz, prime_order);

    // s2 = r2 - c·did (mod q)
    mpz_mul(s2_mpz, c_mpz, did);
    mpz_sub(s2_mpz, r2_mpz, s2_mpz);
    mpz_mod(s2_mpz, s2_mpz, prime_order);

    // s3 = r3 - c·o (mod q)
    mpz_mul(s3_mpz, c_mpz, o);
    mpz_sub(s3_mpz, r3_mpz, s3_mpz);
    mpz_mod(s3_mpz, s3_mpz, prime_order);

    // Convert to hex strings
    proof.c = mpz_to_hex(c_mpz);
    proof.s1 = mpz_to_hex(s1_mpz);
    proof.s2 = mpz_to_hex(s2_mpz);
    proof.s3 = mpz_to_hex(s3_mpz);

    // Cleanup
    for (int i = 0; i < 7; i++) {
        free((void*)hash_inputs[i]);
    }
    mpz_clears(c_mpz, r1_mpz, r2_mpz, r3_mpz, s1_mpz, s2_mpz, s3_mpz, NULL);
    element_clear(r1);
    element_clear(r2);
    element_clear(r3);
    element_clear(c);
    element_clear(com_i_prime);
    element_clear(com_prime);
    element_clear(g1_r1);
    element_clear(h1_r2);
    element_clear(g1_r3);
    element_clear(h_r2);

    return proof;
}

// Helper: hex string to element bytes
static int hex_to_bytes(const char *hex, unsigned char **out, int *out_len) {
    int hex_len = strlen(hex);
    if (hex_len % 2 != 0) {
        return -1; // Invalid hex string
    }

    *out_len = hex_len / 2;
    *out = (unsigned char*)malloc(*out_len);

    for (int i = 0; i < *out_len; i++) {
        char byte_str[3] = {hex[i*2], hex[i*2+1], '\0'};
        (*out)[i] = (unsigned char)strtol(byte_str, NULL, 16);
    }

    return 0;
}

// Initialize pairing with crypto parameters from backend
EMSCRIPTEN_KEEPALIVE
int init_pairing(const char *pairing_params_str, const char *prime_order_hex, const char *g1_hex, const char *g2_hex, const char *h1_hex) {
    if (initialized) {
        return 1; // Already initialized
    }

    // Initialize pairing from pairing_params string (from backend PBC setup)
    pairing_init_set_buf(pairing, pairing_params_str, strlen(pairing_params_str));

    // Initialize prime order from hex
    mpz_init(prime_order);
    if (mpz_set_str(prime_order, prime_order_hex, 16) != 0) {
        fprintf(stderr, "Error: Invalid prime order hex\n");
        return -1;
    }

    // Initialize generators from hex strings (from backend)
    element_init_G1(g1, pairing);
    element_init_G2(g2, pairing);
    element_init_G1(h1, pairing);

    // Convert g1 hex to element
    unsigned char *g1_bytes;
    int g1_len;
    if (hex_to_bytes(g1_hex, &g1_bytes, &g1_len) != 0) {
        fprintf(stderr, "Error: Invalid g1 hex\n");
        return -1;
    }
    if (element_from_bytes(g1, g1_bytes) != g1_len) {
        fprintf(stderr, "Warning: g1 length mismatch\n");
    }
    free(g1_bytes);

    // Convert g2 hex to element
    unsigned char *g2_bytes;
    int g2_len;
    if (hex_to_bytes(g2_hex, &g2_bytes, &g2_len) != 0) {
        fprintf(stderr, "Error: Invalid g2 hex\n");
        return -1;
    }
    if (element_from_bytes(g2, g2_bytes) != g2_len) {
        fprintf(stderr, "Warning: g2 length mismatch\n");
    }
    free(g2_bytes);

    // Convert h1 hex to element
    unsigned char *h1_bytes;
    int h1_len;
    if (hex_to_bytes(h1_hex, &h1_bytes, &h1_len) != 0) {
        fprintf(stderr, "Error: Invalid h1 hex\n");
        return -1;
    }
    if (element_from_bytes(h1, h1_bytes) != h1_len) {
        fprintf(stderr, "Warning: h1 length mismatch\n");
    }
    free(h1_bytes);

    initialized = 1;
    return 0;
}

// PrepareBlindSign (Algorithm 4)
EMSCRIPTEN_KEEPALIVE
char* prepare_blind_sign(const char *did_hex, const char *o_hex) {
    if (!initialized) {
        return strdup("{\"error\":\"Pairing not initialized\"}");
    }

    // Parse DID and o from hex
    mpz_t did, o, o_i;
    mpz_inits(did, o, o_i, NULL);

    hex_to_mpz(did, did_hex, prime_order);
    hex_to_mpz(o, o_hex, prime_order);

    // Generate random o_i ∈ Zr
    element_t o_i_elem;
    element_init_Zr(o_i_elem, pairing);
    element_random(o_i_elem);
    element_to_mpz(o_i, o_i_elem);
    element_clear(o_i_elem);

    // Compute com_i = g1^o_i · h1^did
    element_t com_i, g1_oi, h1_did, exp;
    element_init_G1(com_i, pairing);
    element_init_G1(g1_oi, pairing);
    element_init_G1(h1_did, pairing);
    element_init_Zr(exp, pairing);

    element_set_mpz(exp, o_i);
    element_pow_zn(g1_oi, g1, exp);

    element_set_mpz(exp, did);
    element_pow_zn(h1_did, h1, exp);

    element_mul(com_i, g1_oi, h1_did);

    // Compute h = Hash(com_i)
    element_t h;
    element_init_G1(h, pairing);
    hash_to_g1(h, com_i);

    // Compute com = g1^o · h^did
    element_t com, g1_o, h_did;
    element_init_G1(com, pairing);
    element_init_G1(g1_o, pairing);
    element_init_G1(h_did, pairing);

    element_set_mpz(exp, o);
    element_pow_zn(g1_o, g1, exp);

    element_set_mpz(exp, did);
    element_pow_zn(h_did, h, exp);

    element_mul(com, g1_o, h_did);

    // Compute KoR proof π_s
    KoRProof proof = compute_kor(com, com_i, g1, h1, h, o_i, did, o);

    // Convert to hex strings
    char *com_hex = element_to_hex(com);
    char *com_i_hex = element_to_hex(com_i);
    char *h_hex = element_to_hex(h);
    char *o_hex_out = mpz_to_hex(o);

    // Build JSON output
    char *json_output = (char*)malloc(8192);
    snprintf(json_output, 8192,
        "{"
        "\"com\":\"%s\","
        "\"com_i\":\"%s\","
        "\"h\":\"%s\","
        "\"proof\":{"
        "\"c\":\"%s\","
        "\"s1\":\"%s\","
        "\"s2\":\"%s\","
        "\"s3\":\"%s\""
        "},"
        "\"o\":\"%s\""
        "}",
        com_hex, com_i_hex, h_hex,
        proof.c, proof.s1, proof.s2, proof.s3,
        o_hex_out
    );

    // Cleanup
    free(com_hex);
    free(com_i_hex);
    free(h_hex);
    free(o_hex_out);
    free(proof.c);
    free(proof.s1);
    free(proof.s2);
    free(proof.s3);
    mpz_clears(did, o, o_i, NULL);
    element_clear(com_i);
    element_clear(h);
    element_clear(com);
    element_clear(g1_oi);
    element_clear(h1_did);
    element_clear(g1_o);
    element_clear(h_did);
    element_clear(exp);

    return json_output;
}

// Cleanup
EMSCRIPTEN_KEEPALIVE
void cleanup_pairing() {
    if (initialized) {
        element_clear(g1);
        element_clear(g2);
        element_clear(h1);
        mpz_clear(prime_order);
        pairing_clear(pairing);
        initialized = 0;
    }
}
