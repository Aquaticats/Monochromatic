/**
 Cache-friendly image build steps shared by the gate image's compile stages.

 A stage that copies the sources and compiles in one step recompiles every dependency whenever any copied file
 changes. These helpers split the compile in two: a dependency layer built from the manifests, the lock files and
 empty stand-ins for each crate's targets, then a source layer that copies the real sources over the stand-ins and
 compiles only the crates the repository owns. A source-only change then reuses the dependency layer, because the
 layer's cache key is the manifests, the lock files and the vendored registry sources, never a source file.
 */
import {
  copyFile,
  mkdir,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
  relative,
  resolve,
} from 'node:path';

import {
  NativeVerificationError,
  runCommand,
} from './native-verification-process.ts';

/**
 Directory below a stage's tree root that mirrors the image's `/work`: the stand-in `package` tree, the vendored `vendor`
 and the Cargo mapping `.cargo`, which the dependency layer copies in one step.
 */
export const imageRootDirectory = 'image-root';

/**
 Image path of the repository-shaped tree every compile stage builds in.
 */
export const imagePackageRoot = '/work/package';

/**
 Contents of the stand-in for a target that only declares items: an empty crate root.
 */
const emptyCrateRoot = '';

/**
 Contents of the stand-in for a target that needs an entry point.
 */
const emptyMainCrateRoot = 'fn main() {}\n';

/**
 Target kinds Cargo reports that a stand-in is written for, with the stand-in's contents.
 */
const standInByKind = new Map<string, string>([
  [
    'lib',
    emptyCrateRoot,
  ],
  [
    'rlib',
    emptyCrateRoot,
  ],
  [
    'test',
    emptyCrateRoot,
  ],
  [
    'bin',
    emptyMainCrateRoot,
  ],
  [
    'example',
    emptyMainCrateRoot,
  ],
  [
    'custom-build',
    emptyMainCrateRoot,
  ],
]);

/**
 One target of a package as `cargo metadata` reports it.
 */
type MetadataTarget = {
  readonly kind: readonly string[];
  readonly src_path: string;
};

/**
 Whether a parsed JSON value is an object whose members can be read by name.

 @param value - parsed JSON value

 @returns a type guard
 */
function isRecord(value: unknown): value is Record<string, unknown> {
  return ((typeof value) === 'object') && (value !== null);
}

/**
 Whether a parsed JSON value is a list of strings.

 @param value - parsed JSON value

 @returns a type guard
 */
function isStringList(value: unknown): value is string[] {
  if (!Array.isArray(value))
    return false;
  return value.every(function isString(item) {
    return (typeof item) === 'string';
  });
}

/**
 Whether a parsed JSON value is a target with the fields this module reads.

 @param value - parsed JSON value

 @returns a type guard
 */
function isMetadataTarget(value: unknown): value is MetadataTarget {
  if (!isRecord(value))
    return false;
  return isStringList(value.kind) && ((typeof value.src_path) === 'string');
}

/**
 Narrow a parsed `cargo metadata --no-deps` document to the targets this module reads.

 @param value - parsed JSON document

 @returns the targets of every package of the document

 @throws NativeVerificationError when the document lacks the fields this module reads
 */
function metadataTargets(value: unknown): readonly MetadataTarget[] {
  if (!isRecord(value))
    throw new NativeVerificationError('Cargo metadata did not return an object.');
  if (!Array.isArray(value.packages))
    throw new NativeVerificationError('Cargo metadata did not return a package list.');
  /**
   Packages of the document, each still unchecked.
   */
  const packages: readonly unknown[] = value.packages;
  return packages.flatMap(function packageTargets(entry) {
    if (!isRecord(entry))
      throw new NativeVerificationError('Cargo metadata returned a package that is not an object.');
    if (!Array.isArray(entry.targets))
      throw new NativeVerificationError('Cargo metadata returned a package without targets.');
    /**
     Targets of the package, each still unchecked.
     */
    const targets: readonly unknown[] = entry.targets;
    return targets.map(function checkedTarget(target) {
      if (!isMetadataTarget(target))
        throw new NativeVerificationError('Cargo metadata returned a target without kind and source path.');
      return target;
    });
  });
}

/**
 Write the stand-ins of one crate.

 @param treeRoot - directory holding the real tree and receiving the stand-in tree

 @param crate - crate directory below `package`

 @throws NativeVerificationError when Cargo reports a target outside the crate or of a kind without a stand-in
 */
