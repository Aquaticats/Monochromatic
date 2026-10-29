/**
 OCR input generation and persisted-session discovery guidance.
 
 @module
 */

import type { Dirent, } from 'node:fs';
import {
  readdir,
  stat,
} from 'node:fs/promises';
import { homedir, } from 'node:os';
import { join, } from 'node:path';

import { caughtValueText, } from '@monochromatic-dev/module-caught-value/ts';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

/**
 Tagged discovery logger that never reads session content.
 */
const l = tagged({ tag: 'input-guidance', },);

/**
 Domain sentinel indicating no persisted OCR session was discovered.
 */
const NO_OCR_SESSION_JSONL: unique symbol = Symbol('no OCR session JSONL found',);

/**
 Candidate persisted session ordered by filesystem modification time.
 */
type SessionCandidate = {
  readonly path: string;
  readonly modifiedMilliseconds: number;
};

/**
 Reads directory entries without replacing primary input diagnostics.
 
 @param path - OCR session root or encoded repository directory.
 
 @returns Entries, or empty list when discovery path is unavailable.
 */
async function readableDirectory(path: string,): Promise<readonly Dirent[]> {
  try {
    return await readdir(
      path,
      { withFileTypes: true, },
    );
  }
  catch (error: unknown) {
    l.debug(`OCR session discovery skipped a directory: ${caughtValueText(error,)}`,);
    return [];
  }
}

/**
 Reads one candidate's modification evidence.
 
 @param path - Persisted JSONL candidate path.
 
 @returns Singleton candidate list, or empty list after filesystem failure.
 */
async function sessionCandidate(path: string,): Promise<readonly SessionCandidate[]> {
  try {
    /**
     Filesystem metadata used only for latest-session selection.
     */
    const metadata = await stat(path,);
    return [{
      path,
      modifiedMilliseconds: metadata.mtimeMs,
    },];
  }
  catch (error: unknown) {
    l.debug(`OCR session discovery skipped a file: ${caughtValueText(error,)}`,);
    return [];
  }
}

/**
 Identifies regular persisted OCR JSONL files.
 
 @param entry - Directory entry under encoded repository session directory.
 
 @returns Whether entry is a regular `.jsonl` file.
 */
function isJsonlSessionEntry(entry: Dirent,): boolean {
  /**
   Regular-file evidence excluding nested directories and special files.
   */
  const regularFile = entry.isFile();
  /**
   Persisted OCR transcript extension evidence.
   */
  const jsonlExtension = entry.name
    .endsWith('.jsonl',);
  return regularFile && jsonlExtension;
}

/**
 Reads timed candidates from one encoded repository directory.
 
 @param sessionsRoot - OCR persisted-session root.
 
 @param repositoryEntry - Encoded repository directory entry.
 
 @returns Timed readable JSONL candidates.
 */
async function repositorySessionCandidates({
  sessionsRoot,
  repositoryEntry,
}: {
  readonly sessionsRoot: string;
  readonly repositoryEntry: Dirent;
},): Promise<readonly SessionCandidate[]> {
  if (!repositoryEntry.isDirectory()) {
    return [];
  }
  /**
   Persisted session directory for one encoded repository.
   */
  const repositoryDirectory = join(
    sessionsRoot,
    repositoryEntry.name,
  );
  /**
   Entries inspected without reading transcript contents.
   */
  const sessionEntries = await readableDirectory(repositoryDirectory,);
  return (await Promise.all(sessionEntries
    .filter(isJsonlSessionEntry,)
    .map(function inspectSessionEntry(entry,): Promise<readonly SessionCandidate[]> {
      return sessionCandidate(join(
        repositoryDirectory,
        entry.name,
      ),);
    },),))
    .flat();
}

/**
 Finds latest persisted OCR JSONL without invoking OCR or reading session content.
 
 @param homeDirectory - Runtime home containing `.opencodereview`.
 
 @returns Latest path or domain-specific absent sentinel.
 */
async function findLatestOcrSessionJsonl(
  homeDirectory: string,
): Promise<string | typeof NO_OCR_SESSION_JSONL> {
  /**
   OCR v1.9.4 persisted-session root confirmed from current CLI help and source.
   */
  const sessionsRoot = join(
    homeDirectory,
    '.opencodereview',
    'sessions',
  );
  /**
   Encoded repository directories under OCR session root.
   */
  const repositoryEntries = await readableDirectory(sessionsRoot,);
  /**
   Timed candidates loaded concurrently from finite directory entries.
   */
  const candidates = (await Promise.all(repositoryEntries
    .map(function inspectRepositoryEntry(entry,): Promise<readonly SessionCandidate[]> {
      return repositorySessionCandidates({
        sessionsRoot,
        repositoryEntry: entry,
      },);
    },),))
    .flat();
  if (candidates.length === 0) {
    return NO_OCR_SESSION_JSONL;
  }
  return candidates
    .reduce(function selectLaterCandidate(
      current,
      candidate,
    ): SessionCandidate {
      return candidate.modifiedMilliseconds > current.modifiedMilliseconds
        ? candidate
        : current;
    },)
    .path;
}

/**
 Builds actionable OCR generation and discovery diagnostic.
 
 @param homeDirectory - Optional injected home for disposable verification.
 
 @returns Commands, common path, and latest concrete session when found.
 
 @example
 ```ts
 await buildInputGuidance({ homeDirectory: '/tmp/home' });
 ```
 */
export async function buildInputGuidance({
  homeDirectory = homedir(),
}: {
  readonly homeDirectory?: string;
} = {},): Promise<string> {
  /**
   Latest persisted transcript discovered without invoking OCR.
   */
  const latestSession = await findLatestOcrSessionJsonl(homeDirectory,);
  /**
   Stable generation and common-location guidance.
   */
  const baseLines = [
    'Get supported OCR input with one of these commands:',
    '  open-code-review-issue --interactive "$(ocr review --format json)"',
    '  ocr review --format json > review.json',
    '  ocr scan --format json > scan.json',
    '  ocr session list --json',
    '  ocr session comments --json <session-id> > comments.json',
    '',
    'Persisted OCR session JSONL files are commonly stored at:',
    '  ~/.opencodereview/sessions/<encoded-repo-path>/<session-id>.jsonl',
  ] as const;
  if ((typeof latestSession) === 'symbol') {
    return baseLines.join('\n',);
  }
  return [
    ...baseLines,
    '',
    'Latest OCR session JSONL found:',
    `  ${latestSession}`,
  ].join('\n',);
}
