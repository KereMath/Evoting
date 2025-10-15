#include "prepareblindsign.h"
#include <openssl/sha.h>
#include <vector>
#include <random>
#include <sstream>
#include <iomanip>
#include <stdexcept>
#include <iostream>

static void randomZr(element_t zr, TIACParams &params) {
    element_random(zr);  
}

static void didStringToMpz(const std::string &didStr, mpz_t rop, const mpz_t p) {
    if(mpz_set_str(rop, didStr.c_str(), 16) != 0) {
        throw std::runtime_error("didStringToMpz: invalid hex string");
    }
    mpz_mod(rop, rop, p);
}

static std::string elementToStringG1(element_t elem) {
    int length = element_length_in_bytes(elem);
    std::vector<unsigned char> buf(length);
    element_to_bytes(buf.data(), elem);
    std::ostringstream oss;
    oss << std::hex << std::setfill('0');
    for (auto c : buf) {
        oss << std::setw(2) << (int)c;
    }
    return oss.str();
}

static void hashToG1(element_t outG1, TIACParams &params, element_t inElem) {
    std::string s = elementToStringG1(inElem); 
    element_from_hash(outG1, s.data(), s.size());
}

static void hashToZr(element_t outZr, TIACParams &params, const std::vector<std::string> &elems) {
    std::ostringstream oss;
    for (const auto &s : elems) {
        oss << s;
    }
    std::string msg = oss.str();
    unsigned char digest[SHA512_DIGEST_LENGTH];
    SHA512(reinterpret_cast<const unsigned char*>(msg.data()), msg.size(), digest);

    mpz_t tmp;
    mpz_init(tmp);
    mpz_import(tmp, SHA512_DIGEST_LENGTH, 1, 1, 0, 0, digest);
    mpz_mod(tmp, tmp, params.prime_order);
    element_set_mpz(outZr, tmp);
    mpz_clear(tmp);
}
static KoRProof computeKoR(
    TIACParams &params,
    element_t com,   
    element_t comi,  
    element_t g1,   
    element_t h1, 
    element_t h,   
    mpz_t oi,        
    mpz_t did,
    mpz_t o
) {
    KoRProof proof;

    element_t r1, r2, r3;
    element_init_Zr(r1, params.pairing);
    element_init_Zr(r2, params.pairing);
    element_init_Zr(r3, params.pairing);
    randomZr(r1, params);
    randomZr(r2, params);
    randomZr(r3, params);

    element_t comi_prime;
    element_init_G1(comi_prime, params.pairing);
    element_t g1_r1, h1_r2;
    element_init_G1(g1_r1, params.pairing);
    element_init_G1(h1_r2, params.pairing);
    element_pow_zn(g1_r1, g1, r1);
    element_pow_zn(h1_r2, h1, r2);
    element_mul(comi_prime, g1_r1, h1_r2);
    element_clear(g1_r1);
    element_clear(h1_r2);

    element_t com_prime;
    element_init_G1(com_prime, params.pairing);
    element_t g1_r3, h_r2;
    element_init_G1(g1_r3, params.pairing);
    element_init_G1(h_r2, params.pairing);
    element_pow_zn(g1_r3, g1, r3);
    element_pow_zn(h_r2, h, r2);
    element_mul(com_prime, g1_r3, h_r2);
    element_clear(g1_r3);
    element_clear(h_r2);

    element_init_Zr(proof.c,  params.pairing);
    element_init_Zr(proof.s1, params.pairing);
    element_init_Zr(proof.s2, params.pairing);
    element_init_Zr(proof.s3, params.pairing);

    std::vector<std::string> toHash;
    toHash.reserve(7);
    toHash.push_back(elementToStringG1(g1));
    toHash.push_back(elementToStringG1(h));
    toHash.push_back(elementToStringG1(h1));
    toHash.push_back(elementToStringG1(com));
    toHash.push_back(elementToStringG1(com_prime));
    toHash.push_back(elementToStringG1(comi));
    toHash.push_back(elementToStringG1(comi_prime));
    hashToZr(proof.c, params, toHash);

    mpz_t c_mpz;
    mpz_init(c_mpz);
    element_to_mpz(c_mpz, proof.c);
    
    mpz_t r1_mpz, r2_mpz, r3_mpz;
    mpz_inits(r1_mpz, r2_mpz, r3_mpz, NULL);
    element_to_mpz(r1_mpz, r1);
    element_to_mpz(r2_mpz, r2);
    element_to_mpz(r3_mpz, r3);
    
    mpz_t s1_mpz;
    mpz_init(s1_mpz);
    mpz_mul(s1_mpz, c_mpz, oi);
    mpz_sub(s1_mpz, r1_mpz, s1_mpz);
    mpz_mod(s1_mpz, s1_mpz, params.prime_order);
    element_set_mpz(proof.s1, s1_mpz);
    
    mpz_t s2_mpz;
    mpz_init(s2_mpz);
    mpz_mul(s2_mpz, c_mpz, did);
    mpz_sub(s2_mpz, r2_mpz, s2_mpz);
    mpz_mod(s2_mpz, s2_mpz, params.prime_order);
    element_set_mpz(proof.s2, s2_mpz);
    
    mpz_t s3_mpz;
    mpz_init(s3_mpz);
    mpz_mul(s3_mpz, c_mpz, o);
    mpz_sub(s3_mpz, r3_mpz, s3_mpz);
    mpz_mod(s3_mpz, s3_mpz, params.prime_order);
    element_set_mpz(proof.s3, s3_mpz);
    
    mpz_clears(c_mpz, r1_mpz, r2_mpz, r3_mpz, s1_mpz, s2_mpz, s3_mpz, NULL);
    element_clear(r1);
    element_clear(r2);
    element_clear(r3);
    element_clear(com_prime);
    element_clear(comi_prime);
    
    return proof;
}

