/* tslint:disable */
/* eslint-disable */
/**
 * Generate a new DID with 12-word BIP39 mnemonic (128-bit entropy)
 * Returns: { mnemonic: "word1 word2 ...", did: "sha512_hash" }
 */
export function generate_did(): any;
/**
 * Recover DID from existing 12-word mnemonic
 * Input: "word1 word2 word3 ... word12"
 * Returns: { did: "sha512_hash", valid: true/false }
 */
export function recover_did(mnemonic_phrase: string): any;
/**
 * Generate o-value with 12-word BIP39 mnemonic (for blind signature blinding)
 * Returns: { mnemonic: "word1 word2 ...", o_value: "sha512_hash" }
 */
export function generate_o_value(): any;
/**
 * Recover o-value from existing 12-word mnemonic
 * Input: "word1 word2 word3 ... word12"
 * Returns: { did: "sha512_hash", valid: true/false }
 */
export function recover_o_value(mnemonic_phrase: string): any;
/**
 * PrepareBlindSign - Algorithm 4 from TIAC
 * Creates a blind signature request with zero-knowledge proof
 *
 * Inputs:
 *   - did_hex: Digital Identity (SHA-512 hash from mnemonic, hex string)
 *   - o_hex: Blinding factor (SHA-512 hash from mnemonic, hex string)
 *
 * Returns: PrepareBlindSignResult with commitment, proof, and blinding factors
 */
export function prepare_blind_sign(did_hex: string, o_hex: string): any;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly generate_did: () => [number, number, number];
  readonly recover_did: (a: number, b: number) => [number, number, number];
  readonly prepare_blind_sign: (a: number, b: number, c: number, d: number) => [number, number, number];
  readonly generate_o_value: () => [number, number, number];
  readonly recover_o_value: (a: number, b: number) => [number, number, number];
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_export_4: WebAssembly.Table;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
