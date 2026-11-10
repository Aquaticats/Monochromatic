// Checks of the store reading a disposable TypeScript project copies its compiler from.
// What: a store holding one version, a store holding two, and a store holding neither.
// Why: the folder name carries the version, and the check must follow the workspace rather than a version written in it.
import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, rmSync } from 'node:fs';
import { homedir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';
import { typescriptInStore } from './typescript-in-store.ts';

// A disposable store below the private agent scratch root, holding one folder per name, removed with the test that made it.
const storeHolding = (names: string[]): string => {
  const root = join(homedir(), 'temp', 'agent');
  mkdirSync(root, { recursive: true });
  const store = mkdtempSync(join(root, 'ide-typescript-store-'));
  for (const name of names) mkdirSync(join(store, name, 'node_modules'), { recursive: true });
  return store;
};

test(
  'the store folders the check copies are the ones the store holds',
  () => {
    // The pair the workspace installs, as its store names them; the version is the store's, not this check's.
    const store = storeHolding(['typescript@7.1.0-beta', '@typescript+typescript-linux-x64@7.1.0-beta', 'typescript-eslint@8.0.0']);
    try {
      assert.deepEqual(typescriptInStore(store), {
        compiler: 'typescript@7.1.0-beta',
        platformBinary: '@typescript+typescript-linux-x64@7.1.0-beta',
      });
    } finally {
      rmSync(store, { recursive: true, force: true });
    }
  }
);

test(
  'a store holding two versions names both instead of choosing one',
  () => {
    const store = storeHolding(['typescript@7.0.2', 'typescript@7.1.0-beta', '@typescript+typescript-linux-x64@7.1.0-beta']);
    try {
      assert.throws(() => typescriptInStore(store), { message: /typescript@7\.0\.2, typescript@7\.1\.0-beta/ });
    } finally {
      rmSync(store, { recursive: true, force: true });
    }
  }
);

test(
  'a store without the compiler is an error that names the store',
  () => {
    const store = storeHolding(['typescript-eslint@8.0.0']);
    try {
      assert.throws(() => typescriptInStore(store), { message: /no TypeScript compiler/ });
    } finally {
      rmSync(store, { recursive: true, force: true });
    }
  }
);
