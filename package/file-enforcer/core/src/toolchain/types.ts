/**
 Shared types for the typed toolchain plan and its generated artifacts.
 */

import type { ToolArtifacts, } from './plan.artifacts.data.ts';

//region Request intent

/**
 Request intent for one tool, exactly the three shapes
 `doc/planning/mpm-toolchain-migration.md` fixes for `toolchain.lock.json`.
 */
export type ToolRequest =
  | { readonly track: 'latest'; }
    | { readonly pinned: string; }
    | { readonly channel: string; };

//endregion Request intent

//region Routes

/**
 Install route for one tool
 (doc/planning/mpm-toolchain-migration.md, section `Routing (user-confirmed 2026-10-26)`;
 the `cargo-git` and `go-install` routes come from
 `doc/planning/mpm-global-toolchain-migration.md`).

 `bin-sourced` tools install with the `bin` CLI at the repository boundary;
 `archive` is the fallback for sources and payloads `bin` cannot fetch in one
 call. `rustup-toolchain` and `rustup-components` are also the step routes the
 sync algorithm runs for the Rust toolchain and its component/target phase.
 `cargo-git` installs one package from a git source at a pinned revision, and
 `go-install` builds one Go module at its locked version into a scoped
 `GOBIN`.
 */
export type ToolRoute =
  | 'bin-sourced'
    | 'archive'
    | 'cargo-binstall'
    | 'cargo-git'
    | 'mpm-pnpm'
    | 'mpm-uvx'
    | 'go-install'
    | 'rustup-toolchain'
    | 'rustup-components'
    | 'android-sdk';

/**
 Every route value, in declaration order, for exhaustiveness checks and tests.
 */
export const TOOL_ROUTES: readonly ToolRoute[] = [
  'bin-sourced',
  'archive',
  'cargo-binstall',
  'cargo-git',
  'mpm-pnpm',
  'mpm-uvx',
  'go-install',
  'rustup-toolchain',
  'rustup-components',
  'android-sdk',
] as const;

//endregion Routes

//region Options

/**
 Atomic link step run after install
 (the `cli-git-bin` postinstall pattern:
 temporary name plus rename over the final name).
 */
export type PostinstallLink = {
  readonly when: 'always';
  readonly link: {
    readonly sourceBin: string;
    readonly linkName: string;
    /**
     Worktree-relative or absolute directory holding the link; absent means
     the scope's own link directory (`paths.cliGitBinDir`).
     */
    readonly linkDir?: string;
  };
};

/**
 Provider options one tool carries
 (the decision doc's `provider options` column of `toolchain.lock.json`).
 */
export type ToolOptions = {
  /**
   `rustup component add` names applied after the Rust toolchain lands.
   */
  readonly rustComponents?: readonly string[];
  /**
   `rustup target add` triples applied after the Rust toolchain lands.
   */
  readonly rustTargets?: readonly string[];
  /**
   `cargo binstall --features` values (source-build fallback only).
   */
  readonly cargoFeatures?: readonly string[];
  /**
   Git source of a `cargo-git` tool (`cargo install --git <url>`), with the
   pinned revision riding in `request.pinned` as `ref:<rev>` and the package
   name in `packageId`.
   */
  readonly cargoGit?: {
    readonly url: string;
  };
  /**
   Release-asset glob the GitHub resolver matches when it records artifact
   rows (`asset_pattern` / `matching` in the retired Mise config), for
   releases whose asset names carry no platform marker.
   */
  readonly assetPattern?: string;
  /**
   Packages whose install scripts the `mpm-pnpm` provider allows
   (`allow_builds` in the retired Mise config).
   */
  readonly allowBuilds?: readonly string[];
  /**
   `sdkmanager` component pins installed after cmdline-tools.
   */
  readonly androidPackages?: readonly string[];
  /**
   Postinstall atomic link step.
   */
  readonly postinstall?: PostinstallLink;
  /**
   Extra exposure names that link the same target as another declared bin
   (`bunx` for `bun`, `pn`/`pnpx`/`pnx` for `pnpm`): alias name to primary
   bin name. `bin install` copies one file per invocation, so aliases are
   linked instead of re-downloaded.
   */
  readonly binAliases?: Readonly<Record<string, string>>;
  /**
   Bootstrap metadata for the pinned `mpm` and `bin` prerequisites.
   */
  readonly bootstrap?: {
    /**
     Tool that must exist before this one installs, or `null`.
     */
    readonly requiresTool: string | null;
    /**
     Distribution kind the provider installs.
     */
    readonly distribution: 'wheel' | 'standalone-binary';
  };
};

//endregion Options

//region Plan entry

/**
 Environment variables one tool contributes to
 `.cache/toolchain.env.json`.

 Values are either literal strings (`RUSTUP_TOOLCHAIN`) or toolchain-root
 relative paths (`java/<version>`), never machine-absolute paths.
 */
export type ToolEnv = Readonly<Record<string, string>>;

/**
 One typed tool plan entry: request intent, resolved version, route,
 mpm manager identity, exposure bins, OS filter, provider options,
 tool environment exports, and per-platform artifact provenance.
 */
export type ToolPlanEntry = {
  /**
   Stable tool id, the `mise.no-env.toml` key (plus `mpm` for the bootstrap).
   */
  readonly id: string;
  readonly request: ToolRequest;
  /**
   Resolved version locked from `mise.lock` (or the mpm release).
   */
  readonly version: string;
  readonly route: ToolRoute;
  /**
   MPM manager id declaring this tool in `mpm.packages.toml`, or `null`.
   */
  readonly manager: string | null;
  /**
   Package id under `manager`, or `null` when no MPM manager applies.
   */
  readonly packageId: string | null;
  /**
   Binary names the per-worktree exposure directory links.
   */
  readonly bins: readonly string[];
  /**
   OS filter carried over from the tool's `os` option, or `null`.
   */
  readonly os: readonly ('linux' | 'macos' | 'windows')[] | null;
  readonly options: ToolOptions;
  readonly env: ToolEnv;
  readonly artifacts: ToolArtifacts;
};

//endregion Plan entry

//region Generated artifact shapes

/**
 `toolchain.lock.json` tool record: the plan entry as a JSON record
 with stable key order (see `lock.ts` for the renderer).
 */
export type ToolchainLockTool = {
  readonly request: ToolRequest;
  readonly version: string;
  readonly route: ToolRoute;
  readonly manager: string | null;
  readonly packageId: string | null;
  readonly bins: readonly string[];
  readonly os: readonly ('linux' | 'macos' | 'windows')[] | null;
  readonly env: ToolEnv;
  readonly options: ToolOptions;
  readonly artifacts: ToolArtifacts;
};

/**
 `toolchain.lock.json` top level.
 */
export type ToolchainLock = {
  readonly schemaVersion: 1;
  readonly generatedFrom: 'package/file-enforcer/core/src/toolchain/plan.ts';
  readonly tools: Readonly<Record<string, ToolchainLockTool>>;
};

//endregion Generated artifact shapes
