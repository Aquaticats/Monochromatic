/**
 * Unit tests for the high-level git server ops.
 *
 * Each test runs against a freshly created temp directory via `mkdtemp`
 * so they do not pollute each other's state.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core';
import nodeFs from 'node:fs';
import { mkdtemp, } from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import * as git from 'isomorphic-git';

import { ZERO_OID, } from './iso-server-refs.ts';
import {
  buildInfoRefsAdvertisement,
  ensureRepoExists,
  getRef,
  handleReceivePack,
  handleUploadPack,
  listRefs,
} from './iso-server.ts';
import {
  encodePkt,
  flushPkt,
} from './pkt-line.ts';

/** All-zero OID used by the smart-HTTP protocol to mark a ref creation. */
const ZERO = ZERO_OID;

/**
 * Allocates a fresh temp dir and points `WEBAPP_FORGE_GITDIR_ROOT` at it
 * so the test exercises a clean state.
 *
 * @returns the temp dir path
 *
 * @example
 * ```ts
 * const root = await freshGitdirRoot();
 * ```
 */
async function freshGitdirRoot(): Promise<string> {
  const root = await mkdtemp(join(
    scratchRoot(),
    'forge-iso-server-',
  ),);
  process.env.WEBAPP_FORGE_GITDIR_ROOT = root;
  return root;
}

/**
 * Concatenates byte chunks.
 *
 * @param chunks - ordered chunks
 *
 * @returns flattened bytes
 *
 * @example
 * ```ts
 * concat([encodePkt('a'), flushPkt()]);
 * ```
 */
function concat(chunks: readonly Uint8Array[],): Uint8Array {
  let total = 0;
  for (const chunk of chunks)
    total += chunk.byteLength;
  const out = new Uint8Array(total,);
  let cursor = 0;
  for (const chunk of chunks) {
    out.set(
      chunk,
      cursor,
    );
    cursor += chunk.byteLength;
  }
  return out;
}

/**
 * Builds a self-contained commit in a temp gitdir and returns the
 * generated pack bytes plus the commit's OID. Used to simulate a
 * client's `git push` payload.
 *
 * @returns commit OID and packfile bytes
 *
 * @example
 * ```ts
 * const { oid, packfile } = await fabricateSingleCommit();
 * ```
 */
async function fabricateSingleCommit(): Promise<{
  oid: string;
  packfile: Uint8Array;
}> {
  const gitdir = await mkdtemp(join(
    scratchRoot(),
    'forge-fab-',
  ),);
  await git.init({
    fs: nodeFs,
    gitdir,
    bare: true,
    defaultBranch: 'main',
  },);
  // Write a blob, a tree pointing at it, and a commit pointing at the tree.
  const blobOid = await git.writeBlob({
    fs: nodeFs,
    gitdir,
    blob: new TextEncoder().encode('hello\n',),
  },);
  const treeOid = await git.writeTree({
    fs: nodeFs,
    gitdir,
    tree: [
      {
        mode: '100644',
        path: 'README',
        oid: blobOid,
        type: 'blob',
      },
    ],
  },);
  const commitOid = await git.writeCommit({
    fs: nodeFs,
    gitdir,
    commit: {
      tree: treeOid,
      parent: [],
      author: {
        name: 'Test',
        email: 'test@example.com',
        timestamp: 1_700_000_000,
        timezoneOffset: 0,
      },
      committer: {
        name: 'Test',
        email: 'test@example.com',
        timestamp: 1_700_000_000,
        timezoneOffset: 0,
      },
      message: 'initial\n',
    },
  },);
  const result = await git.packObjects({
    fs: nodeFs,
    gitdir,
    oids: [
      commitOid,
      treeOid,
      blobOid,
    ],
    write: false,
  },);
  if (result.packfile === undefined)
    throw new Error('packObjects returned no packfile',);
  return {
    oid: commitOid,
    packfile: new Uint8Array(result.packfile,),
  };
}