async function writeCrateStandIns({
  treeRoot,
  crate,
}: {
  readonly treeRoot: string;
  readonly crate: string;
}): Promise<void> {
  /**
   The crate's directory in the real tree.
   */
  const realDirectory = join(
    treeRoot,
    'package',
    crate,
  );
  /**
   The crate's directory in the stand-in tree.
   */
  const stubDirectory = join(
    treeRoot,
    imageRootDirectory,
    'package',
    crate,
  );
  await mkdir(
    stubDirectory,
    { recursive: true },
  );
  /**
   Cargo's own list of the crate's targets, so the stand-ins follow the manifest and never a copy of its rules.
   */
  const described = await runCommand({
    command: 'cargo',
    args: [
      'metadata',
      '--no-deps',
      '--format-version',
      '1',
      '--offline',
      '--locked',
      '--manifest-path',
      join(
        realDirectory,
        'Cargo.toml',
      ),
    ],
    capture: true,
  });
  /**
   Stand-in files to write, each with its contents.
   */
  const standIns = metadataTargets(JSON.parse(described.stdout) as unknown)
    .map(function standInOf(target) {
      /**
       Where the target's crate root sits relative to its crate.
       */
      const inside = relative(
        realDirectory,
        resolve(target.src_path),
      );
      if (inside.startsWith('..'))
        throw new NativeVerificationError(`Target ${target.src_path} of ${crate} lies outside the crate.`);
      /**
       Kind of the target that selects its stand-in.
       */
      const kind = target.kind
        .find(function hasStandIn(candidate) {
          return standInByKind.has(candidate);
        });
      /**
       Contents written for the target's kind.
       */
      const contents = (kind === undefined)
        ? undefined
        : standInByKind.get(kind);
      if (contents === undefined) {
        /**
         Kinds Cargo reported for the target, named in the error.
         */
        const kinds = target.kind
          .join(', ');
        throw new NativeVerificationError(`Target ${inside} of ${crate} has kinds ${kinds}, none with a stand-in.`);
      }
      return {
        file: join(
          stubDirectory,
          inside,
        ),
        contents,
      };
    });
  await Promise.all([
    ...[
      'Cargo.toml',
      'Cargo.lock',
    ].map(function copyManifest(name) {
      return copyFile(
        join(
          realDirectory,
          name,
        ),
        join(
          stubDirectory,
          name,
        ),
      );
    }),
    ...standIns.map(async function writeStandIn(standIn: Readonly<{
      file: string;
      contents: string;
    }>) {
      await mkdir(
        dirname(standIn.file),
        { recursive: true },
      );
      await writeFile(
        standIn.file,
        standIn.contents,
      );
    }),
  ]);
}

/**
 Write the stand-in tree of one stage: for every crate, its manifest and lock file copied byte for byte, plus an
 empty source file at the path of each target the manifest declares or Cargo discovers. The tree lets Cargo resolve
 and compile the dependency graph while the real sources are still absent.

 @param treeRoot - directory holding the stage's real `package/<crate>` tree, whose manifests are read, and receiving
   the stand-in tree below `image-root/package/<crate>`

 @param crates - crate directories below `package`, such as `git-policy/cli`

 @throws NativeVerificationError when Cargo reports a target outside its crate or of a kind without a stand-in

 @example
 ```ts
 await writeDependencyStubs({ treeRoot: context, crates: ['git-policy/cli'] });
 ```
 */
export async function writeDependencyStubs({
  treeRoot,
  crates,
}: {
  readonly treeRoot: string;
  readonly crates: readonly string[];
}): Promise<void> {
  await Promise.all(crates.map(function writeStandIns(crate) {
    return writeCrateStandIns({
      treeRoot,
      crate,
    });
  }));
}

/**
 Quote one word for the shell command of a `RUN` step.

 @param word - text to pass as a single shell word

 @returns the word in single quotes, with each embedded quote closed, escaped and reopened
 */
function shellWord(word: string): string {
  return `'${word.replaceAll(
    '\'',
    String.raw`'\''`,
  )}'`;
}

/**
 `RUN` step that marks the real sources as newer than the stand-in build, then runs Cargo. Files copied into an image
 keep the build context's modification times, which precede the dependency layer's build; Cargo compares those times
 with its recorded outputs and would treat the stand-in build as current. Touching the copied files in the same step
 makes the crates they belong to rebuild, and no other crate.

 @param touchedPaths - image paths of the copied sources, files or directories searched for files

 @param cargoArguments - Cargo arguments of the compile

 @returns one `RUN` instruction in exec form

 @example
 ```ts
 const line = touchedCargoRun({ touchedPaths: ['/work/package/a/src'], cargoArguments: ['build'] });
 ```
 */
export function touchedCargoRun({
  touchedPaths,
  cargoArguments,
}: {
  readonly touchedPaths: readonly string[];
  readonly cargoArguments: readonly string[];
}): string {
  /**
   Shell text of the step: touch every source file, then compile.
   */
  const command = [
    'find',
    ...touchedPaths.map(shellWord),
    '-type',
    'f',
    '-exec',
    'touch',
    '{}',
    '+',
    '&&',
    'cargo',
    ...cargoArguments.map(shellWord),
  ].join(' ');
  return `RUN ${JSON.stringify([
    'sh',
    '-c',
    command,
  ])}`;
}

/**
 Containerfile lines that create the unprivileged builder's home and switch to that user, so Cargo's outputs belong to
 the user the tests later run as. They come first in a stage and hold nothing a source change touches, so every later
 rebuild finds them cached. Each line is a build step with a fixed cost, which is why the home is made in one step.

 @param environment - space-separated `NAME=value` assignments of the stage's single `ENV` step

 @returns the `RUN`, `ENV` and `USER` instructions

 @example
 ```ts
 const lines = builderPreludeLines({ environment: 'HOME=/home/tester' });
 ```
 */
export function builderPreludeLines({ environment, }: {
  readonly environment: string;
},): string[] {
  return [
    `RUN ${JSON.stringify([
      'sh',
      '-c',
      'mkdir --parents /home/tester/.cargo && chown --recursive 1000:1000 /home/tester',
    ])}`,
    `ENV ${environment}`,
    'USER 1000:1000',
  ];
}
