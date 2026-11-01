/**
 * Integration test for the git smart-HTTP route handlers via h3's
 * in-process `app.fetch()`. Drives the URL routing, header parsing,
 * and response framing without needing to bind a port.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core';
import { mkdtemp, } from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import { H3, } from 'h3';

import { ZERO_OID, } from '../../git/iso-server-refs.ts';
import {
  encodePkt,
  flushPkt,
} from '../../git/pkt-line.ts';

import {
  gitInfoRefsHandler,
  gitReceivePackHandler,
  gitUploadPackHandler,
} from './git.ts';

/** HTTP 200 OK status code. */
const STATUS_OK = 200;

/**
 * Builds a fresh h3 app with the git routes mounted and a temp gitdir
 * root pointed at an isolated directory.
 *
 * @returns the configured app
 *
 * @example
 * ```ts
 * const app = await buildApp();
 * ```
 */
async function buildApp(): Promise<H3> {
  const root = await mkdtemp(join(
    scratchRoot(),
    'forge-git-routes-',
  ),);
  process.env.WEBAPP_FORGE_GITDIR_ROOT = root;
  const app = new H3();
  app.get(
    '/:owner/:repo/info/refs',
    gitInfoRefsHandler,
  );
  app.post(
    '/:owner/:repo/git-upload-pack',
    gitUploadPackHandler,
  );
  app.post(
    '/:owner/:repo/git-receive-pack',
    gitReceivePackHandler,
  );
  return app;
}

await describe({
  name: 'routes/git',
  concurrency: 1,
  children: [
    it({
      name:
        'info/refs?service=git-upload-pack returns the advertisement for an empty repo',
      async fn() {
        const app = await buildApp();
        const response = await app.fetch(new Request(
          'http://localhost/alice/demo.git/info/refs?service=git-upload-pack',
        ),);
        expect(response.status,).toBe(STATUS_OK,);
        const text = await response.text();
        expect(text.startsWith('001e# service=git-upload-pack\n',),).toBe(true,);
        expect(text.includes(`${ZERO_OID} capabilities^{}`,),).toBe(true,);
      },
    },),
    it({
      name: 'info/refs?service=git-receive-pack returns the receive-pack advertisement',
      async fn() {
        const app = await buildApp();
        const response = await app.fetch(new Request(
          'http://localhost/alice/demo.git/info/refs?service=git-receive-pack',
        ),);
        expect(response.status,).toBe(STATUS_OK,);
        const text = await response.text();
        expect(text.startsWith('001f# service=git-receive-pack\n',),).toBe(true,);
        expect(text.includes('report-status',),).toBe(true,);
      },
    },),
    it({
      name: 'git-upload-pack returns NAK when the want set is empty',
      async fn() {
        const app = await buildApp();
        // Empty body: no wants/haves; server should still respond with NAK.
        const body = new Uint8Array(0,);
        const response = await app.fetch(new Request(
          'http://localhost/alice/demo.git/git-upload-pack',
          {
            method: 'POST',
            body,
          },
        ),);
        expect(response.status,).toBe(STATUS_OK,);
        const text = await response.text();
        expect(text.startsWith('0008NAK\n',),).toBe(true,);
      },
    },),
    it({
      name: 'git-receive-pack handles a triplet-only request without a packfile',
      async fn() {
        const app = await buildApp();
        // Just a flush-pkt: no triplets, no pack. Should report unpack ok with no refs.
        const body = flushPkt();
        const response = await app.fetch(new Request(
          'http://localhost/alice/demo.git/git-receive-pack',
          {
            method: 'POST',
            body: new Uint8Array(body,),
          },
        ),);
        expect(response.status,).toBe(STATUS_OK,);
        const text = await response.text();
        expect(text.includes('unpack ok\n',),).toBe(true,);
      },
    },),
    it({
      name: 'rejects URLs whose repo path does not end in .git',
      async fn() {
        const app = await buildApp();
        const response = await app.fetch(new Request(
          'http://localhost/alice/demo/info/refs?service=git-upload-pack',
        ),);
        expect(response.status,).not.toBe(STATUS_OK,);
      },
    },),
    it({
      name: 'rejects unknown service values',
      async fn() {
        const app = await buildApp();
        const response = await app.fetch(new Request(
          'http://localhost/alice/demo.git/info/refs?service=git-bogus',
        ),);
        expect(response.status,).not.toBe(STATUS_OK,);
      },
    },),
    it({
      name: 'sample triplet body wraps with encodePkt and is parsed by the route',
      async fn() {
        const app = await buildApp();
        const triplet = `${ZERO_OID} ${ZERO_OID} refs/heads/probe\0report-status\n`;
        const body = new Uint8Array(
          encodePkt(triplet,).byteLength + flushPkt().byteLength,
        );
        const t = encodePkt(triplet,);
        body.set(
          t,
          0,
        );
        body.set(
          flushPkt(),
          t.byteLength,
        );
        const response = await app.fetch(new Request(
          'http://localhost/alice/demo.git/git-receive-pack',
          {
            method: 'POST',
            body,
          },
        ),);
        expect(response.status,).toBe(STATUS_OK,);
        const text = await response.text();
        expect(text.includes('unpack ok\n',),).toBe(true,);
        // The triplet has zero/zero: that's a no-op delete-when-not-present.
        // Either ok or ng response is acceptable; the wire shape matters.
        expect(text.includes('refs/heads/probe',),).toBe(true,);
      },
    },),
  ],
},);
