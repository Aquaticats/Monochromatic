// Build the gate image of the ZFS binding package from this checkout alone.
//
// What:  assembles the build context `gate.Containerfile` reads: the two
//        toolchain.lock.json binaries (file-enforcer and monochromatic-lint,
//        fetched and checked against the lock's sha256), the crate under
//        deps/subject and its fuzz sidecar under deps/sidecar, and then builds
//        the image with podman under a tag this script prints.
// Why:   step 1's gate review found no repository file assembled that context, so
//        the image could not be rebuilt from the repository alone; this script is
//        that owner, and it never types the base image's tag: the wrapper's
//        resolver prints it.
//
// Usage: node build-gate-image.ts [--tag <tag>] [--base <tag>] [--keep-context]
// Prints the built image's tag last.

import { spawnSync, } from 'node:child_process';
import { chmodSync, existsSync, mkdirSync, readFileSync, readdirSync, rmSync, } from 'node:fs';
import { join, } from 'node:path';

//region Constants

/** The image this package's step 2 gate runs in; never the plain `development`. */
const DEFAULT_TAG = 'zfs-request-to-openzfs-kernel-device-gate:zfs-step-2-live-requests-2026-11-19';
/** The two lock rows whose linux-x64 archives the gate image needs. */
const LOCKED_TOOLS = ['cargo:monochromatic-file-enforcer-cli', 'cargo:monochromatic-lint'] as const;
/** The binaries those rows install, which the Containerfile copies by name. */
const BINARIES: Record<string, string> = {
  'cargo:monochromatic-file-enforcer-cli': 'file-enforcer',
  'cargo:monochromatic-lint': 'monochromatic-lint',
};
/** The platform whose archive every host of this gate runs. */
const LOCKED_PLATFORM = 'linux-x64';

//endregion Constants

//region Arguments

/** The tag to build, the base image if given, and whether the context stays. */
function options(): { readonly tag: string; readonly base: string | undefined; readonly keep: boolean } {
  const rest = process.argv.slice(2);
  const value = (flag: string): string | undefined => {
    const at = rest.indexOf(flag);
    return at === -1 ? undefined : rest[at + 1];
  };
  return { tag: value('--tag') ?? DEFAULT_TAG, base: value('--base'), keep: rest.includes('--keep-context'), };
}

//endregion Arguments

//region Commands

/** Run one host command and return its standard output. */
function capture(command: string, args: readonly string[], cwd: string | undefined = undefined): string {
  const result = spawnSync(command, args, { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024, cwd, });
  if (result.error || result.status !== 0)
    throw result.error ?? new Error(`${command} ${args.join(' ')} exited ${result.status}: ${result.stderr}`);
  return result.stdout;
}

/** Run one host command with inherited output. */
function run(command: string, args: readonly string[]): void {
  const result = spawnSync(command, args, { stdio: 'inherit', });
  if (result.error || result.status !== 0)
    throw result.error ?? new Error(`${command} ${args.join(' ')} exited ${result.status}`);
}

//endregion Commands

//region Context

/** One lock row's artifact: where to fetch it and the sha256 it must match. */
interface Artifact {
  readonly url: string;
  readonly sha256: string;
}

/** The lock row's linux-x64 artifact, or an error naming the row. */
function artifactFor(row: string): Artifact {
  const lock = JSON.parse(readFileSync(join(repoRoot(), 'toolchain.lock.json'), 'utf8')) as {
    readonly tools: Record<string, { readonly artifacts?: Record<string, Artifact> }>;
  };
  const artifact = lock.tools[row]?.artifacts?.[LOCKED_PLATFORM];
  if (!artifact)
    throw new Error(`toolchain.lock.json has no ${LOCKED_PLATFORM} artifact for ${row}`);
  return artifact;
}

/** The directory this script's files live under. */
function repoRoot(): string {
  return join(import.meta.dirname, '..', '..', '..');
}

/** The repository-relative path of this package. */
function packageDir(): string {
  return import.meta.dirname;
}

