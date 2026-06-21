/* tslint:disable */
/* eslint-disable */

/**
 * Encoding result with metadata, returned to JavaScript by [`qr_encode`].
 */
export class QrResult {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Applied mask pattern (0–7).
     */
    readonly mask: number;
    /**
     * Row-major module grid (`1`=dark, `0`=light), length `side*side`.
     */
    readonly modules: Uint8Array;
    /**
     * Side length in modules.
     */
    readonly side: number;
    /**
     * Chosen QR Code version (1–40).
     */
    readonly version: number;
}

/**
 * Multi-line ASCII preview.
 */
export function qr_ascii(text: string, ecl: number, border: number): string;

/**
 * Encodes `text` with explicit ECL ordinal, minimum version, and mask mode
 * (`-1` = automatic, `0..=7` = forced), returning matrix + metadata.
 */
export function qr_encode(text: string, ecl: number, version_min: number, mask_mode: number): QrResult;

/**
 * Row-major module grid (`1`=dark, `0`=light); the side length is the integer
 * square root of the returned length.
 */
export function qr_modules(text: string, ecl: number): Uint8Array;

/**
 * QR Code side length in modules.
 */
export function qr_side(text: string, ecl: number): number;

/**
 * Standalone SVG string (`scale` pixels per module, 4-module quiet zone).
 */
export function qr_svg(text: string, ecl: number, scale: number): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_qrresult_free: (a: number, b: number) => void;
    readonly qr_ascii: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly qr_encode: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly qr_modules: (a: number, b: number, c: number) => [number, number, number, number];
    readonly qr_side: (a: number, b: number, c: number) => [number, number, number];
    readonly qr_svg: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly qrresult_mask: (a: number) => number;
    readonly qrresult_modules: (a: number) => [number, number];
    readonly qrresult_side: (a: number) => number;
    readonly qrresult_version: (a: number) => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
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
