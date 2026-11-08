/**
 The sidecar crates beside this package, and the check that each locks every package this package locks, at the same
 version.

 A sidecar such as `package/git-policy/cli.fuzz` links this package through a path dependency, so its own `Cargo.lock`
 resolves this package's whole dependency graph. Adding, removing or moving a dependency here therefore makes that
 lock stale, and nothing in this package's checks read it: the sidecar broke unnoticed until the repository-wide lint
 reached it.

 Every resolution is read through `cargo metadata --locked --offline`, so cargo's own refusal of a lock its manifest
 refuses and a comparison of the two package lists both find a sidecar lock that lagged behind.
 */
import {
  readdir,
  stat,
} from 'node:fs/promises';
import {
  basename,
  dirname,
  join,
  resolve,
} from 'node:path';

import {
  isMissing,
  isRecord,
} from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts/fs-guards.ts';
import {
  NativeVerificationError,
  runCommand,
} from './native-verification-process.ts';

/**
 One package a lockfile names, at one version.
 */
export type LockedPackage = {
  readonly name: string;
  readonly version: string;
};

/**
 One package this package locks that a sidecar does not lock at the same version.
 */
export type StaleDependency = {
  readonly name: string;
  readonly expected: string;
  /**
   Every version the sidecar locks for that name, absent when it locks none.
   */
  readonly found?: readonly string[];
};

/**
 What one sidecar's lock disagrees with this package's.
 */
export type SidecarLockProblem = {
  /**
   The sidecar's directory name, such as `cli.fuzz`.
   */
  readonly sidecar: string;
  /**
   The sidecar's absolute directory, which names the task that repairs its lock.
   */
  readonly directory: string;
  /**
   Cargo's refusal of the sidecar's lock, empty when cargo read it.
   */
  readonly refusal: string;
  readonly stale: readonly StaleDependency[];
};

/**
 One directory's resolution, read through cargo so the lock is the lock its own manifest allows.
 */
type Resolution = {
  readonly packages: readonly LockedPackage[];
  /**
   Cargo's refusal when it refused the lock, empty when it read it.
   */
  readonly refusal: string;
};

/**
 The packages one `cargo metadata` document lists, in name and version order.

 @param document - parsed `cargo metadata --format-version 1` output

 @returns every listed package once, sorted, with a repeated name and version counted once

 @throws NativeVerificationError when the document is not the shape cargo documents

 @example
 ```ts
 const packages = lockedPackages({ document: { packages: [{ name: 'serde', version: '1.0.200' }] } });
 ```
 */
export function lockedPackages({ document, }: {
  readonly document: unknown;
},): readonly LockedPackage[] {
  if (!isRecord(document))
    throw new NativeVerificationError('cargo metadata did not print an object.');
  /**
   The package list of the document, unchecked until every entry is.
   */
  const listed = document.packages;
  if (!Array.isArray(listed))
    throw new NativeVerificationError('cargo metadata printed no package list.');
  /**
   Every package the document names, in the order it names them.
   */
  const packages: LockedPackage[] = [];
  for (const entry of listed) {
    if (!isRecord(entry))
      throw new NativeVerificationError('cargo metadata printed a package that is not an object.');
    /**
     The name and version one listed package carries.
     */
    const {
      name,
      version
    } = entry;
    if (((typeof name) !== 'string') || ((typeof version) !== 'string'))
      throw new NativeVerificationError(`cargo metadata printed a package without a name and version: ${JSON.stringify(entry)}`);
    packages.push({
      name,
      version,
    });
  }
  /**
   One entry per name and version pair, so a pair cargo lists twice is compared once.
   */
  const unique = new Map(packages.map(function keyOf(package_) {
    return [
      `${package_.name}@${package_.version}`,
      package_
    ] as const;
  }));
  return [...unique.values()].toSorted(function byNameAndVersion(
    left,
    right
  ) {
    return left.name === right.name
      ? left.version
        .localeCompare(right.version)
      : left.name
        .localeCompare(right.name);
  });
}

/**
 The packages this package locks that a sidecar does not lock at the same version.

 A version the sidecar locks beside the wanted one is allowed: its own dependencies may need another version of the
 same crate, and its resolution then holds both.

 @param subject - packages this package's lock resolves

 @param sidecar - packages the sidecar's lock resolves

 @returns one entry per package this package locks that the sidecar does not lock at that version, in subject order

 @example
 ```ts
 const stale = staleDependencies({ subject: [{ name: 'serde', version: '1.0.200' }], sidecar: [] });
 ```
 */
