/**
 * Minimal HTTP test server for Playwright browser test harness with CSP controls.
 */

import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT_DIR = path.resolve(__dirname, '../../..');

const MIME_TYPES: Record<string, string> = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'application/javascript; charset=utf-8',
  '.mjs': 'application/javascript; charset=utf-8',
  '.wasm': 'application/wasm',
  '.json': 'application/json',
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.jpeg': 'image/jpeg',
  '.webp': 'image/webp',
  '.css': 'text/css',
};

export function createHarnessServer(port = 3456): http.Server {
  const server = http.createServer((req, res) => {
    const parsedUrl = new URL(req.url || '/', `http://localhost:${port}`);
    let pathname = parsedUrl.pathname;
    const isStrictCsp = pathname.startsWith('/strict-csp');

    if (isStrictCsp) {
      res.setHeader(
        'Content-Security-Policy',
        "default-src 'self'; script-src 'self'; worker-src 'self'; img-src 'self' blob: data:; style-src 'self' 'unsafe-inline'; object-src 'none';"
      );
    }

    // Default route to harness index.html
    if (pathname === '/' || pathname === '/standard' || pathname === '/strict-csp') {
      pathname = '/tests/browser/harness/index.html';
    }

    // Resolve path relative to repo root or packages/heic
    let filePath = path.join(ROOT_DIR, pathname);
    if (pathname.startsWith('/dist/') || pathname.startsWith('/pkg/')) {
      filePath = path.join(ROOT_DIR, 'packages/heic', pathname);
    }

    if (!fs.existsSync(filePath) || fs.statSync(filePath).isDirectory()) {
      res.writeHead(404, { 'Content-Type': 'text/plain' });
      res.end(`404 Not Found: ${pathname}`);
      return;
    }

    const ext = path.extname(filePath).toLowerCase();
    const contentType = MIME_TYPES[ext] || 'application/octet-stream';

    res.writeHead(200, {
      'Content-Type': contentType,
      'Access-Control-Allow-Origin': '*',
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp',
    });

    fs.createReadStream(filePath).pipe(res);
  });

  return server;
}

// Start standalone server if executed directly
if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const port = Number(process.env.PORT) || 3456;
  const server = createHarnessServer(port);
  server.listen(port, () => {
    console.log(`[Harness Server] Listening at http://localhost:${port}`);
  });
}
