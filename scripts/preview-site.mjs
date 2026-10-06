import http from 'node:http';
import { readFile, stat } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { root } from './build-site.mjs';

const directory = path.join(root, 'dist');
const types = { '.html': 'text/html; charset=utf-8', '.css': 'text/css; charset=utf-8', '.js': 'text/javascript; charset=utf-8', '.png': 'image/png', '.webp': 'image/webp', '.xml': 'application/xml; charset=utf-8', '.txt': 'text/plain; charset=utf-8' };
export function createPreviewServer() {
  return http.createServer(async (request, response) => {
    try {
      const url = new URL(request.url, 'http://localhost');
      const pathname = decodeURIComponent(url.pathname);
      if (pathname.includes('\0')) throw new Error('Invalid path');
      let filename = path.resolve(directory, `.${pathname}`);
      if (filename !== directory && !filename.startsWith(directory + path.sep)) throw new Error('Invalid path');
      const info = await stat(filename);
      if (info.isDirectory()) {
        if (!pathname.endsWith('/')) {
          response.writeHead(301, { Location: url.pathname + '/' + url.search });
          return response.end();
        }
        filename = path.join(filename, 'index.html');
      }
      const content = await readFile(filename);
      response.writeHead(200, { 'Content-Type': types[path.extname(filename)] || 'application/octet-stream' });
      response.end(content);
    } catch {
      response.writeHead(404, { 'Content-Type': 'text/html; charset=utf-8' });
      response.end(await readFile(path.join(directory, '404.html')).catch(() => 'Build the site first.'));
    }
  });
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  createPreviewServer().listen(4173, '127.0.0.1', () => console.log('Preview: http://127.0.0.1:4173'));
}