/** Fetch one locked binary into the context and verify it. */
function fetchBinary(context: string, row: string): void {
  const artifact = artifactFor(row);
  const bin = BINARIES[row];
  if (!bin)
    throw new Error(`no binary name is known for ${row}`);
  const archive = join(context, `${bin}.tar.gz`);
  run('curl', ['--fail', '--silent', '--show-error', '--location', '--output', archive, artifact.url]);
  const listed = capture('sha256sum', [archive]).split(' ')[0];
  if (listed !== artifact.sha256)
    throw new Error(`${row}: sha256 ${listed} is not the lock's ${artifact.sha256}`);
  run('tar', ['--extract', '--gzip', '--file', archive, '--directory', context]);
  rmSync(archive);
  const installed = findBinary(context, bin);
  if (!installed)
    throw new Error(`${row}: the archive holds no ${bin}`);
  chmodSync(installed, 0o755);
  if (installed !== join(context, bin))
    run('mv', [installed, join(context, bin)]);
  console.log(`${bin}: ${capture(join(context, bin), ['--version']).trim()}`);
}

/** The binary inside the extracted archive, which the release names a directory. */
function findBinary(context: string, bin: string): string | undefined {
  const direct = join(context, bin);
  if (existsSync(direct))
    return direct;
  for (const entry of readdirSync(context, { withFileTypes: true, })) {
    if (!entry.isDirectory())
      continue;
    const nested = join(context, entry.name, bin);
    if (existsSync(nested))
      return nested;
  }
  return undefined;
}

/** Copy the two crates the Containerfile's `deps/` layers read. */
function copyCrates(context: string): void {
  const source = packageDir();
  mkdirSync(join(context, 'deps', 'subject'), { recursive: true, mode: 0o700, });
  mkdirSync(join(context, 'deps', 'sidecar'), { recursive: true, mode: 0o700, });
  run('rsync', ['--archive', '--exclude', 'target', `${source}/`, join(context, 'deps', 'subject', '')]);
  run('rsync', ['--archive', '--exclude', 'target', `${source}.fuzz/`, join(context, 'deps', 'sidecar', '')]);
}

//endregion Context

//region Build

/**
 The base image the wrapper's resolver prints, built first when absent.
 A `--base` that names another tag is refused instead of used: the resolver owns
 the tag, so building on anything else would build an image the next run of
 this script could not rebuild.
 */
function baseImage(given: string | undefined): string {
  const printed = capture('mise', ['run', '//package/git-policy/cli:test:git-image'], repoRoot());
  const match = /(?:Built|Found) the gate base (\S+)/.exec(printed);
  const tag = match?.[1];
  if (tag === undefined || tag === '')
    throw new Error(`the base image resolver printed no tag:\n${printed}`);
  if (given !== undefined && given !== '' && given !== tag)
    throw new Error(
      `--base names ${given}, and the wrapper's resolver prints ${tag} ` +
        `(mise run //package/git-policy/cli:test:git-image); drop --base or pass the tag it prints`,
    );
  return tag;
}

//endregion Build

//region Entry point

const chosen = options();
const context = join(packageDir(), 'target', 'gate-image-context');
rmSync(context, { recursive: true, force: true, });
mkdirSync(context, { recursive: true, mode: 0o700, });
try {
  for (const row of LOCKED_TOOLS)
    fetchBinary(context, row);
  copyCrates(context);
  run('cp', [join(packageDir(), 'gate.Containerfile'), join(context, 'gate.Containerfile')]);
  const base = baseImage(chosen.base);
  console.log(`base image: ${base}`);
  run('podman', [
    'build',
    '--file',
    join(context, 'gate.Containerfile'),
    '--build-arg',
    `GATE_BASE_IMAGE=${base}`,
    '--tag',
    chosen.tag,
    context,
  ]);
} finally {
  // A failed build leaves the context as garbage in `target/` unless it goes
  // here too; `--keep-context` is the one reason to keep it.
  if (!chosen.keep)
    rmSync(context, { recursive: true, force: true, });
}
console.log(chosen.tag);

//endregion Entry point
