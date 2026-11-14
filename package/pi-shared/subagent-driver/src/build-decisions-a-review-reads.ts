/**
 The manifests and tool configurations a review reads although no extension of theirs says they are code: the files that decide how
 the code beside them is built, checked or run.

 A change to one of them changes what a build, check or run of that folder produces or is allowed to do, while the code that runs
 stays as it was, so a review that answered nothing to review for it left that decision unread. The names are listed instead of the
 extensions they share (`.toml`, `.json`, `.jsonc`, `.yaml`, `.yml`), because an extension cannot tell a manifest from a fixture: on
 2026-11-13 this repository held 1345 files of those extensions under `package/`, 487 of them inside the fixture, evidence,
 performance, design and witness folders, among them the 92 TOML specification cases named `sample`, and a review of a change to
 one of those must still answer nothing to review.

 A name list costs the manifest of a tool nobody has added here yet, which goes unread until its name is listed; each name below is
 one this repository's packages or its root carry, or a standard manifest of a tool a package of this repository could run (a pinned
 toolchain, `rustfmt`, `cargo-deny`, a Gradle build), listed so that its first appearance is reviewed rather than missed.

 @module
 */

import { basename, } from 'node:path';

/**
 The names of the files a review reads as build decisions, in the groups this repository carries them in.
 */
const BUILD_FILE_NAME_LISTS: readonly (readonly string[])[] = [
  // The folder's own manifest and task file, which say what it builds, from which dependencies, and how its tasks run.
  [
    'mise.toml',
    'mise.local.toml',
    'package.json',
    'Cargo.toml',
    'build.gradle.kts',
    'settings.gradle.kts',
    'gradle.properties',
  ],
  // The versions of a dependency every package is built against.
  [
    'pnpm-workspace.yaml',
    'rust-catalog.toml',
    'bunfig.toml',
    'toolchain.plan.jsonc',
  ],
  // What the compiler, the linters, the tests and the mutation checks are told to do.
  [
    'clippy.toml',
    'rust-toolchain.toml',
    'rustfmt.toml',
    'deny.toml',
    'dprint.json',
    'monochromatic-lint.config.jsonc',
    'monochromatic-lint.prerequisites.jsonc',
    'file-enforcer.config.jsonc',
    'mutants.toml',
  ],
  // What the build produces beside the code, and what the program that runs it is allowed to do.
  [
    'about.toml',
    'wrangler.toml',
    'node.config.json',
    'sea-config.json',
  ],
];

/**
 The words the name of a build decision begins with, which a variant extends after a dot (`tsconfig.json` and
 `tsconfig.build.json`, `nextest.toml` and `nextest.windows-gnu.toml`, `Containerfile` and `Containerfile.release`), so a list of
 whole names would miss the next variant. The word alone names a file too (`Containerfile`), and a note that merely begins with
 the word (`ContainerfileREADME.md`) is not one of these.
 */
const BUILD_FILE_NAME_WORDS: readonly string[] = [
  'Containerfile',
  'Dockerfile',
  'nextest',
  'tsconfig',
];

/**
 The ends of the names a review reads as build decisions: a container file named for what it builds
 (`native-test-base.Containerfile`) ends with its tool rather than beginning with it.
 */
const BUILD_FILE_NAME_ENDINGS: readonly string[] = [
  '.Containerfile',
  '.Dockerfile',
];

/**
 The ends of the paths a review reads as build decisions, each anchored to the folder that holds it: `.cargo/config.toml` is a
 cargo configuration while a file inside a folder named `foo.cargo` is not, and the dprint package's `index.json` is the format
 every file of this repository is checked against, which its own generic name would never say.
 */
const BUILD_FILE_PATH_ENDINGS: readonly string[] = [
  '.cargo/config.toml',
  'config/dprint/index.json',
];

/**
 Says whether a path is a manifest or a tool configuration that decides how the code beside it is built, checked or run.

 @param path - a path from the top of the checkout

 @returns whether the path is one of the build decisions a reviewer reads

 @example
 ```ts
 const isABuildDecision = isABuildDecisionAReviewerReads('package/a/Cargo.toml',);
 ```
 */
export function isABuildDecisionAReviewerReads(path: string,): boolean {
  /**
   The file's own name, which the name lists are about.
   */
  const name = basename(path,);
  return BUILD_FILE_NAME_LISTS.some(function holdsTheName(names,) {
    return names.includes(name,);
  },)
    || BUILD_FILE_NAME_WORDS.some(function holdsTheWord(word,) {
      return (name === word) || name.startsWith(`${word}.`,);
    },)
    || BUILD_FILE_NAME_ENDINGS.some(function endsWithTheEnding(ending,) {
      return name.endsWith(ending,);
    },)
    || BUILD_FILE_PATH_ENDINGS.some(function endsWithThePathEnding(ending,) {
      // A git path is separated by `/` alone, and the ending is anchored to the folder that holds it, so `.cargo/config.toml`
      // matches `package/a/.cargo/config.toml` and not `package/a/foo.cargo/config.toml`; a review of the whole checkout reads
      // paths from its top, where the same file carries no folder before its name.
      return (path === ending) || path.endsWith(`/${ending}`,);
    },);
}
