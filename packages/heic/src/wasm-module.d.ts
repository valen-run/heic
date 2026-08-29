/**
 * Ambient module declarations for WebAssembly pkg bindings.
 */

declare module '*/valen_heic_wasm.js' {
  export function convert(data: Uint8Array, options: any): Uint8Array;
  export function get_raw_pixels(data: Uint8Array, options: any): any;
  export function is_heif(data: Uint8Array): boolean;
  export function probe(data: Uint8Array, options: any): any;
  export function wasm_detect(data: Uint8Array): boolean;
  export function wasm_inspect(
    data: Uint8Array,
    max_file_size?: number | null,
    max_width?: number | null,
    max_height?: number | null,
    max_pixel_count?: number | null
  ): any;

  export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;
  export interface InitOutput {
    readonly memory: WebAssembly.Memory;
  }
  export type SyncInitInput = BufferSource | WebAssembly.Module;
  export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;
  export default function __wbg_init(
    module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>
  ): Promise<InitOutput>;
}

declare module '../pkg/valen_heic_wasm.js' {
  export function convert(data: Uint8Array, options: any): Uint8Array;
  export function get_raw_pixels(data: Uint8Array, options: any): any;
  export function is_heif(data: Uint8Array): boolean;
  export function probe(data: Uint8Array, options: any): any;
  export function wasm_detect(data: Uint8Array): boolean;
  export function wasm_inspect(
    data: Uint8Array,
    max_file_size?: number | null,
    max_width?: number | null,
    max_height?: number | null,
    max_pixel_count?: number | null
  ): any;

  export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;
  export interface InitOutput {
    readonly memory: WebAssembly.Memory;
  }
  export type SyncInitInput = BufferSource | WebAssembly.Module;
  export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;
  export default function __wbg_init(
    module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>
  ): Promise<InitOutput>;
}
