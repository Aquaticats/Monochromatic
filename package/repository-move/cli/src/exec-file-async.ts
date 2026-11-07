/**
 `execFile` with a promise result, which never goes through a shell. One place holds the typed adapter so no file
 repeats it.

 @module
 */

import { execFile, } from 'node:child_process';
import { promisify, } from 'node:util';

/**
 Specialized promisified `execFile`, typed through Node's own adapter contract.
 The generic `promisify(execFile)` form trips `strict-void-return`;
 see `doc/troubleshooting/oxlint-promisify-void-return.md`.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;

/**
 Runs a program without a shell and waits for it to end.
 */
export const execFileAsync: typeof execFile.__promisify__ = promisifyExecFile(execFile,);
