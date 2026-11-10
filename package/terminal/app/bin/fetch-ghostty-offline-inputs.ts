// Fetch task of the libghostty offline inputs: `mise run //package/terminal/app:fetch:ghostty-offline-inputs`.
// What: it leaves target/ghostty-offline/source (Ghostty at the commit libghostty-vt-sys pins), target/ghostty-offline/packages/<zig hash>
// (every Zig package the library build needs, each from an archive whose SHA-256 the manifest pins) and, last,
// target/ghostty-offline/ready. libghostty-vt-sys then reads them through GHOSTTY_SOURCE_DIR and GHOSTTY_ZIG_SYSTEM_DIR
// and downloads nothing while Cargo compiles; the closure check of monochromatic-lint sets both variables itself.
// Why: Zig's own downloader cannot connect on this host, and a build-time download had no checksum of this repository.
// Run from the package directory. The task is idempotent: a finished set of unchanged pins is left alone, and the
// ready marker counts only while the source tree and package directories it promises are still there.

import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, readdirSync, renameSync, rmSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { missingInputs, parseManifest, pinnedGhosttyCommit, readyMarkerFor, readyMarkerIsCurrent, sha256Of, verifyArchive } from './ghostty-offline-inputs.ts';
import type { OfflineInputsManifest, PinnedPackage } from './ghostty-offline-inputs.ts';

const manifestFile = 'ghostty-offline-inputs.json'
const root = resolve('target', 'ghostty-offline')

/**
 * The commit pinned by the libghostty-vt-sys crate that the lock file selects, read from the crate's build script in
 * the Cargo cache.
 */
function lockedGhosttyCommit(): string {
  const metadata = JSON.parse(execFileSync('cargo', ['metadata', '--format-version', '1', '--locked', '--offline'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'], maxBuffer: 256 * 1024 * 1024 }))
  const crate = metadata.packages.find((candidate: { name: string }) => candidate.name === 'libghostty-vt-sys')
  if (crate === undefined) { throw new Error('cargo metadata lists no libghostty-vt-sys; fetch the crate with `mise run fetch:rust-dependencies -- package/terminal/app`.') }
  return pinnedGhosttyCommit(readFileSync(join(dirname(crate.manifest_path), 'build.rs'), 'utf8'))
}

/**
 * Put Ghostty at the pinned commit in target/ghostty-offline/source, and prove it by asking Git for HEAD.
 */
function prepareSource(manifest: OfflineInputsManifest): void {
  const source = join(root, 'source')
  if (!existsSync(join(source, '.git'))) {
    rmSync(source, { recursive: true, force: true })
    mkdirSync(root, { recursive: true })
    execFileSync('git', ['clone', '--config', 'core.longpaths=true', '--filter=blob:none', '--no-checkout', manifest.ghosttyRepository, source], { stdio: 'inherit' })
  }
  const head = () => execFileSync('git', ['-C', source, 'rev-parse', 'HEAD'], { encoding: 'utf8' }).trim()
  let current = ''
  try { current = head() } catch (error) { console.error('The source tree has no readable HEAD yet:', error) }
  if (current !== manifest.ghosttyCommit) {
    execFileSync('git', ['-C', source, 'checkout', manifest.ghosttyCommit], { stdio: 'inherit' })
    current = head()
  }
  if (current !== manifest.ghosttyCommit) { throw new Error(`Ghostty source is at ${current}, not the pinned ${manifest.ghosttyCommit}.`) }
}

/**
 * Download one archive, verify its SHA-256 against the pin, and extract it to packages/<zig hash>.
 *
 * Zig names a package directory by the `.hash` of its zon file and reads it without hashing it again in system mode,
 * so the pinned SHA-256 is the integrity check of this step. As a second, independent check the archive is also given
 * to `zig fetch`, which prints the content hash that upstream's zon file declares.
 */
async function installPackage(pinned: PinnedPackage): Promise<void> {
  const archives = join(root, 'archives')
  mkdirSync(archives, { recursive: true })
  const archiveFile = join(archives, pinned.archive.split('/').pop()!)
  let bytes: Uint8Array | undefined
  if (existsSync(archiveFile)) {
    const cached = readFileSync(archiveFile)
    if (sha256Of(cached) === pinned.sha256) { bytes = cached }
  }
  if (bytes === undefined) {
    const response = await fetch(pinned.archive)
    if (!response.ok) { throw new Error(`${pinned.name}: ${pinned.archive} answered ${response.status}.`) }
    bytes = new Uint8Array(await response.arrayBuffer())
    verifyArchive({ bytes, pinned })
    writeFileSync(archiveFile, bytes)
  }
  const scratch = join(root, 'zig-cross-check')
  mkdirSync(scratch, { recursive: true })
  const printed = execFileSync('zig', ['fetch', archiveFile], { encoding: 'utf8', env: { ...process.env, ZIG_GLOBAL_CACHE_DIR: scratch }, stdio: ['ignore', 'pipe', 'inherit'] }).trim()
  if (printed !== pinned.zigHash) { throw new Error(`${pinned.name}: zig computes the content hash ${printed} for the verified archive, but the manifest records ${pinned.zigHash}.`) }
  const destination = join(root, 'packages', pinned.zigHash)
  if (existsSync(destination)) { return }
  const partial = `${destination}.partial`
  rmSync(partial, { recursive: true, force: true })
  mkdirSync(partial, { recursive: true })
  execFileSync('tar', ['--extract', '--gzip', '--strip-components=1', '--file', archiveFile, '--directory', partial])
  if (readdirSync(partial).length === 0) { throw new Error(`${pinned.name}: the archive extracted to nothing.`) }
  renameSync(partial, destination)
}

async function main(): Promise<void> {
  const manifestText = readFileSync(manifestFile, 'utf8')
  const manifest = parseManifest(manifestText)
  const locked = lockedGhosttyCommit()
  if (locked !== manifest.ghosttyCommit) {
    throw new Error(`libghostty-vt-sys builds Ghostty ${locked}, but ${manifestFile} pins ${manifest.ghosttyCommit}. Re-derive the Zig packages for ${locked} (every url and hash in the build.zig.zon files that \`zig build -Demit-lib-vt=true\` reads), pin each archive's SHA-256, and update the manifest.`)
  }
  const marker = join(root, 'ready')
  const expected = readyMarkerFor(manifestText)
  const markerText = existsSync(marker) ? readFileSync(marker, 'utf8') : undefined
  const missing = missingInputs({ manifest, isPresent: (relativePath) => existsSync(join(root, relativePath)) })
  if (readyMarkerIsCurrent({ markerText, expectedMarkerText: expected }) && missing.length === 0) {
    console.error('The libghostty offline inputs are current for these pins.')
    return
  }
  if (missing.length > 0) {
    console.error(`The offline inputs are missing ${missing.join(', ')}, so they are fetched again.`)
  }
  rmSync(marker, { force: true })
  prepareSource(manifest)
  rmSync(join(root, 'packages'), { recursive: true, force: true })
  for (const pinned of manifest.packages) { await installPackage(pinned) }
  writeFileSync(marker, expected)
  console.error(`The libghostty offline inputs are ready in ${root} (${manifest.packages.length} packages, Ghostty ${manifest.ghosttyCommit}).`)
}

if (import.meta.main) { await main() }
