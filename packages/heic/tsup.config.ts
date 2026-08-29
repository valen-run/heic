import { defineConfig } from 'tsup';

export default defineConfig({
  entry: {
    index: 'src/index.ts',
    worker: 'src/worker.ts',
    'worker-runtime': 'src/worker/runtime.ts',
  },
  format: ['esm'],
  dts: true,
  sourcemap: true,
  clean: false,
  outDir: 'dist',
  target: 'es2022',
  splitting: false,
  treeshake: true,
});
