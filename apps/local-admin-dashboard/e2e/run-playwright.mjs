import { spawn, spawnSync } from 'node:child_process';
import net from 'node:net';
import path from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';

const externalServer = process.env.DEK_PLAYWRIGHT_EXTERNAL_SERVER === '1';
let baseURL = process.env.PLAYWRIGHT_BASE_URL ?? (
  externalServer ? 'http://127.0.0.1:3000' : undefined
);

let vite;

function reservePort(preferredPort) {
  return new Promise((resolve, reject) => {
    const server = net.createServer();
    server.unref();
    server.on('error', () => {
      const fallback = net.createServer();
      fallback.unref();
      fallback.on('error', reject);
      fallback.listen(0, '127.0.0.1', () => {
        const address = fallback.address();
        const port = typeof address === 'object' && address ? address.port : 0;
        fallback.close(() => resolve(port));
      });
    });
    server.listen(preferredPort, '127.0.0.1', () => {
      const address = server.address();
      const port = typeof address === 'object' && address ? address.port : preferredPort;
      server.close(() => resolve(port));
    });
  });
}

async function waitForServer(url) {
  const deadline = Date.now() + 120_000;
  while (Date.now() < deadline) {
    try {
      const response = await fetch(url);
      if (response.ok) {
        return;
      }
    } catch {
      // Server is still starting.
    }
    await delay(500);
  }
  throw new Error(`Timed out waiting for ${url}`);
}

function stopVite() {
  if (!vite?.pid) {
    return;
  }
  if (process.platform === 'win32') {
    spawnSync('taskkill', ['/pid', String(vite.pid), '/t', '/f'], { stdio: 'ignore' });
  } else {
    vite.kill('SIGTERM');
  }
}

function runNode(args, env) {
  return new Promise((resolve) => {
    const child = spawn(process.execPath, args, {
      stdio: 'inherit',
      env,
    });
    child.on('exit', (code, signal) => {
      resolve(code ?? (signal ? 1 : 0));
    });
  });
}

process.on('exit', stopVite);
process.on('SIGINT', () => {
  stopVite();
  process.exit(130);
});
process.on('SIGTERM', () => {
  stopVite();
  process.exit(143);
});

if (!externalServer) {
  const port = await reservePort(Number(process.env.DEK_PLAYWRIGHT_PORT ?? 5173));
  baseURL = `http://127.0.0.1:${port}`;
  const viteCli = path.resolve('node_modules', 'vite', 'bin', 'vite.js');
  vite = spawn(process.execPath, [viteCli, '--host', '127.0.0.1', '--port', String(port), '--strictPort'], {
    stdio: 'inherit',
    env: process.env,
  });
  await waitForServer(baseURL);
}

const playwrightCli = path.resolve('node_modules', '@playwright', 'test', 'cli.js');
const exitCode = await runNode([playwrightCli, 'test', ...process.argv.slice(2)], {
  ...process.env,
  PLAYWRIGHT_BASE_URL: baseURL,
});

stopVite();
process.exit(exitCode);
