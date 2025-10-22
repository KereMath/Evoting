#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <gmp.h>
#include <emscripten.h>

// Simple PrepareBlindSign implementation using GMP
// This takes backend crypto parameters and performs client-side computation

// Helper: Generate random number in range [0, max)
static void random_mpz(mpz_t rop, gmp_randstate_t state, const mpz_t max) {
    mpz_urandomm(rop, state, max);
}

// Helper: Parse hex string to mpz
static int hex_to_mpz(mpz_t rop, const char *hex) {
    return mpz_set_str(rop, hex, 16);
}

// Helper: Convert mpz to hex string
static char* mpz_to_hex(const mpz_t num) {
    return mpz_get_str(NULL, 16, num);
}

// PrepareBlindSign - Simplified version
// Takes: DID (hex), o-value (hex), backend crypto params
// Returns: JSON with commitments and proof
EMSCRIPTEN_KEEPALIVE
char* prepare_blind_sign_simple(
    const char *did_hex,
    const char *o_hex,
    const char *prime_order_hex,
    const char *g1_hex,
    const char *h1_hex
) {
    // Initialize GMP random state
    gmp_randstate_t rand_state;
    gmp_randinit_default(rand_state);
    gmp_randseed_ui(rand_state, (unsigned long)time(NULL));

    // Parse inputs
    mpz_t did, o, prime_order, g1_base, h1_base;
    mpz_inits(did, o, prime_order, g1_base, h1_base, NULL);

    if (hex_to_mpz(did, did_hex) != 0 ||
        hex_to_mpz(o, o_hex) != 0 ||
        hex_to_mpz(prime_order, prime_order_hex) != 0 ||
        hex_to_mpz(g1_base, g1_hex) != 0 ||
        hex_to_mpz(h1_base, h1_hex) != 0) {
        mpz_clears(did, o, prime_order, g1_base, h1_base, NULL);
        gmp_randclear(rand_state);
        return strdup("{\"error\":\"Invalid hex input\"}");
    }

    // Reduce inputs modulo prime_order
    mpz_mod(did, did, prime_order);
    mpz_mod(o, o, prime_order);

    // Generate random o_i for blinding
    mpz_t o_i;
    mpz_init(o_i);
    random_mpz(o_i, rand_state, prime_order);

    // Compute com_i = g1^{o_i} * h1^{did} (mod prime_order)
    // Using multiplicative group operations
    mpz_t com_i, temp1, temp2;
    mpz_inits(com_i, temp1, temp2, NULL);

    mpz_powm(temp1, g1_base, o_i, prime_order);  // g1^{o_i}
    mpz_powm(temp2, h1_base, did, prime_order);   // h1^{did}
    mpz_mul(com_i, temp1, temp2);                 // temp1 * temp2
    mpz_mod(com_i, com_i, prime_order);           // mod prime_order

    // Compute h = Hash(com_i) - simplified: just use com_i as base for h
    // In real implementation, this would be hash-to-curve
    mpz_t h_base;
    mpz_init(h_base);
    mpz_set(h_base, com_i);  // Simplified: h = com_i (deterministic)

    // Compute com = g1^{o} * h^{did} (mod prime_order)
    mpz_t com;
    mpz_init(com);
    mpz_powm(temp1, g1_base, o, prime_order);     // g1^{o}
    mpz_powm(temp2, h_base, did, prime_order);    // h^{did}
    mpz_mul(com, temp1, temp2);                   // temp1 * temp2
    mpz_mod(com, com, prime_order);               // mod prime_order

    // Generate KoR proof
    // Generate random r1, r2, r3
    mpz_t r1, r2, r3;
    mpz_inits(r1, r2, r3, NULL);
    random_mpz(r1, rand_state, prime_order);
    random_mpz(r2, rand_state, prime_order);
    random_mpz(r3, rand_state, prime_order);

    // Compute com_i' = g1^{r1} * h1^{r2}
    mpz_t com_i_prime;
    mpz_init(com_i_prime);
    mpz_powm(temp1, g1_base, r1, prime_order);
    mpz_powm(temp2, h1_base, r2, prime_order);
    mpz_mul(com_i_prime, temp1, temp2);
    mpz_mod(com_i_prime, com_i_prime, prime_order);

    // Compute com' = g1^{r3} * h^{r2}
    mpz_t com_prime;
    mpz_init(com_prime);
    mpz_powm(temp1, g1_base, r3, prime_order);
    mpz_powm(temp2, h_base, r2, prime_order);
    mpz_mul(com_prime, temp1, temp2);
    mpz_mod(com_prime, com_prime, prime_order);

    // Compute challenge c = Hash(g1, h, h1, com, com', com_i, com_i')
    // Simplified: c = (com + com' + com_i + com_i') mod prime_order
    mpz_t c;
    mpz_init(c);
    mpz_add(c, com, com_prime);
    mpz_add(c, c, com_i);
    mpz_add(c, c, com_i_prime);
    mpz_mod(c, c, prime_order);

    // Compute responses
    // s1 = r1 - c*o_i (mod prime_order)
    mpz_t s1, s2, s3;
    mpz_inits(s1, s2, s3, NULL);
    mpz_mul(temp1, c, o_i);
    mpz_sub(s1, r1, temp1);
    mpz_mod(s1, s1, prime_order);

    // s2 = r2 - c*did (mod prime_order)
    mpz_mul(temp1, c, did);
    mpz_sub(s2, r2, temp1);
    mpz_mod(s2, s2, prime_order);

    // s3 = r3 - c*o (mod prime_order)
    mpz_mul(temp1, c, o);
    mpz_sub(s3, r3, temp1);
    mpz_mod(s3, s3, prime_order);

    // Convert to hex strings
    char *com_hex = mpz_to_hex(com);
    char *com_i_hex = mpz_to_hex(com_i);
    char *h_hex = mpz_to_hex(h_base);
    char *c_hex = mpz_to_hex(c);
    char *s1_hex = mpz_to_hex(s1);
    char *s2_hex = mpz_to_hex(s2);
    char *s3_hex = mpz_to_hex(s3);
    char *o_out_hex = mpz_to_hex(o);

    // Build JSON response
    char *result = (char*)malloc(16384);
    snprintf(result, 16384,
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
        c_hex, s1_hex, s2_hex, s3_hex,
        o_out_hex
    );

    // Cleanup
    free(com_hex);
    free(com_i_hex);
    free(h_hex);
    free(c_hex);
    free(s1_hex);
    free(s2_hex);
    free(s3_hex);
    free(o_out_hex);

    mpz_clears(did, o, prime_order, g1_base, h1_base, o_i, NULL);
    mpz_clears(com_i, temp1, temp2, h_base, com, NULL);
    mpz_clears(r1, r2, r3, com_i_prime, com_prime, c, NULL);
    mpz_clears(s1, s2, s3, NULL);
    gmp_randclear(rand_state);

    return result;
}
