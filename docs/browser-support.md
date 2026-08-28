# Browser Support

`@valen/heic` is designed for broad compatibility with modern web browsers supporting WebAssembly.

## Compatibility Matrix

| Browser | Minimum Version | WebAssembly Support | Web Worker Support |
| ------- | --------------- | ------------------- | ------------------ |
| Chrome  | 70+             | Full                | Full               |
| Edge    | 79+ (Chromium)  | Full                | Full               |
| Firefox | 65+             | Full                | Full               |
| Safari  | 12+             | Full                | Full               |
| iOS Safari | 12.2+        | Full                | Full               |
| Chrome Android | 70+       | Full                | Full               |

## Required Browser Features

- `WebAssembly` API (`WebAssembly.instantiate`, `WebAssembly.Memory`)
- `Promise` & `async/await`
- `TypedArray` (`Uint8Array`, `ArrayBuffer`)
- `Blob` and `File` APIs (in DOM environments)
- `AbortController` / `AbortSignal` (for cancellable operations)
