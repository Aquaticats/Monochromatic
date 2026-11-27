/**
 What the whole-path fixture reads back after a run: the commits a repository holds and the text of the files the update's
 dependency group may change.

 Reading is kept apart from driving the update (`daily-update-whole-path-fixture.ts`) so each module stays within the
 package's file size, and so a test can see what a group left behind without knowing how the run started.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import { run, } from './daily-update-whole-path-programs-fixture.ts';

/**
 Every file the update's dependency group may change in the fixture repository, so a test can read what the group left
 behind and tell a landed group from a restored one.
 */
export const DEPENDENCY_FILES: readonly string[] = [
  'rust-catalog.toml',
  'package/a/broken-crate/Cargo.toml',
  'package/a/broken-crate/Cargo.lock',
  'package/c/red-crate/Cargo.toml',
  'package/c/red-crate/Cargo.lock',
  'pnpm-lock.yaml',
  'pnpm-workspace.yaml',
];

/**
 One commit of the disposable repository.
 */
export type RecordedCommit = {
  /**
   Subject line.
   */
  readonly subject: string;
  /**
   Repository-relative paths the commit landed, in the order `git log --name-only` lists them.
   */
  readonly files: readonly string[];
};

/**
 Reads a text file, answering the empty string when it does not exist.

 @param path - absolute path

 @returns the text, or the empty string when the file is absent

 @throws Error when the file exists but cannot be read

 @example
 ```ts
 const text = await readTextOrEmpty('/r/.cache/pending.json',);
 ```
 */
export async function readTextOrEmpty(path: string,): Promise<string> {
  try {
    return await readFile(
      path,
      'utf8',
    );
  }
  catch (error) {
    if ((Error.isError(error,)) && ('code' in error)
      && (error.code === 'ENOENT'))
      return '';
    throw error;
  }
}

/**
 Reads every commit of a repository, newest first, with the paths each one landed.

 The format marker keeps a subject that holds a newline or a path that looks like one from splitting a commit in two.

 @param root - absolute path of the repository

 @param environment - process environment for `git`

 @returns every commit with its subject and paths

 @throws Error when `git log` cannot run

 @example
 ```ts
 const commits = await readCommits({ root: '/r', environment, },);
 ```
 */
export async function readCommits({
  root,
  environment,
}: {
  readonly root: string;
  readonly environment: Readonly<Record<string, string>>;
},): Promise<readonly RecordedCommit[]> {
  /**
   The repository's commits, each subject followed by the paths it landed.
   */
  const log = await run({
    program: 'git',
    args: [
      'log',
      '--format=@@%s',
      '--name-only',
    ],
    directory: root,
    environment,
  },);
  return log.stdout
    .split('@@',)
    .filter(function nonEmpty(block: string,): boolean {
      return block.trim() !== '';
    },)
    .map(function parseCommit(block: string,): RecordedCommit {
      /**
       The block's lines, without the empty ones around the file list.
       */
      const lines = block.split('\n',)
        .filter(function nonEmptyLine(line: string,): boolean {
          return line !== '';
        },);
      return {
        subject: lines[0] ?? '',
        files: lines.slice(1,),
      };
    },);
}

/**
 Reads the text of every file the update's dependency group may change.

 @param root - absolute path of the repository

 @returns path to the text each file holds now; an absent file answers the empty string

 @example
 ```ts
 const texts = await readDependencyFileTexts('/r',);
 ```
 */
export async function readDependencyFileTexts(root: string,): Promise<Readonly<Record<string, string>>> {
  /**
   One entry per file, read side by side.
   */
  const entries = await Promise.all(DEPENDENCY_FILES.map(async function readOne(path,): Promise<readonly [
    string,
    string
  ]> {
    return [
      path,
      await readTextOrEmpty(join(
        root,
        path,
      ),),
    ] as const;
  },),);
  return Object.fromEntries(entries,);
}
