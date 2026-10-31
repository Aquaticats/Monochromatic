/**
 Version re-resolution sources for `toolchain:update`, one resolver per
 family (the sources `doc/planning/mpm-toolchain-migration.md` names).

 Every resolver is pure over a {@link Fetcher}, so unit tests replay
 recorded responses and never touch the network. Digest sources per family:
 GitHub release-asset `digest` fields, nodejs.org and unofficial-builds
 `SHASUMS256.txt`, PyPI file digests, the Adoptium package checksum, and the
 anaconda.org file `sha256`. Google's repository publishes only a SHA-1 for
 cmdline-tools, so that one resolver downloads the zip and hashes the bytes
 when the version actually moves.

 The resolvers live in sibling `update.resolvers.*.ts` modules; this module
 re-exports them so importers keep one entry point.

 @module
 */

export { resolveJava, } from './update.resolvers.adoptium.ts';
export { resolveAndroid, } from './update.resolvers.android.ts';
export { resolveAws, } from './update.resolvers.aws.ts';
export { resolveConda, } from './update.resolvers.conda.ts';
export {
  githubRepoFromRows,
  githubTagFromRows,
  versionFromTag,
  type GithubRelease,
} from './update.resolvers.github-release.ts';
export {
  mapGithubRows,
  resolveGithubFamily,
} from './update.resolvers.github.ts';
export {
  resolveGoModule,
  resolveGoRuntime,
} from './update.resolvers.go.ts';
export {
  parseShasums,
  resolveNode,
} from './update.resolvers.node.ts';
export {
  resolveCargo,
  resolveNpm,
  resolvePypi,
} from './update.resolvers.registry.ts';
export {
  resolveRust,
  resolveRustup,
} from './update.resolvers.rust.ts';
export type {
  FetchedResponse,
  Fetcher,
  ToolResolution,
} from './update.resolvers.shared.ts';
export { resolveZigLatest, } from './update.resolvers.zig.ts';
