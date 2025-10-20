#include <iostream>
#include <string>
#include <sstream>
#include <vector>
#include <iomanip>
#include <cstring>
#include <fstream>
#include "setup.h"
#include "keygen.h"
#include <gmp.h>

// Simple JSON value extraction (minimal parser for our specific needs)
std::string extractJsonString(const std::string& json, const std::string& key) {
    std::string search = "\"" + key + "\":";
    size_t pos = json.find(search);
    if (pos == std::string::npos) return "";

    pos = json.find("\"", pos + search.length());
    if (pos == std::string::npos) return "";
    pos++; // Skip opening quote

    size_t end = json.find("\"", pos);
    if (end == std::string::npos) return "";

    std::string value = json.substr(pos, end - pos);

    // Decode JSON escape sequences (specifically \n for newlines)
    std::string decoded;
    for (size_t i = 0; i < value.length(); i++) {
        if (value[i] == '\\' && i + 1 < value.length()) {
            char next = value[i + 1];
            if (next == 'n') {
                decoded += '\n';
                i++; // Skip next char
            } else if (next == 't') {
                decoded += '\t';
                i++;
            } else if (next == '\\') {
                decoded += '\\';
                i++;
            } else if (next == '"') {
                decoded += '"';
                i++;
            } else {
                decoded += value[i];
            }
        } else {
            decoded += value[i];
        }
    }

    return decoded;
}

// Convert element to hex string
std::string elementToHex(element_t e) {
    int len = element_length_in_bytes(e);
    std::vector<unsigned char> buf(len);
    element_to_bytes(buf.data(), e);

    std::stringstream ss;
    for (unsigned char byte : buf) {
        ss << std::hex << std::setw(2) << std::setfill('0') << (int)byte;
    }
    return ss.str();
}

void hexToElement(element_t e, const std::string& hex, pairing_t pairing, int group) {
    std::vector<unsigned char> buf;
    for (size_t i = 0; i < hex.length(); i += 2) {
        std::string byteStr = hex.substr(i, 2);
        unsigned char byte = (unsigned char)strtol(byteStr.c_str(), NULL, 16);
        buf.push_back(byte);
    }

    if (group == 1) {
        element_init_G1(e, pairing);
    } else if (group == 2) {
        element_init_G2(e, pairing);
    } else {
        element_init_Zr(e, pairing);
    }
    element_from_bytes(e, buf.data());
}

// Convert mpz_t to hex string
std::string mpzToHex(mpz_t z) {
    char* str = mpz_get_str(NULL, 16, z);
    std::string result(str);
    free(str);
    return result;
}

void hexToMpz(mpz_t z, const std::string& hex) {
    mpz_set_str(z, hex.c_str(), 16);
}

