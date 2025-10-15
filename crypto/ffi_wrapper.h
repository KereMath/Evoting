#ifndef FFI_WRAPPER_H
#define FFI_WRAPPER_H

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    char* prime_order;
    char* g1;
    char* g2;
    char* h1;
    char* pairing_params;
    int security_level;
} CryptoParams;

CryptoParams* setup_crypto_params(int security_level);
void free_crypto_params(CryptoParams* params);

#ifdef __cplusplus
}
#endif

#endif
