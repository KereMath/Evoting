#include "keygen.h"
#include <iostream>
#include <vector>
#include <random>
#include <stdexcept>
#include <set>

// ==================== HELPER FUNCTIONS ====================

static void random_mpz_modp(mpz_t rop, const mpz_t p) {
    static std::random_device rd;
    static std::mt19937_64 gen(rd());
    size_t bits = mpz_sizeinbase(p, 2);
    size_t bytes = (bits + 7) / 8;
    std::vector<unsigned char> buf(bytes);
    for (size_t i = 0; i < bytes; i++) {
        buf[i] = static_cast<unsigned char>(gen() & 0xFF);
    }
    mpz_import(rop, bytes, 1, 1, 0, 0, buf.data());
    mpz_mod(rop, rop, p);
}

// Pointer-based polynomial generation (used by DKG)
void randomPolynomial_ptr(mpz_t* poly, int t, const mpz_t p) {
    for (int i = 0; i < t; i++) {
        mpz_init(poly[i]);
        random_mpz_modp(poly[i], p);
    }
}

// Pointer-based polynomial evaluation (used by DKG)
void evalPolynomial_ptr(mpz_t result, mpz_t* poly, int poly_size, int xValue, const mpz_t p) {
    mpz_set_ui(result, 0);
    mpz_t term;
    mpz_init(term);
    mpz_t xPow;
    mpz_init_set_ui(xPow, 1);
    for (int k = 0; k < poly_size; k++) {
        mpz_mul(term, poly[k], xPow);
        mpz_add(result, result, term);
        mpz_mod(result, result, p);
        mpz_mul_ui(xPow, xPow, xValue);
        mpz_mod(xPow, xPow, p);
    }
    mpz_clear(term);
    mpz_clear(xPow);
}

// ==================== PEDERSEN DKG IMPLEMENTATION ====================

void generateCommitments(EACommitments &commitments,
                        const EAPolynomials &polynomials,
                        TIACParams &params) {
    int t = polynomials.size;

    commitments.size = t;
    commitments.V_x = new element_t[t];
    commitments.V_y = new element_t[t];
    commitments.V_y_prime = new element_t[t];

    for (int j = 0; j < t; j++) {
        // V_xij = g2^(x_ij)
        element_init_G2(commitments.V_x[j], params.pairing);
        element_t exp_xij;
        element_init_Zr(exp_xij, params.pairing);
        element_set_mpz(exp_xij, polynomials.F_coeffs[j]);
        element_pow_zn(commitments.V_x[j], params.g2, exp_xij);
        element_clear(exp_xij);

        // V_yij = g2^(y_ij)
        element_init_G2(commitments.V_y[j], params.pairing);
        element_t exp_yij;
        element_init_Zr(exp_yij, params.pairing);
        element_set_mpz(exp_yij, polynomials.G_coeffs[j]);
        element_pow_zn(commitments.V_y[j], params.g2, exp_yij);
        element_clear(exp_yij);

        // V'_yij = g1^(y_ij)
        element_init_G1(commitments.V_y_prime[j], params.pairing);
        element_t exp_yij_prime;
        element_init_Zr(exp_yij_prime, params.pairing);
        element_set_mpz(exp_yij_prime, polynomials.G_coeffs[j]);
        element_pow_zn(commitments.V_y_prime[j], params.g1, exp_yij_prime);
        element_clear(exp_yij_prime);
    }
}

bool verifyShare(Share &share,
                const EACommitments &commitments,
                int i,
                TIACParams &params) {
    int t = commitments.size;

    // Verify: g2^F_l(i) == ∏(j=0 to t-1) V_xlj^(i^j)
    element_t lhs_F, rhs_F;
    element_init_G2(lhs_F, params.pairing);
    element_init_G2(rhs_F, params.pairing);

    // LHS: g2^F_l(i)
    element_t exp_F;
    element_init_Zr(exp_F, params.pairing);
    element_set_mpz(exp_F, share.F_l_i);
    element_pow_zn(lhs_F, params.g2, exp_F);
    element_clear(exp_F);

    // RHS: ∏(j=0 to t-1) V_xlj^(i^j)
    element_set1(rhs_F);  // Identity element
    for (int j = 0; j < t; j++) {
        mpz_t i_pow_j;
        mpz_init(i_pow_j);
        mpz_set_ui(i_pow_j, i);
        mpz_pow_ui(i_pow_j, i_pow_j, j);  // i^j

        element_t term;
        element_init_G2(term, params.pairing);
        element_pow_mpz(term, commitments.V_x[j], i_pow_j);  // V_xlj^(i^j)
        element_mul(rhs_F, rhs_F, term);

        element_clear(term);
        mpz_clear(i_pow_j);
    }

    bool valid_F = (element_cmp(lhs_F, rhs_F) == 0);
    element_clear(lhs_F);
    element_clear(rhs_F);

    // Verify: g2^G_l(i) == ∏(j=0 to t-1) V_ylj^(i^j)
    element_t lhs_G, rhs_G;
    element_init_G2(lhs_G, params.pairing);
    element_init_G2(rhs_G, params.pairing);

    // LHS: g2^G_l(i)
    element_t exp_G;
    element_init_Zr(exp_G, params.pairing);
    element_set_mpz(exp_G, share.G_l_i);
    element_pow_zn(lhs_G, params.g2, exp_G);
    element_clear(exp_G);

    // RHS: ∏(j=0 to t-1) V_ylj^(i^j)
    element_set1(rhs_G);
    for (int j = 0; j < t; j++) {
        mpz_t i_pow_j;
        mpz_init(i_pow_j);
        mpz_set_ui(i_pow_j, i);
        mpz_pow_ui(i_pow_j, i_pow_j, j);

        element_t term;
        element_init_G2(term, params.pairing);
        element_pow_mpz(term, commitments.V_y[j], i_pow_j);
        element_mul(rhs_G, rhs_G, term);

        element_clear(term);
        mpz_clear(i_pow_j);
    }

    bool valid_G = (element_cmp(lhs_G, rhs_G) == 0);
    element_clear(lhs_G);
    element_clear(rhs_G);

    // Verify: g1^G_l(i) == ∏(j=0 to t-1) V'_ylj^(i^j)
    element_t lhs_G_prime, rhs_G_prime;
    element_init_G1(lhs_G_prime, params.pairing);
    element_init_G1(rhs_G_prime, params.pairing);

    // LHS: g1^G_l(i)
    element_t exp_G_prime;
    element_init_Zr(exp_G_prime, params.pairing);
    element_set_mpz(exp_G_prime, share.G_l_i);
    element_pow_zn(lhs_G_prime, params.g1, exp_G_prime);
    element_clear(exp_G_prime);

    // RHS: ∏(j=0 to t-1) V'_ylj^(i^j)
    element_set1(rhs_G_prime);
    for (int j = 0; j < t; j++) {
        mpz_t i_pow_j;
        mpz_init(i_pow_j);
        mpz_set_ui(i_pow_j, i);
        mpz_pow_ui(i_pow_j, i_pow_j, j);

        element_t term;
        element_init_G1(term, params.pairing);
        element_pow_mpz(term, commitments.V_y_prime[j], i_pow_j);
        element_mul(rhs_G_prime, rhs_G_prime, term);

        element_clear(term);
        mpz_clear(i_pow_j);
    }

    bool valid_G_prime = (element_cmp(lhs_G_prime, rhs_G_prime) == 0);
    element_clear(lhs_G_prime);
    element_clear(rhs_G_prime);

    return valid_F && valid_G && valid_G_prime;
}

