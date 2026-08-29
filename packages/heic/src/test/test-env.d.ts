declare module 'node:test' {
  export default function test(name: string, fn: () => void | Promise<void>): void;
}

declare module 'node:assert/strict' {
  export function equal(actual: unknown, expected: unknown, message?: string): void;
  export function notEqual(actual: unknown, expected: unknown, message?: string): void;
  export function ok(value: unknown, message?: string): void;
  export function throws(fn: () => void, error?: Function | RegExp): void;
  export function doesNotThrow(fn: () => void): void;
  export function rejects(asyncFn: () => Promise<unknown>, error?: Function | RegExp): Promise<void>;
}

declare module 'node:fs' {
  export function readFileSync(path: string): Uint8Array;
}

declare module 'node:path' {
  export function dirname(p: string): string;
  export function resolve(...paths: string[]): string;
}

declare module 'node:url' {
  export function fileURLToPath(url: string | URL): string;
}
