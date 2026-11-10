// The TypeScript compiler a disposable project copies out of the workspace's package store.
// What: the store folder names of the compiler and of its platform binary, read from the store itself.
// Why: the store names carry the version, and a version written here as a literal went stale when the workspace moved
//      to another TypeScript; a project that copies a folder the store no longer holds starts no server at all, so the
//      check would report the harness with the same weight as the behaviour it measures.
import { readdirSync } from 'node:fs';

/**
 What: the store folders that hold the TypeScript compiler and its platform binary.
 Why: the workspace's store is the only copy the check may point a disposable project at, and its names carry the
      version, so the store itself is asked which version it holds.

 @param store - the package store of the workspace, `node_modules/.pnpm`
 @returns the compiler folder name and the platform binary folder name, as the store holds them
 @throws {Error} when the store holds no such pair, or more than one version of either
 */
export function typescriptInStore(store: string): { compiler: string; platformBinary: string } {
  // The store names a package `<name>@<version>`, and a scoped one `@<scope>+<name>@<version>`; a name that merely
  // starts with `typescript` (typescript-eslint, for instance) is not the compiler.
  const held = readdirSync(store);
  const compilers = held.filter(name => /^typescript@\d/.test(name)).sort();
  const platformBinaries = held.filter(name => /^@typescript\+typescript-linux-x64@\d/.test(name)).sort();
  if (compilers.length !== 1) {
    throw new Error('the store ' + store + ' holds ' + (compilers.length === 0 ? 'no TypeScript compiler' : compilers.length + ' TypeScript compilers: ' + compilers.join(', ')));
  }
  if (platformBinaries.length !== 1) {
    throw new Error('the store ' + store + ' holds ' + (platformBinaries.length === 0 ? 'no TypeScript compiler platform binary' : platformBinaries.length + ' TypeScript compiler platform binaries: ' + platformBinaries.join(', ')));
  }
  return { compiler: compilers[0] ?? '', platformBinary: platformBinaries[0] ?? '' };
}
