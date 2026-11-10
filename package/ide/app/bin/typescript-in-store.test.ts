// Checks of the store reading a disposable TypeScript project copies its compiler from.
// What: a global virtual store holding one version, one holding two, and one holding no compiler.
// Why: the workspace's store is a shared tree whose folders carry the version and a content hash, so the reading follows
//      the store's own shape instead of a layout or a version written into the check.
import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, rmSync } from 'node:fs';
import { homedir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';
import { typescriptInStore } from './typescript-in-store.ts';

// A content hash as the store spells it.
const hash = 'da4002b0e9981a3f05febc4eae4beb82b74f85298f866c971b5a0a02bde414d8';

// A disposable store below the private agent scratch root, holding one folder per name and version, removed with the test.
const storeHolding = (entries: { name: string; version: string }[]): string => {
  const root = join(homedir(), 'temp', 'agent');
  mkdirSync(root, { recursive: true });
  const store = mkdtempSync(join(root, 'ide-typescript-store-'));
  for (const { name, version } of entries) {
    // The store keeps an unscoped package below its own `@` folder and a scoped one below its scope.
    const holder = name.startsWith('@') ? name : join('@', name);
    mkdirSync(join(store, holder, version, hash, 'node_modules', name), { recursive: true });
  }
  return store;
};

test(
  'the compiler and its platform binary are read from the folders the store holds',
  () => {
    // The pair the workspace installs, as its store names them; the version and hash are the store's, not this check's.
    const store = storeHolding([
      { name: 'typescript', version: '7.1.0-beta' },
      { name: '@typescript/typescript-linux-x64', version: '7.1.0-beta' },
      { name: 'typescript-eslint', version: '8.0.0' },
    ]);
    try {
      assert.deepEqual(typescriptInStore({ store }), {
        compiler: join(store, '@', 'typescript', '7.1.0-beta', hash, 'node_modules', 'typescript'),
        platformBinary: join(store, '@typescript', 'typescript-linux-x64', '7.1.0-beta', hash, 'node_modules', '@typescript', 'typescript-linux-x64'),
      });
    } finally {
      rmSync(store, { recursive: true, force: true });
    }
  }
);

test(
  'a store holding two compiler versions names both instead of choosing one',
  () => {
    const store = storeHolding([
      { name: 'typescript', version: '7.0.2' },
      { name: 'typescript', version: '7.1.0-beta' },
      { name: '@typescript/typescript-linux-x64', version: '7.1.0-beta' },
    ]);
    try {
      assert.throws(() => typescriptInStore({ store }), { message: /7\.0\.2, 7\.1\.0-beta/ });
    } finally {
      rmSync(store, { recursive: true, force: true });
    }
  }
);

test(
  'a store without the compiler is an error that names the store',
  () => {
    const store = storeHolding([{ name: 'typescript-eslint', version: '8.0.0' }]);
    try {
      assert.throws(() => typescriptInStore({ store }), { message: /holds no versions of typescript/ });
    } finally {
      rmSync(store, { recursive: true, force: true });
    }
  }
);