int main(int argc, char* argv[]) {
    if (argc < 2) {
        std::cerr << "{\"error\":\"No command specified\"}" << std::endl;
        return 1;
    }

    std::string command = argv[1];

    try {
        TIACParams params;

        // Try to load shared crypto parameters from file
        std::ifstream param_file("/app/storage/crypto_params.json");
        bool using_shared_params = false;

        if (param_file.good()) {
            // Load parameters from file
            std::stringstream buffer;
            buffer << param_file.rdbuf();
            std::string json_content = buffer.str();
            param_file.close();

            std::string pairing_params_str = extractJsonString(json_content, "pairing_params");
            std::string prime_order_str = extractJsonString(json_content, "prime_order");
            std::string g1_str = extractJsonString(json_content, "g1");
            std::string g2_str = extractJsonString(json_content, "g2");
            std::string h1_str = extractJsonString(json_content, "h1");

            if (!pairing_params_str.empty() && !prime_order_str.empty()) {
                // Initialize pairing from shared parameters
                // Use pbc_param_init_set_str to parse the parameter string
                std::cerr << "[DKG_CLI] 📝 Loading shared pairing parameters..." << std::endl;

                pbc_param_t par;
                std::cerr << "[DKG_CLI] 📄 Pairing params length: " << pairing_params_str.length() << " bytes" << std::endl;
                std::cerr << "[DKG_CLI] 📄 First 200 chars: " << pairing_params_str.substr(0, 200) << std::endl;

                if (pbc_param_init_set_str(par, pairing_params_str.c_str()) != 0) {
                    std::cerr << "[DKG_CLI] ❌ Failed to parse pairing parameters string" << std::endl;
                    std::cerr << "[DKG_CLI] 📝 Full params: " << pairing_params_str << std::endl;
                    std::cerr << "[DKG_CLI] 🔄 Falling back to setupParams()" << std::endl;
                    params = setupParams();
                } else {
                    // Initialize pairing with parsed parameters
                    pairing_init_pbc_param(params.pairing, par);
                    pbc_param_clear(par);

                    // Load prime order
                    mpz_init(params.prime_order);
                    hexToMpz(params.prime_order, prime_order_str);

                    // Load generators
                    hexToElement(params.g1, g1_str, params.pairing, 1);
                    hexToElement(params.g2, g2_str, params.pairing, 2);
                    hexToElement(params.h1, h1_str, params.pairing, 1);

                    using_shared_params = true;
                    std::cerr << "[DKG_CLI] ✅ Using shared crypto parameters from backend" << std::endl;
                }
            } else {
                std::cerr << "[DKG_CLI] ⚠️  Incomplete crypto parameters in file, generating new" << std::endl;
                params = setupParams();
            }
        } else {
            // No shared parameters file, generate new (fallback for testing)
            std::cerr << "[DKG_CLI] ⚠️  No shared crypto parameters file found, generating new" << std::endl;
            params = setupParams();
        }

        if (command == "generate_polynomials") {
            // Args: threshold
            if (argc < 3) {
                std::cerr << "{\"error\":\"Missing threshold parameter\"}" << std::endl;
                return 1;
            }
            int threshold = std::stoi(argv[2]);

            // Generate two random polynomials F and G
            // For threshold t, polynomial degree is t, which needs t+1 coefficients
            int poly_size = threshold + 1;
            EAPolynomials polynomials;
            polynomials.size = poly_size;
            polynomials.F_coeffs = new mpz_t[poly_size];
            polynomials.G_coeffs = new mpz_t[poly_size];

            randomPolynomial_ptr(polynomials.F_coeffs, poly_size, params.prime_order);
            randomPolynomial_ptr(polynomials.G_coeffs, poly_size, params.prime_order);

            // Generate commitments
            EACommitments commitments;
            generateCommitments(commitments, polynomials, params);

            // Output JSON
            std::cout << "{" << std::endl;
            std::cout << "  \"F_coeffs\": [";
            for (int i = 0; i < poly_size; i++) {
                std::cout << "\"" << mpzToHex(polynomials.F_coeffs[i]) << "\"";
                if (i < poly_size - 1) std::cout << ",";
            }
            std::cout << "]," << std::endl;

            std::cout << "  \"G_coeffs\": [";
            for (int i = 0; i < poly_size; i++) {
                std::cout << "\"" << mpzToHex(polynomials.G_coeffs[i]) << "\"";
                if (i < poly_size - 1) std::cout << ",";
            }
            std::cout << "]," << std::endl;

            std::cout << "  \"commitments\": {" << std::endl;
            std::cout << "    \"V_x\": [";
            for (int i = 0; i < poly_size; i++) {
                std::cout << "\"" << elementToHex(commitments.V_x[i]) << "\"";
                if (i < poly_size - 1) std::cout << ",";
            }
            std::cout << "]," << std::endl;

            std::cout << "    \"V_y\": [";
            for (int i = 0; i < poly_size; i++) {
                std::cout << "\"" << elementToHex(commitments.V_y[i]) << "\"";
                if (i < poly_size - 1) std::cout << ",";
            }
            std::cout << "]," << std::endl;

            std::cout << "    \"V_y_prime\": [";
            for (int i = 0; i < poly_size; i++) {
                std::cout << "\"" << elementToHex(commitments.V_y_prime[i]) << "\"";
                if (i < poly_size - 1) std::cout << ",";
            }
            std::cout << "]" << std::endl;
            std::cout << "  }" << std::endl;
            std::cout << "}" << std::endl;

            // Cleanup
            for (int i = 0; i < poly_size; i++) {
                mpz_clear(polynomials.F_coeffs[i]);
                mpz_clear(polynomials.G_coeffs[i]);
                element_clear(commitments.V_x[i]);
                element_clear(commitments.V_y[i]);
                element_clear(commitments.V_y_prime[i]);
            }
            delete[] polynomials.F_coeffs;
            delete[] polynomials.G_coeffs;
            delete[] commitments.V_x;
            delete[] commitments.V_y;
            delete[] commitments.V_y_prime;

        } else if (command == "evaluate_polynomial") {
            // Args: threshold receiver_index F_coeff_0 F_coeff_1 ... G_coeff_0 G_coeff_1 ...
            // Note: For threshold t, we have t+1 coefficients (degree t polynomial)
            if (argc < 4) {
                std::cerr << "{\"error\":\"Missing parameters\"}" << std::endl;
                return 1;
            }
            int threshold = std::stoi(argv[2]);
            int receiver_index = std::stoi(argv[3]);

            int poly_size = threshold + 1;  // Degree t polynomial needs t+1 coefficients

            if (argc < 4 + poly_size * 2) {
                std::cerr << "{\"error\":\"Not enough coefficients\"}" << std::endl;
                return 1;
            }

            mpz_t* F_coeffs = new mpz_t[poly_size];
            mpz_t* G_coeffs = new mpz_t[poly_size];

            for (int i = 0; i < poly_size; i++) {
                mpz_init(F_coeffs[i]);
                mpz_init(G_coeffs[i]);
                hexToMpz(F_coeffs[i], argv[4 + i]);
                hexToMpz(G_coeffs[i], argv[4 + poly_size + i]);
            }

            // Evaluate at receiver_index
            mpz_t F_result, G_result;
            mpz_init(F_result);
            mpz_init(G_result);

            evalPolynomial_ptr(F_result, F_coeffs, poly_size, receiver_index, params.prime_order);
            evalPolynomial_ptr(G_result, G_coeffs, poly_size, receiver_index, params.prime_order);

            // Output JSON
            std::cout << "{" << std::endl;
            std::cout << "  \"F\": \"" << mpzToHex(F_result) << "\"," << std::endl;
            std::cout << "  \"G\": \"" << mpzToHex(G_result) << "\"" << std::endl;
            std::cout << "}" << std::endl;

            // Cleanup
            for (int i = 0; i < poly_size; i++) {
                mpz_clear(F_coeffs[i]);
                mpz_clear(G_coeffs[i]);
            }
            delete[] F_coeffs;
            delete[] G_coeffs;
            mpz_clear(F_result);
            mpz_clear(G_result);

        } else if (command == "verify_share") {
            // Args: threshold my_index F_share G_share V_x_0 V_x_1 ... V_y_0 V_y_1 ... V_y_prime_0 ...
            // Note: For threshold t, we have t+1 coefficients (degree t polynomial)
            if (argc < 5) {
                std::cerr << "{\"error\":\"Missing parameters\"}" << std::endl;
                return 1;
            }

            int threshold = std::stoi(argv[2]);
            int my_index = std::stoi(argv[3]);

            int poly_size = threshold + 1;  // Degree t polynomial needs t+1 coefficients

            if (argc < 4 + 2 + poly_size * 3) {
                std::cerr << "{\"error\":\"Not enough commitment data\"}" << std::endl;
                return 1;
            }

            // Parse share
            Share share;
            mpz_init(share.F_l_i);
            mpz_init(share.G_l_i);
            hexToMpz(share.F_l_i, argv[4]);
            hexToMpz(share.G_l_i, argv[5]);

            // Parse commitments
            EACommitments commitments;
            commitments.size = poly_size;
            commitments.V_x = new element_t[poly_size];
            commitments.V_y = new element_t[poly_size];
            commitments.V_y_prime = new element_t[poly_size];

            int arg_idx = 6;
            for (int i = 0; i < poly_size; i++) {
                hexToElement(commitments.V_x[i], argv[arg_idx++], params.pairing, 2);
            }
            for (int i = 0; i < poly_size; i++) {
                hexToElement(commitments.V_y[i], argv[arg_idx++], params.pairing, 2);
            }
            for (int i = 0; i < poly_size; i++) {
                hexToElement(commitments.V_y_prime[i], argv[arg_idx++], params.pairing, 1);
            }

            // Verify using 1-based index (matching working main.cpp pattern)
            bool valid = verifyShare(share, commitments, my_index, params);

            std::cout << "{\"valid\": " << (valid ? "true" : "false") << "}" << std::endl;

            // Cleanup
            mpz_clear(share.F_l_i);
            mpz_clear(share.G_l_i);
            for (int i = 0; i < poly_size; i++) {
                element_clear(commitments.V_x[i]);
                element_clear(commitments.V_y[i]);
                element_clear(commitments.V_y_prime[i]);
            }
            delete[] commitments.V_x;
            delete[] commitments.V_y;
            delete[] commitments.V_y_prime;

        } else if (command == "aggregate_mvk") {
            // Args: threshold num_qualified qualified_indices V_x_0 arrays for each qualified trustee
            // Note: For threshold t, we have t+1 coefficients (degree t polynomial)
            if (argc < 4) {
                std::cerr << "{\"error\":\"Missing parameters\"}" << std::endl;
                return 1;
            }

            int threshold = std::stoi(argv[2]);
            int num_qualified = std::stoi(argv[3]);

            int poly_size = threshold + 1;  // Degree t polynomial needs t+1 coefficients

            if (argc < 4 + num_qualified + num_qualified * poly_size * 3) {
                std::cerr << "{\"error\":\"Not enough commitment data for aggregation\"}" << std::endl;
                return 1;
            }

            // Parse qualified indices
            std::vector<int> qualified_indices;
            for (int i = 0; i < num_qualified; i++) {
                qualified_indices.push_back(std::stoi(argv[4 + i]));
            }

            // Parse commitments for each qualified trustee
            std::vector<EACommitments> all_commitments(num_qualified);
            int arg_idx = 4 + num_qualified;

            for (int q = 0; q < num_qualified; q++) {
                all_commitments[q].size = poly_size;
                all_commitments[q].V_x = new element_t[poly_size];
                all_commitments[q].V_y = new element_t[poly_size];
                all_commitments[q].V_y_prime = new element_t[poly_size];

                for (int i = 0; i < poly_size; i++) {
                    hexToElement(all_commitments[q].V_x[i], argv[arg_idx++], params.pairing, 2);
                }
                for (int i = 0; i < poly_size; i++) {
                    hexToElement(all_commitments[q].V_y[i], argv[arg_idx++], params.pairing, 2);
                }
                for (int i = 0; i < poly_size; i++) {
                    hexToElement(all_commitments[q].V_y_prime[i], argv[arg_idx++], params.pairing, 1);
                }
            }

            // Aggregate MVK
            element_t alpha2, beta2, beta1;
            element_init_G2(alpha2, params.pairing);
            element_init_G2(beta2, params.pairing);
            element_init_G1(beta1, params.pairing);

            element_set1(alpha2);
            element_set1(beta2);
            element_set1(beta1);

            for (int q = 0; q < num_qualified; q++) {
                element_mul(alpha2, alpha2, all_commitments[q].V_x[0]);
                element_mul(beta2, beta2, all_commitments[q].V_y[0]);
                element_mul(beta1, beta1, all_commitments[q].V_y_prime[0]);
            }

            // Output JSON
            std::cout << "{" << std::endl;
            std::cout << "  \"alpha2\": \"" << elementToHex(alpha2) << "\"," << std::endl;
            std::cout << "  \"beta2\": \"" << elementToHex(beta2) << "\"," << std::endl;
            std::cout << "  \"beta1\": \"" << elementToHex(beta1) << "\"" << std::endl;
            std::cout << "}" << std::endl;

            // Cleanup
            element_clear(alpha2);
            element_clear(beta2);
            element_clear(beta1);
            for (int q = 0; q < num_qualified; q++) {
                for (int i = 0; i < poly_size; i++) {
                    element_clear(all_commitments[q].V_x[i]);
                    element_clear(all_commitments[q].V_y[i]);
                    element_clear(all_commitments[q].V_y_prime[i]);
                }
                delete[] all_commitments[q].V_x;
                delete[] all_commitments[q].V_y;
                delete[] all_commitments[q].V_y_prime;
            }

        } else if (command == "compute_signing_key") {
            // Args: threshold num_qualified my_index F_share_1 G_share_1 F_share_2 G_share_2 ...
            if (argc < 5) {
                std::cerr << "{\"error\":\"Missing parameters\"}" << std::endl;
                return 1;
            }

            int threshold = std::stoi(argv[2]);
            int num_qualified = std::stoi(argv[3]);
            int my_index = std::stoi(argv[4]);

            if (argc < 5 + num_qualified * 2) {
                std::cerr << "{\"error\":\"Not enough shares\"}" << std::endl;
                return 1;
            }

            // Sum all shares
            mpz_t sgk1, sgk2;
            mpz_init_set_ui(sgk1, 0);
            mpz_init_set_ui(sgk2, 0);

            for (int i = 0; i < num_qualified; i++) {
                mpz_t F_share, G_share;
                mpz_init(F_share);
                mpz_init(G_share);
                hexToMpz(F_share, argv[5 + i * 2]);
                hexToMpz(G_share, argv[5 + i * 2 + 1]);

                mpz_add(sgk1, sgk1, F_share);
                mpz_mod(sgk1, sgk1, params.prime_order);

                mpz_add(sgk2, sgk2, G_share);
                mpz_mod(sgk2, sgk2, params.prime_order);

                mpz_clear(F_share);
                mpz_clear(G_share);
            }

            // Output JSON
            std::cout << "{" << std::endl;
            std::cout << "  \"sgk1\": \"" << mpzToHex(sgk1) << "\"," << std::endl;
            std::cout << "  \"sgk2\": \"" << mpzToHex(sgk2) << "\"" << std::endl;
            std::cout << "}" << std::endl;

            mpz_clear(sgk1);
            mpz_clear(sgk2);

        } else if (command == "compute_verification_keys") {
            // Args: threshold num_qualified my_index + commitment arrays
            if (argc < 5) {
                std::cerr << "{\"error\":\"Missing parameters\"}" << std::endl;
                return 1;
            }

            int threshold = std::stoi(argv[2]);
            int num_qualified = std::stoi(argv[3]);
            int my_index = std::stoi(argv[4]);

            if (argc < 5 + num_qualified * threshold * 3) {
                std::cerr << "{\"error\":\"Not enough commitment data\"}" << std::endl;
                return 1;
            }

            // Parse commitments
            std::vector<EACommitments> all_commitments(num_qualified);
            int arg_idx = 5;

            for (int q = 0; q < num_qualified; q++) {
                all_commitments[q].size = threshold;
                all_commitments[q].V_x = new element_t[threshold];
                all_commitments[q].V_y = new element_t[threshold];
                all_commitments[q].V_y_prime = new element_t[threshold];

                for (int i = 0; i < threshold; i++) {
                    hexToElement(all_commitments[q].V_x[i], argv[arg_idx++], params.pairing, 2);
                }
                for (int i = 0; i < threshold; i++) {
                    hexToElement(all_commitments[q].V_y[i], argv[arg_idx++], params.pairing, 2);
                }
                for (int i = 0; i < threshold; i++) {
                    hexToElement(all_commitments[q].V_y_prime[i], argv[arg_idx++], params.pairing, 1);
                }
            }

            // Compute VKs
            element_t vkm1, vkm2, vkm3;
            element_init_G2(vkm1, params.pairing);
            element_init_G2(vkm2, params.pairing);
            element_init_G1(vkm3, params.pairing);

            element_set1(vkm1);
            element_set1(vkm2);
            element_set1(vkm3);

            for (int q = 0; q < num_qualified; q++) {
                for (int j = 0; j < threshold; j++) {
                    mpz_t i_pow_j;
                    mpz_init(i_pow_j);
                    mpz_set_ui(i_pow_j, my_index);
                    mpz_pow_ui(i_pow_j, i_pow_j, j);

                    element_t term_x, term_y, term_y_prime;
                    element_init_G2(term_x, params.pairing);
                    element_init_G2(term_y, params.pairing);
                    element_init_G1(term_y_prime, params.pairing);

                    element_pow_mpz(term_x, all_commitments[q].V_x[j], i_pow_j);
                    element_pow_mpz(term_y, all_commitments[q].V_y[j], i_pow_j);
                    element_pow_mpz(term_y_prime, all_commitments[q].V_y_prime[j], i_pow_j);

                    element_mul(vkm1, vkm1, term_x);
                    element_mul(vkm2, vkm2, term_y);
                    element_mul(vkm3, vkm3, term_y_prime);

                    element_clear(term_x);
                    element_clear(term_y);
                    element_clear(term_y_prime);
                    mpz_clear(i_pow_j);
                }
            }

            // Output JSON
            std::cout << "{" << std::endl;
            std::cout << "  \"vk1\": \"" << elementToHex(vkm1) << "\"," << std::endl;
            std::cout << "  \"vk2\": \"" << elementToHex(vkm2) << "\"," << std::endl;
            std::cout << "  \"vk3\": \"" << elementToHex(vkm3) << "\"" << std::endl;
            std::cout << "}" << std::endl;

            // Cleanup
            element_clear(vkm1);
            element_clear(vkm2);
            element_clear(vkm3);
            for (int q = 0; q < num_qualified; q++) {
                for (int i = 0; i < threshold; i++) {
                    element_clear(all_commitments[q].V_x[i]);
                    element_clear(all_commitments[q].V_y[i]);
                    element_clear(all_commitments[q].V_y_prime[i]);
                }
                delete[] all_commitments[q].V_x;
                delete[] all_commitments[q].V_y;
                delete[] all_commitments[q].V_y_prime;
            }

        } else {
            std::cerr << "{\"error\":\"Unknown command: " << command << "\"}" << std::endl;
            return 1;
        }

        clearParams(params);

    } catch (const std::exception& e) {
        std::cerr << "{\"error\":\"Exception: " << e.what() << "\"}" << std::endl;
        return 1;
    }

    return 0;
}