PrepareBlindSignOutput prepareBlindSign(TIACParams &params, const std::string &didStr) {
    PrepareBlindSignOutput out;
    mpz_t oi, o;
    mpz_inits(oi, o, NULL);
    element_t tmp;
    element_init_Zr(tmp, params.pairing);
    element_random(tmp);
    element_to_mpz(oi, tmp);
    element_random(tmp);
    element_to_mpz(o, tmp);
    element_clear(tmp);
    mpz_t didInt;
    mpz_init(didInt);
    didStringToMpz(didStr, didInt, params.prime_order);
    element_init_G1(out.comi, params.pairing);
    element_t g1_oi, h1_did;
    element_init_G1(g1_oi, params.pairing);
    element_init_G1(h1_did, params.pairing);
    element_t exp;
    element_init_Zr(exp, params.pairing);
    element_set_mpz(exp, oi);
    element_pow_zn(g1_oi, params.g1, exp);
    element_clear(exp);
    element_init_Zr(exp, params.pairing);
    element_set_mpz(exp, didInt);
    element_pow_zn(h1_did, params.h1, exp);
    element_clear(exp);
    element_mul(out.comi, g1_oi, h1_did);
    element_clear(g1_oi);
    element_clear(h1_did);
    element_init_G1(out.h, params.pairing);
    hashToG1(out.h, params, out.comi);
    element_init_G1(out.com, params.pairing);
    element_t g1_o, h_did;
    element_init_G1(g1_o, params.pairing);
    element_init_G1(h_did, params.pairing);
    element_init_Zr(exp, params.pairing);
    element_set_mpz(exp, o);
    element_pow_zn(g1_o, params.g1, exp);
    element_clear(exp);
    element_init_Zr(exp, params.pairing);
    element_set_mpz(exp, didInt);
    element_pow_zn(h_did, out.h, exp);
    element_clear(exp);
    element_mul(out.com, g1_o, h_did);
    element_clear(g1_o);
    element_clear(h_did);
    out.com_str = elementToStringG1(out.com);
    out.pi_s = computeKoR(
        params,
        out.com,
        out.comi,
        params.g1,
        params.h1,
        out.h,
        oi,
        didInt,
        o
    );
    mpz_init(out.o);
    mpz_set(out.o, o);
    mpz_clears(oi, o, didInt, NULL);
    return out;
}