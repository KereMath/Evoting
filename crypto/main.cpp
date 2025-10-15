#include <iostream>
#include <chrono>
#include <fstream>
#include <string>
#include <vector>
#include <random>
#include <algorithm>
#include <memory>
#include <mutex>
#include <thread>
// TBB header'ını kaldırdık
#include "setup.h"
#include "keygen.h"
#include "didgen.h"
#include "prepareblindsign.h"
#include "blindsign.h"  
#include "unblindsign.h"
#include "aggregate.h" 
#include "provecredential.h" 
#include "pairinginverify.h"
#include "checkkorverify.h"
#include "kor.h"
using Clock = std::chrono::steady_clock;

struct PipelineTiming {
    Clock::time_point prep_start;
    Clock::time_point prep_end;
    Clock::time_point blind_start;
    Clock::time_point blind_end;
};

struct PipelineResult {
    std::vector<BlindSignature> signatures;
    PipelineTiming timing;
};

void my_element_dup(element_t dest, element_t src) {
    element_init_same_as(dest, src);
    element_set(dest, src);
}

int main() {
    // Paralellik kontrolünü kaldırdık
    auto programStart = Clock::now();
    int ne = 0;       
    int t  = 0;       
    int voterCount = 0; 
    {
        std::ifstream infile("params.txt");
        if (!infile) {
            std::cerr << "Error: params.txt acilamadi!\n";
            return 1;
        }
        std::string line;
        while (std::getline(infile, line)) {
            if (line.rfind("ea=", 0) == 0)
                ne = std::stoi(line.substr(3));
            else if (line.rfind("threshold=", 0) == 0)
                t = std::stoi(line.substr(10));
            else if (line.rfind("votercount=", 0) == 0)
                voterCount = std::stoi(line.substr(11));
        }
        infile.close();
    }
    
    auto startSetup = Clock::now();
    TIACParams params = setupParams();
    auto endSetup = Clock::now();
    auto setup_us = std::chrono::duration_cast<std::chrono::microseconds>(endSetup - startSetup).count();
    
    element_t pairingTest;
    element_init_GT(pairingTest, params.pairing);
    auto startPairing = Clock::now();
    pairing_apply(pairingTest, params.g1, params.g2, params.pairing);
    auto endPairing = Clock::now();
    auto pairing_us = std::chrono::duration_cast<std::chrono::microseconds>(endPairing - startPairing).count();
    element_clear(pairingTest);
    
    auto startKeygen = Clock::now();
    KeyGenOutput keyOut = keygen(params, t, ne);
    auto endKeygen = Clock::now();
    auto keygen_us = std::chrono::duration_cast<std::chrono::microseconds>(endKeygen - startKeygen).count();
    
    auto startIDGen = Clock::now();
    std::vector<std::string> voterIDs(voterCount);
    {
        std::random_device rd;
        std::mt19937_64 gen(rd());
        std::uniform_int_distribution<unsigned long long> dist(10000000000ULL, 99999999999ULL);
        for (int i = 0; i < voterCount; i++) {
            unsigned long long id = dist(gen);
            voterIDs[i] = std::to_string(id);
        }
    }
    auto endIDGen = Clock::now();
    auto idGen_us = std::chrono::duration_cast<std::chrono::microseconds>(endIDGen - startIDGen).count();
    
    auto startDIDGen = Clock::now();
    std::vector<DID> dids(voterCount);
    for (int i = 0; i < voterCount; i++) {
        dids[i] = createDID(params, voterIDs[i]);
    }
    auto endDIDGen = Clock::now();
    auto didGen_us = std::chrono::duration_cast<std::chrono::microseconds>(endDIDGen - startDIDGen).count();
    
    std::vector<PipelineResult> pipelineResults(voterCount);
    auto pipelineStart = Clock::now();
    
    // Prepare BlindSign işlemleri - sıralı (sequential) çalışır
    auto prepStart = Clock::now();
    std::vector<PrepareBlindSignOutput> preparedOutputs(voterCount);
    for(int i = 0; i < voterCount; i++){
        preparedOutputs[i] = prepareBlindSign(params, dids[i].did);
    }
    auto prepEnd = Clock::now();
    auto prepTime = std::chrono::duration_cast<std::chrono::microseconds>(prepEnd - prepStart).count();
    
    // BlindSign görevlerini hazırla
    struct SignTask {
        int voterId;
        int indexInVoter;
        int adminId;
    };
    std::vector<SignTask> tasks;
    tasks.reserve(voterCount * t);
    std::random_device rd;
    std::mt19937 rng(rd());
    
    for (int i = 0; i < voterCount; i++) {
        pipelineResults[i].signatures.resize(t);
        std::vector<int> adminIndices(ne);
        std::iota(adminIndices.begin(), adminIndices.end(), 0);
        std::shuffle(adminIndices.begin(), adminIndices.end(), rng);
        
        for (int j = 0; j < t; j++) {
            SignTask st;
            st.voterId = i;
            st.indexInVoter = j;
            st.adminId = adminIndices[j]; 
            tasks.push_back(st);
        }
    }
    
    // BlindSign işlemleri - sıralı (sequential) çalışır
    auto blindStart = Clock::now();
    for(int idx = 0; idx < (int)tasks.size(); idx++) {
        const SignTask &st = tasks[idx];
        int vId = st.voterId;
        int j = st.indexInVoter;
        int aId = st.adminId;
        
        mpz_t xm, ym;
        mpz_init(xm);
        mpz_init(ym);
        element_to_mpz(xm, keyOut.eaKeys[aId].sgk1);
        element_to_mpz(ym, keyOut.eaKeys[aId].sgk2);
        BlindSignature sig = blindSign(params, preparedOutputs[vId], xm, ym, aId, vId);
        mpz_clear(xm);
        mpz_clear(ym);
        pipelineResults[vId].signatures[j] = sig;
    }
    auto blindEnd = Clock::now();
    auto blindTime = std::chrono::duration_cast<std::chrono::microseconds>(blindEnd - blindStart).count();
    
    auto pipelineEnd = Clock::now();
    auto totalTime = std::chrono::duration_cast<std::chrono::microseconds>(pipelineEnd - pipelineStart).count();
    
    for (int i = 0; i < voterCount; i++) {
        for (int j = 0; j < (int)pipelineResults[i].signatures.size(); j++) {
            BlindSignature &sig = pipelineResults[i].signatures[j];
        }
    } 
    
    // Unblind işlemleri - sıralı (sequential) çalışır
    auto unblindStart = Clock::now();
    std::vector<std::vector<std::pair<int, UnblindSignature>>> unblindResultsWithAdmin(voterCount);
    std::vector<std::vector<UnblindSignature>> unblindResults(voterCount);
    
    for(int i = 0; i < voterCount; i++) {
        int numSigs = (int) pipelineResults[i].signatures.size();
        unblindResults[i].resize(numSigs);
        unblindResultsWithAdmin[i].resize(numSigs);
        
        for(int j = 0; j < numSigs; j++) {
            int adminId = pipelineResults[i].signatures[j].adminId; 
            UnblindSignature usig = unblindSign(params, preparedOutputs[i], pipelineResults[i].signatures[j], keyOut.eaKeys[adminId], dids[i].did);
            unblindResults[i][j] = usig;
            unblindResultsWithAdmin[i][j] = {adminId, usig};
        }
    }
    auto unblindEnd = Clock::now();
    auto unblind_us = std::chrono::duration_cast<std::chrono::microseconds>(unblindEnd - unblindStart).count();
    
    for (int i = 0; i < voterCount; i++) {
        for (int j = 0; j < (int)unblindResultsWithAdmin[i].size(); j++) {
            int adminId = unblindResultsWithAdmin[i][j].first;
            UnblindSignature &usig = unblindResultsWithAdmin[i][j].second;
        }
    }
    
    // Aggregate işlemleri - sıralı (sequential) çalışır
    std::vector<AggregateSignature> aggregateResults(voterCount);
    auto aggregateStart = Clock::now();
    
    for(int i = 0; i < voterCount; i++) {
        AggregateSignature aggSig = aggregateSign(params, unblindResultsWithAdmin[i], keyOut.mvk, dids[i].did, params.prime_order);
        aggregateResults[i] = aggSig;
    }
    
    auto aggregateEnd = Clock::now();
    auto aggregate_us = std::chrono::duration_cast<std::chrono::microseconds>(aggregateEnd - aggregateStart).count();
    
    // Prove Credential işlemleri - sıralı (sequential) çalışır
    std::vector<ProveCredentialOutput> proveResults(voterCount);
    auto proveStart = Clock::now();
    
    for(int i = 0; i < voterCount; i++) {
        ProveCredentialOutput pOut = proveCredential(params, aggregateResults[i], keyOut.mvk, dids[i].did, preparedOutputs[i].o);
        proveResults[i] = pOut;
    }
    
    auto proveEnd = Clock::now();
    auto prove_us = std::chrono::duration_cast<std::chrono::microseconds>(proveEnd - proveStart).count();
    
    // KoR üretimi - sıralı (sequential) çalışır
    auto korStart = Clock::now();
    
    for(int i = 0; i < voterCount; i++) {
        mpz_t did_int;
        mpz_init(did_int);
        mpz_set_str(did_int, dids[i].did.c_str(), 16);
        mpz_mod(did_int, did_int, params.prime_order);
        
        element_t com_elem;
        element_init_G1(com_elem, params.pairing);
        try {
            stringToElement(com_elem, preparedOutputs[i].com_str, params.pairing, 1);
        } catch (const std::exception& e) {
            std::cerr << "Error converting com string to element: " << e.what() << std::endl;
            element_random(com_elem);
        }
        
        KnowledgeOfRepProof korProof = generateKoRProof(
            params,
            aggregateResults[i].h,
            proveResults[i].k,
            proveResults[i].r,
            com_elem,
            keyOut.mvk.alpha2,
            keyOut.mvk.beta2,
            did_int,
            preparedOutputs[i].o
        );
        
        element_set(proveResults[i].c, korProof.c);
        element_set(proveResults[i].s1, korProof.s1);
        element_set(proveResults[i].s2, korProof.s2);
        element_set(proveResults[i].s3, korProof.s3);
        proveResults[i].proof_v = korProof.proof_string;
        
        element_clear(com_elem);
        element_clear(korProof.c);
        element_clear(korProof.s1);
        element_clear(korProof.s2);
        element_clear(korProof.s3);
        mpz_clear(did_int);
    }
    
    auto korEnd = Clock::now();
    auto kor_us = std::chrono::duration_cast<std::chrono::microseconds>(korEnd - korStart).count();
    
    // Pairing Check - sıralı (sequential) çalışır
    auto pairingCheckStart = Clock::now();
    bool allPairingVerified = true;
    
    for(int i = 0; i < voterCount; i++) {
        bool pairing_ok = pairingCheck(params, proveResults[i]);
        if (!pairing_ok) {
            allPairingVerified = false;
        }
    }
    
    auto pairingCheckEnd = Clock::now();
    auto pairingCheck_us = std::chrono::duration_cast<std::chrono::microseconds>(pairingCheckEnd - pairingCheckStart).count();
    
    // KoR Verify - sıralı (sequential) çalışır
    auto korVerStart = Clock::now();
    bool allKorVerified = true;
    
    for(int i = 0; i < voterCount; i++) {
        bool kor_ok = checkKoRVerify(
            params,
            proveResults[i],
            keyOut.mvk,
            preparedOutputs[i].com_str, 
            aggregateResults[i].h
        );
        if (!kor_ok) {
            allKorVerified = false;
        }
    }
    
    auto korVerEnd = Clock::now();
    auto korVer_us = std::chrono::duration_cast<std::chrono::microseconds>(korVerEnd - korVerStart).count();
    
    // Toplam doğrulama süresi
    auto totalVerStart = Clock::now();
    bool allVerified = true;
    
    for(int i = 0; i < voterCount; i++) {
        bool pairing_ok = pairingCheck(params, proveResults[i]);
        bool kor_ok = checkKoRVerify(
            params,
            proveResults[i],
            keyOut.mvk,
            preparedOutputs[i].com_str, 
            aggregateResults[i].h
        );
        
        bool verified = pairing_ok && kor_ok;
        if (!verified) {
            allVerified = false;
        }
    }
    
    auto totalVerEnd = Clock::now();
    auto totalVer_us = std::chrono::duration_cast<std::chrono::microseconds>(totalVerEnd - totalVerStart).count();
    
    if (!allVerified) {
        throw std::runtime_error("Verification failed: pairing check or KoR verification returned false");
    }
    
    // Kaynakları temizle
    element_clear(keyOut.mvk.alpha2);
    element_clear(keyOut.mvk.beta2);
    element_clear(keyOut.mvk.beta1);
    
    for (int i = 0; i < ne; i++) {
        element_clear(keyOut.eaKeys[i].sgk1);
        element_clear(keyOut.eaKeys[i].sgk2);
        element_clear(keyOut.eaKeys[i].vkm1);
        element_clear(keyOut.eaKeys[i].vkm2);
        element_clear(keyOut.eaKeys[i].vkm3);
    }
    
    for (int i = 0; i < voterCount; i++) {
        mpz_clear(dids[i].x);
    }
    
    clearParams(params);
    
    auto programEnd = Clock::now();
    auto totalDuration = std::chrono::duration_cast<std::chrono::microseconds>(programEnd - programStart).count();
    
    double setup_ms    = setup_us    / 1000.0;
    double pairing_ms  = pairing_us  / 1000.0;
    double keygen_ms   = keygen_us   / 1000.0;
    double idGen_ms    = idGen_us    / 1000.0;
    double didGen_ms   = didGen_us   / 1000.0;
    double prep_ms     = prepTime    / 1000.0;
    double blind_ms    = blindTime   / 1000.0;
    double unblind_ms  = unblind_us  / 1000.0;
    double aggregate_ms = aggregate_us / 1000.0;
    double prove_ms    = prove_us    / 1000.0;
    double kor_ms      = kor_us      / 1000.0;
    double pairingCheck_ms = pairingCheck_us / 1000.0;
    double korVer_ms   = korVer_us   / 1000.0;
    double totalVer_ms = totalVer_us / 1000.0;
    double total_ms    = totalDuration / 1000.0;
    
    std::cout << "=== Zaman Olcumleri (ms) ===\n";
    std::cout << "Setup suresi       : " << setup_ms    << " ms\n";
    std::cout << "Pairing suresi     : " << pairing_ms  << " ms\n";
    std::cout << "KeyGen suresi      : " << keygen_ms   << " ms\n";
    std::cout << "ID Generation      : " << idGen_ms    << " ms\n";
    std::cout << "DID Generation     : " << didGen_ms   << " ms\n";
    std::cout << "Prepare Phase      : " << prep_ms     << " ms\n";
    std::cout << "BlindSign Phase    : " << blind_ms    << " ms\n";
    std::cout << "Unblind Phase      : " << unblind_ms  << " ms\n";
    std::cout << "Aggregate Phase    : " << aggregate_ms << " ms\n";
    std::cout << "ProveCredential    : " << prove_ms    << " ms\n";
    std::cout << "KoR Generation     : " << kor_ms      << " ms\n";
    std::cout << "Pairing Check      : " << pairingCheck_ms << " ms\n";
    std::cout << "KoR Verification   : " << korVer_ms   << " ms\n";
    std::cout << "Total Verification : " << totalVer_ms << " ms\n";
    std::cout << "Total execution    : " << total_ms    << " ms\n";
    std::cout << "\n=== Program Sonu ===\n";
    
    return 0;
}
