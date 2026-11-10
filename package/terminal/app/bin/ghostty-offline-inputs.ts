// Pure rules of the libghostty offline inputs: which pins the manifest must hold, what a download must match, and how a
// pin is read from the crate that owns it. The fetch task (fetch-ghostty-offline-inputs.ts) performs the effects.
// Why: libghostty-vt-sys otherwise clones Ghostty and lets `zig build` download Zig packages while Cargo compiles. Zig
// verifies its own content hash, but its downloader cannot connect on this host (see doc/troubleshooting/
// zig-downloader-cannot-connect.md), and nothing in the repository recorded an archive checksum.

import { createHash } from 'node:crypto';

/**
 * One Zig package that the Ghostty library build needs, pinned by the SHA-256 of its archive.
 */
export interface PinnedPackage {
  readonly name: string;
  /** The `.url` of the package in a build.zig.zon file of Ghostty or of one of its packages. */
  readonly source: string;
  /** The https address of the archive that is downloaded. */
  readonly archive: string;
  readonly sha256: string;
  /** The `.hash` of the package in the zon file, which names the directory Zig reads the package from. */
  readonly zigHash: string;
}

/**
 * The checked-in manifest of the inputs.
 */
export interface OfflineInputsManifest {
  readonly ghosttyRepository: string;
  readonly ghosttyCommit: string;
  readonly packages: readonly PinnedPackage[];
}

const commitPattern = /^[0-9a-f]{40}$/;
const sha256Pattern = /^[0-9a-f]{64}$/;
const pinnedCommitInBuildScript = /const GHOSTTY_COMMIT: &str = "([0-9a-f]{40})";/;

/**
 * Validate parsed manifest text and return it typed.
 *
 * Every archive must be https and carry a SHA-256, and every Ghostty pin must be a full commit, so no input can be
 * fetched without a checksum or by a movable name.
 *
 * @param text - the manifest file's text.
 * @returns The manifest.
 * @throws Error naming the first field that breaks a rule.
 */
export function parseManifest(text: string): OfflineInputsManifest {
  const parsed: unknown = JSON.parse(text);
  if (typeof parsed !== 'object' || parsed === null) { throw new Error('The manifest must be a JSON object.') }
  const manifest = parsed as Partial<OfflineInputsManifest>;
  if (typeof manifest.ghosttyRepository !== 'string' || !manifest.ghosttyRepository.startsWith('https://')) {
    throw new Error('ghosttyRepository must be an https address.')
  }
  if (typeof manifest.ghosttyCommit !== 'string' || !commitPattern.test(manifest.ghosttyCommit)) {
    throw new Error('ghosttyCommit must be a full 40-digit commit id.')
  }
  if (!Array.isArray(manifest.packages) || manifest.packages.length === 0) {
    throw new Error('packages must list at least one package.')
  }
  const directories = new Set<string>()
  for (const entry of manifest.packages as Partial<PinnedPackage>[]) {
    const label = String(entry.name)
    if (typeof entry.name !== 'string' || entry.name === '') { throw new Error('Every package needs a name.') }
    if (typeof entry.archive !== 'string' || !entry.archive.startsWith('https://')) {
      throw new Error(`${label}: archive must be an https address.`)
    }
    if (typeof entry.source !== 'string' || entry.source === '') { throw new Error(`${label}: source is missing.`) }
    if (typeof entry.sha256 !== 'string' || !sha256Pattern.test(entry.sha256)) {
      throw new Error(`${label}: sha256 must be 64 lowercase hex digits; a package without a checksum is never fetched.`)
    }
    if (typeof entry.zigHash !== 'string' || entry.zigHash === '' || /[/\\]/.test(entry.zigHash) || entry.zigHash.startsWith('.')) {
      throw new Error(`${label}: zigHash must be a plain directory name.`)
    }
    if (directories.has(entry.zigHash)) { throw new Error(`${label}: zigHash ${entry.zigHash} is listed twice.`) }
    directories.add(entry.zigHash)
  }
  return manifest as OfflineInputsManifest
}

/**
 * Read the Ghostty commit that libghostty-vt-sys builds from its build script.
 *
 * @param buildScript - text of the crate's build.rs.
 * @returns The 40-digit commit.
 * @throws Error when the script holds no such pin, so a changed crate cannot pass unnoticed.
 */
export function pinnedGhosttyCommit(buildScript: string): string {
  const match = pinnedCommitInBuildScript.exec(buildScript)
  if (match === null) { throw new Error('libghostty-vt-sys build.rs no longer declares `const GHOSTTY_COMMIT: &str = "<commit>";`; update the offline-inputs fetch.') }
  return match[1]!
}

/**
 * SHA-256 of bytes as lowercase hex.
 *
 * @param bytes - the downloaded archive.
 * @returns The digest.
 */
export function sha256Of(bytes: Uint8Array): string {
  return createHash('sha256').update(bytes).digest('hex')
}

/**
 * Compare a download with its pin.
 *
 * @param options - `bytes` is the archive and `package` its manifest entry.
 * @throws Error quoting both digests when they differ.
 */
export function verifyArchive({ bytes, pinned }: { readonly bytes: Uint8Array; readonly pinned: PinnedPackage }): void {
  const actual = sha256Of(bytes)
  if (actual !== pinned.sha256) {
    throw new Error(`${pinned.name}: ${pinned.archive} has sha256 ${actual}, but the manifest pins ${pinned.sha256}. Nothing was installed from it.`)
  }
}

/**
 * The text recorded in the ready marker: the pins the finished inputs were built from.
 *
 * @param manifestText - the manifest file's exact text.
 * @returns A digest line that changes whenever any pin changes.
 */
export function readyMarkerFor(manifestText: string): string {
  return `${sha256Of(new TextEncoder().encode(manifestText))}\n`
}

/**
 * Whether a ready marker may be trusted: it must carry the digest of the current pins.
 *
 * @param options - `markerText` is what the marker file holds, or undefined when no marker exists.
 * @returns True only for a marker written from the same pins.
 */
export function readyMarkerIsCurrent({ markerText, expectedMarkerText }: { readonly markerText: string | undefined; readonly expectedMarkerText: string }): boolean {
  return markerText === expectedMarkerText
}

/**
 * The paths, relative to the offline-inputs root, that a finished fetch leaves behind.
 *
 * The Ghostty checkout is named by its build.zig, because that is the file libghostty-vt-sys asserts inside
 * GHOSTTY_SOURCE_DIR before it builds, and each Zig package by the directory Zig reads it from.
 *
 * @param manifest - the parsed manifest.
 * @returns One relative path per promised input.
 */
export function promisedInputPaths(manifest: OfflineInputsManifest): readonly string[] {
  return ['source/build.zig', ...manifest.packages.map((pinned) => `packages/${pinned.zigHash}`)]
}

/**
 * The promised inputs that are absent from disk.
 *
 * A marker alone stands for nothing: a worktree that kept the gitignored marker while its target directory lost the
 * source tree would otherwise report the inputs as current and the build would stop.
 *
 * @param options - `isPresent` answers whether a relative path exists under the offline-inputs root.
 * @returns The missing relative paths, empty when every promised input is present.
 */
export function missingInputs({ manifest, isPresent }: { readonly manifest: OfflineInputsManifest; readonly isPresent: (relativePath: string) => boolean }): readonly string[] {
  return promisedInputPaths(manifest).filter((relativePath) => !isPresent(relativePath))
}
