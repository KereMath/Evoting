#include "ffi_wrapper.h"
#include "setup.h"
#include <cstring>
#include <cstdlib>

extern "C" {

CryptoParams* setup_crypto_params(int security_level) {
    // Generate PBC parameters first and save them before initializing pairing
    pbc_param_t par;
    pbc_param_init_a_gen(par, 256, 512);

    // Convert parameters to string before creating pairing
    char pairing_str[8192];
    memset(pairing_str, 0, sizeof(pairing_str)); // Initialize to zeros
    FILE* pairing_mem = fmemopen(pairing_str, sizeof(pairing_str) - 1, "w"); // Leave room for null terminator
    char* pairing_params_str = NULL;
    if (pairing_mem) {
        pbc_param_out_str(pairing_mem, par);
        fclose(pairing_mem);
        // Ensure null termination
        pairing_str[sizeof(pairing_str) - 1] = '\0';
        pairing_params_str = strdup(pairing_str);
    }

    // Now initialize the params using the pbc_param
    TIACParams params;
    mpz_init(params.prime_order);
    pairing_init_pbc_param(params.pairing, par);
    mpz_set(params.prime_order, params.pairing->r);
    element_init_G1(params.g1, params.pairing);
    element_init_G1(params.h1, params.pairing);
    element_init_G2(params.g2, params.pairing);
    element_random(params.g1);
    element_random(params.h1);
    element_random(params.g2);
    pbc_param_clear(par);

    CryptoParams* result = (CryptoParams*)malloc(sizeof(CryptoParams));

    // Convert prime_order to hex string
    char* prime_str = mpz_get_str(NULL, 16, params.prime_order);
    result->prime_order = strdup(prime_str);
    free(prime_str);

    // Convert g1 to string
    int g1_len = element_length_in_bytes(params.g1);
    unsigned char* g1_bytes = (unsigned char*)malloc(g1_len);
    element_to_bytes(g1_bytes, params.g1);
    result->g1 = (char*)malloc(g1_len * 2 + 1);
    for(int i = 0; i < g1_len; i++) {
        sprintf(result->g1 + i*2, "%02x", g1_bytes[i]);
    }
    free(g1_bytes);

    // Convert g2 to string
    int g2_len = element_length_in_bytes(params.g2);
    unsigned char* g2_bytes = (unsigned char*)malloc(g2_len);
    element_to_bytes(g2_bytes, params.g2);
    result->g2 = (char*)malloc(g2_len * 2 + 1);
    for(int i = 0; i < g2_len; i++) {
        sprintf(result->g2 + i*2, "%02x", g2_bytes[i]);
    }
    free(g2_bytes);

    // Convert h1 to string
    int h1_len = element_length_in_bytes(params.h1);
    unsigned char* h1_bytes = (unsigned char*)malloc(h1_len);
    element_to_bytes(h1_bytes, params.h1);
    result->h1 = (char*)malloc(h1_len * 2 + 1);
    for(int i = 0; i < h1_len; i++) {
        sprintf(result->h1 + i*2, "%02x", h1_bytes[i]);
    }
    free(h1_bytes);

    // Use the pairing params string we saved earlier
    if (pairing_params_str) {
        result->pairing_params = pairing_params_str;
    } else {
        // Fallback to a default if fmemopen failed
        result->pairing_params = strdup("type a\nq 8780710799663312522437781984754049815806883199414208211028653399266475630880222957078625179422662221423155858769582317459277713367317481324925129998224791\nh 12016012264891146079388821366740534204802954401251311822919615131047207289359704531102844802183906537786776\nr 730750818665451621361119245571504901405976559617\nexp2 159\nexp1 107\nsign1 1\nsign0 1");
    }

    result->security_level = security_level;

    clearParams(params);

    return result;
}

void free_crypto_params(CryptoParams* params) {
    if (params) {
        free(params->prime_order);
        free(params->g1);
        free(params->g2);
        free(params->h1);
        free(params->pairing_params);
        free(params);
    }
}

}
