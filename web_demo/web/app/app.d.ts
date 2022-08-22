/* tslint:disable */
/* eslint-disable */
/**
* @param {any} render_ctx
*/
export function startup(render_ctx: any): void;
/**
* @param {any} data
*/
export function parse_font(data: any): void;
/**
* @param {number} value
*/
export function update_render_mid_value(value: number): void;
/**
* @param {string} character
* @param {number} height
* @returns {number | undefined}
*/
export function compute_fixed_height(character: string, height: number): number | undefined;
/**
* @param {any} output_ctx
* @param {string} character
* @param {number} size
* @param {number} spread
* @returns {number | undefined}
*/
export function render(output_ctx: any, character: string, size: number, spread: number): number | undefined;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly startup: (a: number) => void;
  readonly parse_font: (a: number) => void;
  readonly update_render_mid_value: (a: number) => void;
  readonly compute_fixed_height: (a: number, b: number, c: number, d: number) => void;
  readonly render: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
  readonly __wbindgen_malloc: (a: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number) => number;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_free: (a: number, b: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
}

/**
* Synchronously compiles the given `bytes` and instantiates the WebAssembly module.
*
* @param {BufferSource} bytes
*
* @returns {InitOutput}
*/
export function initSync(bytes: BufferSource): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