export function staleDependencies({
  subject,
  sidecar,
}: {
  readonly subject: readonly LockedPackage[];
  readonly sidecar: readonly LockedPackage[];
},): readonly StaleDependency[] {
  /**
   Every version the sidecar locks, by package name.
   */
  const versionsByName = new Map<string, string[]>();
  for (const package_ of sidecar) {
    /**
     The versions of this package's name the sidecar locks so far.
     */
    const versions = versionsByName.get(package_.name) ?? [];
    versions.push(package_.version);
    versionsByName.set(
      package_.name,
      versions,
    );
  }
  /**
   The packages this package locks that the sidecar does not lock at that version.
   */
  const stale: StaleDependency[] = [];
  for (const wanted of subject) {
    /**
     What the sidecar locks under this package's name, absent when it locks nothing.
     */
    const found = versionsByName.get(wanted.name);
    if ((found !== undefined) && found.includes(wanted.version))
      continue;
    stale.push({
      name: wanted.name,
      expected: wanted.version,
      ...(found === undefined ? {} : { found: [...found].toSorted(), }),
    });
  }
  return stale;
}

/**
 The directories beside a package that hold a crate: every sibling whose name starts with the package's own name and a
 dot, as a sidecar's does (repository rule `SBS`).

 @param packageDirectory - the subject package's directory, which holds its `Cargo.toml`

 @returns the sidecar directories, absolute, in name order

 @example
 ```ts
 const sidecars = await sidecarDirectories({ packageDirectory: '/repo/package/git-policy/cli' });
 ```
 */
export async function sidecarDirectories({ packageDirectory, }: {
  readonly packageDirectory: string;
},): Promise<readonly string[]> {
  /**
   The subject's absolute directory, named in every task a caller prints.
   */
  const subject = resolve(packageDirectory);
  /**
   The directory every sidecar of the package sits in.
   */
  const parent = dirname(subject);
  /**
   What a sibling's name starts with to be a sidecar of this package.
   */
  const prefix = `${basename(subject)}.`;
  /**
   Every entry beside the package.
   */
  const entries = await readdir(
    parent,
    { withFileTypes: true, },
  );
  /**
   Every sibling beside the package whose name marks it as one.
   */
  const candidates = entries
    .filter(function isSidecarDirectory(entry) {
      return (entry.isDirectory())
        && (entry.name
          .startsWith(prefix));
    })
    .map(function directoryOf(entry) {
      return join(
        parent,
        entry.name,
      );
    });
  /**
   Whether each candidate holds a manifest, in candidate order.
   */
  const holdsManifest = await Promise.all(candidates.map(function manifestOf(directory) {
    return holdsManifestAt({ directory, });
  }));
  /**
   Every candidate that holds a manifest, which is one sidecar crate.
   */
  const crates: string[] = [];
  for (const [position, directory] of candidates.entries()) {
    if (holdsManifest[position] !== true)
      continue;
    crates.push(directory);
  }
  return crates.toSorted();
}

/**
 Whether a directory holds a `Cargo.toml`.

 @param directory - absolute directory to look in

 @returns true when the directory holds that file

 @throws The failure of a lookup that is not a missing file

 @example
 ```ts
 const crate = await holdsManifestAt({ directory: '/repo/package/git-policy/cli.fuzz' });
 ```
 */
async function holdsManifestAt({ directory, }: {
  readonly directory: string;
},): Promise<boolean> {
  try {
    return (await stat(join(
      directory,
      'Cargo.toml',
    ))).isFile();
  }
  catch (error) {
    if (!isMissing(error))
      throw error;
    return false;
  }
}

/**
 What:
  The line of cargo's standard error that says what it refused.
 Why:
  Cargo prints progress, lock waits and warnings before the error,
  so the first line of the stream is rarely the refusal,
  and the refusal is the one line a person needs.

 @param stderr - everything cargo wrote on standard error

 @returns the first `error:` line, or the first non-empty line when cargo wrote none

 @example
 ```ts
 const refusal = refusalLine({ stderr: 'warning: waiting\nerror: the lock file needs to be updated\n' });
 ```
 */
export function refusalLine({ stderr, }: {
  readonly stderr: string;
},): string {
  /**
   Every non-empty line cargo wrote, trimmed.
   */
  const lines = stderr.split('\n')
    .map(function trimmed(line) {
      return line.trim();
    })
    .filter(function isNotEmpty(line) {
      return line !== '';
    });
  return lines.find(function isError(line) {
    return line.startsWith('error:');
  }) ?? lines[0]
    ?? '';
}

/**
 One directory's resolution, read through cargo so its lock is the lock its manifest allows.

 @param directory - directory holding a `Cargo.toml`

 @returns the packages the directory resolves,
   and cargo's refusal when it refused the lock instead

 @example
 ```ts
 const resolution = await readResolution({ directory: '/repo/package/git-policy/cli.fuzz' });
 ```
 */
