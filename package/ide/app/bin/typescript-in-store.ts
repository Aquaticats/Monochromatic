// The TypeScript compiler a disposable project copies out of the workspace's shared package store.
// What: the store directories of the compiler and of its platform binary, read from the store's own shape.
// Why: the workspace keeps one global virtual store whose folders carry the version and a content hash, so a path and a
//      version written into a check go stale silently: the copy fails or copies nothing, and the check reports the
//      harness with the same weight as the behaviour it measures.
import { readdirSync } from 'node:fs';
import { join } from 'node:path';

/**
 What: the store directories holding the TypeScript compiler and its platform binary.
 Why: the shared store is the only copy the check may point a disposable project at, and only the store knows which
      version and which content hash it holds.

 @param options - the store directory of the workspace
 @returns the directory of each package, as the store holds it
 @throws {Error} when the store holds no such package, or more than one version or content of one
 */
export function typescriptInStore({ store }: { store: string }): { compiler: string; platformBinary: string } {
  return {
    compiler: packageInStore({ store, name: 'typescript' }),
    platformBinary: packageInStore({ store, name: '@typescript/typescript-linux-x64' }),
  };
}

/**
 What: the directory of one installed package in the shared store.
 Why: the store nests a package below its name, its version, and a content hash, so every check that copies one has to
      walk the same three steps and say what it found when a step holds something else.

 @param options - the store directory and the package name as a manifest spells it
 @returns the package's own directory inside the store
 @throws {Error} when a step holds no entry or more than one

 @example A store holding one compiler version

 ```ts
 packageInStore({ store: '/home/me/.local/share/pnpm/store/v11/links', name: 'typescript' });
 ```
 */
function packageInStore({ store, name }: { store: string; name: string }): string {
  // An unscoped package sits below the store's own `@` folder; a scoped one below its scope.
  const holder = name.startsWith('@') ? join(store, name) : join(store, '@', name);
  const versions = entriesOf({ directory: holder, store, name, step: 'version' });
  const contents = entriesOf({ directory: join(holder, versions[0] ?? ''), store, name, step: 'content' });
  return join(holder, versions[0] ?? '', contents[0] ?? '', 'node_modules', name);
}

/**
 What: the single entry a store step holds.
 Why: a check that picks one of several would measure another version than the workspace uses, and a check that finds
      none has nothing to copy, so both are errors that name the step and what the store holds instead.

 @param options - the directory to read, the store and package it belongs to, and the step's name
 @returns the one entry the directory holds
 @throws {Error} when the directory holds no entry or more than one
 */
function entriesOf({ directory, store, name, step }: { directory: string; store: string; name: string; step: string }): string[] {
  let held: string[] = [];
  try {
    held = readdirSync(directory).toSorted();
  } catch (error) {
    // A missing step is the store not holding the package; anything else is a real failure.
    if (!(error instanceof Error && 'code' in error && error.code === 'ENOENT')) throw error;
    held = [];
  }
  if (held.length !== 1) {
    const found = held.length === 0 ? 'no ' : `${held.length} `;
    const shown = held.length === 0 ? '' : `: ${held.join(', ')}`;
    throw new Error(`the store ${store} holds ${found}${step}s of ${name}${shown}`);
  }
  return held;
}