await describe({
  name: 'iso-server',
  concurrency: 1,
  children: [
    it({
      name: 'ensureRepoExists creates a bare gitdir',
      async fn() {
        await freshGitdirRoot();
        const gitdir = await ensureRepoExists({
          owner: 'alice',
          repo: 'demo',
        },);
        expect(gitdir.endsWith('alice/demo.git',),).toBe(true,);
        // HEAD must exist as a symbolic ref pointing at refs/heads/main.
        const headPath = join(
          gitdir,
          'HEAD',
        );
        const headBytes = nodeFs.readFileSync(headPath,);
        expect(headBytes.toString('utf8',).trim(),).toBe('ref: refs/heads/main',);
      },
    },),
    it({
      name:
        'buildInfoRefsAdvertisement returns the empty-repo placeholder for a fresh repo',
      async fn() {
        await freshGitdirRoot();
        const body = await buildInfoRefsAdvertisement({
          owner: 'alice',
          repo: 'demo',
          service: 'git-upload-pack',
        },);
        const text = new TextDecoder().decode(body,);
        expect(text.startsWith('001e# service=git-upload-pack\n',),).toBe(true,);
        // Empty repo line begins with all-zero OID + the placeholder.
        expect(text.includes(`${ZERO} capabilities^{}`,),).toBe(true,);
      },
    },),
    it({
      name: 'handleReceivePack indexes pack, updates ref, and reports ok',
      async fn() {
        await freshGitdirRoot();
        const fab = await fabricateSingleCommit();
        const triplet =
          `${ZERO} ${fab.oid} refs/heads/main\0report-status side-band-64k\n`;
        const body = concat([
          encodePkt(triplet,),
          flushPkt(),
          fab.packfile,
        ],);
        const outcome = await handleReceivePack({
          owner: 'alice',
          repo: 'demo',
          body,
        },);
        const responseText = new TextDecoder().decode(outcome.body,);
        expect(responseText.includes('unpack ok\n',),).toBe(true,);
        expect(responseText.includes('ok refs/heads/main\n',),).toBe(true,);
        expect(outcome.applied.length,).toBe(1,);
        const sha = await getRef({
          owner: 'alice',
          repo: 'demo',
          ref: 'refs/heads/main',
        },);
        expect(sha,).toBe(fab.oid,);
      },
    },),
    it({
      name: 'handleUploadPack returns a packfile for a wanted OID',
      async fn() {
        await freshGitdirRoot();
        const fab = await fabricateSingleCommit();
        // Push first so the server has the data on hand.
        const triplet =
          `${ZERO} ${fab.oid} refs/heads/main\0report-status side-band-64k\n`;
        await handleReceivePack({
          owner: 'alice',
          repo: 'demo',
          body: concat([
            encodePkt(triplet,),
            flushPkt(),
            fab.packfile,
          ],),
        },);
        // Now request the OID via upload-pack.
        const wantBody = concat([
          encodePkt(`want ${fab.oid} side-band-64k thin-pack\n`,),
          flushPkt(),
          encodePkt('done\n',),
        ],);
        const responseBody = await handleUploadPack({
          owner: 'alice',
          repo: 'demo',
          body: wantBody,
        },);
        const text = new TextDecoder().decode(responseBody,);
        expect(text.startsWith('0008NAK\n',),).toBe(true,);
        // PACK header must appear inside the sideband-1 chunks somewhere.
        expect(text.includes('PACK',),).toBe(true,);
      },
    },),
    it({
      name: 'listRefs surfaces a newly created branch',
      async fn() {
        await freshGitdirRoot();
        const fab = await fabricateSingleCommit();
        await handleReceivePack({
          owner: 'alice',
          repo: 'demo',
          body: concat([
            encodePkt(
              `${ZERO} ${fab.oid} refs/heads/feat-x\0report-status side-band-64k\n`,
            ),
            flushPkt(),
            fab.packfile,
          ],),
        },);
        const heads = await listRefs({
          owner: 'alice',
          repo: 'demo',
          filepath: 'refs/heads',
        },);
        expect([...heads,],).toEqual(['feat-x',],);
      },
    },),
  ],
},);