export async function readResolution({ directory, }: {
  readonly directory: string;
},): Promise<Resolution> {
  /**
   What cargo answered, which is a refusal when it would not read the lock.
   */
  const outcome = await runCommand({
    command: 'cargo',
    args: [
      'metadata',
      '--locked',
      '--offline',
      '--format-version',
      '1',
      '--manifest-path',
      join(
        directory,
        'Cargo.toml',
      ),
    ],
    capture: true,
    allowFailure: true,
    cwd: directory,
  });
  if (outcome.status !== 0)
    return {
      packages: [],
      refusal: refusalLine({ stderr: outcome.stderr, }),
    };
  return {
    packages: lockedPackages({ document: JSON.parse(outcome.stdout), }),
    refusal: '',
  };
}

/**
 Every way the sidecars beside one package disagree with its lock.

 @param packageDirectory - the subject package's directory

 @returns the problems, empty when every sidecar locks what the package locks

 @throws NativeVerificationError when cargo refuses this package's own lock, which no sidecar can then be compared with

 @example
 ```ts
 const problems = await sidecarLockProblems({ packageDirectory: '/repo/package/git-policy/cli' });
 ```
 */
export async function sidecarLockProblems({ packageDirectory, }: {
  readonly packageDirectory: string;
},): Promise<readonly SidecarLockProblem[]> {
  /**
   The crates beside this package that link it.
   */
  const sidecars = await sidecarDirectories({ packageDirectory, });
  if (sidecars.length === 0)
    return [];
  /**
   This package's own resolution, which every sidecar's is compared with.
   */
  const subject = await readResolution({ directory: resolve(packageDirectory), });
  if (subject.refusal !== '')
    throw new NativeVerificationError(`cargo refused this package's own lock, so no sidecar can be compared with it: ${subject.refusal}`);
  /**
   One answer per sidecar, in sidecar order.
   */
  const problems = await Promise.all(sidecars.map(async function problemOf(directory): Promise<SidecarLockProblem> {
    /**
     The sidecar's resolution, which is a refusal when cargo would not read its lock.
     */
    const sidecar = await readResolution({ directory, });
    return {
      sidecar: basename(directory),
      directory,
      refusal: sidecar.refusal,
      stale: sidecar.refusal === ''
        ? staleDependencies({
          subject: subject.packages,
          sidecar: sidecar.packages,
        })
        : [],
    };
  }));
  return problems.filter(function isProblem(problem) {
    return (problem.refusal !== '') || (problem.stale
      .length
      > 0);
  });
}

/**
 The mise task that owns a directory, named from the repository-relative path every task name holds.

 @param directory - absolute package directory inside the repository's `package` folder

 @returns the task name, such as `mise run //package/git-policy/cli.fuzz`

 @example
 ```ts
 const task = miseTaskOf({ directory: '/repo/package/git-policy/cli.fuzz' });
 ```
 */
export function miseTaskOf({ directory, }: {
  readonly directory: string;
},): string {
  /**
   What every task name of the repository separates its `package` folder with.
   */
  const marker = '/package/';
  /**
   Where the directory's own package folder starts, absent when it is not inside one.
   */
  const index = directory.lastIndexOf(marker);
  /**
   The package folder and everything below it.
   */
  const relative = index === (-1)
    ? basename(directory)
    : directory.slice(index + marker.length);
  return `mise run //package/${relative}`;
}

/**
 The lines that tell a person what one sidecar's lock no longer agrees with, and what writes it again.

 @param problem - one sidecar's disagreement

 @param packageDirectory - the subject package's directory, named in the line

 @returns one line per stale package, cargo's refusal when it gave one, and the command that repairs the lock

 @example
 ```ts
 console.error(describeProblem({ problem, packageDirectory: '/repo/package/git-policy/cli' }));
 ```
 */
export function describeProblem({
  problem,
  packageDirectory,
}: {
  readonly problem: SidecarLockProblem;
  readonly packageDirectory: string;
},): string {
  /**
   The subject's directory name, which the lines compare the sidecar with.
   */
  const subject = basename(resolve(packageDirectory));
  /**
   What the person is told, which starts with the sidecar the lock belongs to.
   */
  const lines: string[] = [`${problem.sidecar}: its Cargo.lock no longer agrees with ${subject}/Cargo.lock`];
  for (const dependency of problem.stale) {
    /**
     What the sidecar locks under this package's name.
     */
    const found = dependency.found === undefined
      ? 'which it does not lock'
      : `where it locks ${dependency.found
        .join(', ')}`;
    lines.push(`  ${dependency.name}: ${subject} locks ${dependency.expected}, ${found}`);
  }
  if (problem.refusal !== '')
    lines.push(`  cargo refused the lock: ${problem.refusal}`);
  lines.push(`Run \`${miseTaskOf({ directory: problem.directory, })}:lock:members\` to record these versions, then commit its Cargo.lock.`);
  return lines.join('\n');
}
