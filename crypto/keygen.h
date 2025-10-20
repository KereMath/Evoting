#ifndef KEYGEN_H
#define KEYGEN_H

#include "setup.h"
#include <vector>
#include <gmp.h>

// Master Verification Key (mvk)
struct MasterVerKey {
    element_t alpha2;  // vk_1 = g2^x
    element_t beta2;   // vk_2 = g2^y
    element_t beta1;   // vk_3 = g1^y
};

// EA Key (Signing Key + Verification Key for each EA)
struct EAKey {
    // Signing keys (private)
    element_t sgk1;    // sk_i1 = Σ F_l(i)
    element_t sgk2;    // sk_i2 = Σ G_l(i)

    // Verification keys (public)
    element_t vkm1;    // vk_i1 = g2^(F(i))
    element_t vkm2;    // vk_i2 = g2^(G(i))
    element_t vkm3;    // vk_i3 = g1^(G(i))
};

// KeyGen Output
struct KeyGenOutput {
    MasterVerKey mvk;
    std::vector<EAKey> eaKeys;
};

// === Pedersen DKG Internal Structures ===

// Note: We use raw pointers because mpz_t and element_t are arrays (typedef)
// and cannot be stored directly in std::vector

// Polynomial coefficients for one EA
struct EAPolynomials {
    mpz_t* F_coeffs;  // F_i[X] = x_i0 + x_i1*X + ... + x_it*X^t
    mpz_t* G_coeffs;  // G_i[X] = y_i0 + y_i1*X + ... + y_it*X^t
    int size;

    EAPolynomials() : F_coeffs(nullptr), G_coeffs(nullptr), size(0) {}
};

// Commitments for polynomial coefficients
struct EACommitments {
    element_t* V_x;   // V_xij = g2^(x_ij) for j in [0,t]
    element_t* V_y;   // V_yij = g2^(y_ij) for j in [0,t]
    element_t* V_y_prime;  // V'_yij = g1^(y_ij) for j in [0,t]
    int size;

    EACommitments() : V_x(nullptr), V_y(nullptr), V_y_prime(nullptr), size(0) {}
};

// Share sent from EA_l to EA_i
struct Share {
    mpz_t F_l_i;  // F_l(i)
    mpz_t G_l_i;  // G_l(i)
};

// === Pedersen DKG Functions (for distributed key generation) ===

// Generate random polynomial of degree t (pointer-based for DKG CLI)
void randomPolynomial_ptr(mpz_t* poly, int t, const mpz_t p);

// Evaluate polynomial at point x: poly(x) (pointer-based for DKG CLI)
void evalPolynomial_ptr(mpz_t result, mpz_t* poly, int poly_size, int xValue, const mpz_t p);

// Generate Pedersen commitments for polynomial coefficients
void generateCommitments(EACommitments &commitments,
                        const EAPolynomials &polynomials,
                        TIACParams &params);

// Verify share using commitments: g2^F_l(i) == ∏(V_xlj^(i^j))
bool verifyShare(Share &share,
                const EACommitments &commitments,
                int i,  // EA index receiving the share
                TIACParams &params);

#endif
