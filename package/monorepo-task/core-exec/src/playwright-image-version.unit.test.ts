/**
 Tests that the Playwright container image carries the Playwright version the workspace resolves.

 `playwright.Dockerfile` says the tag must match `@playwright/test` in `pnpm-lock.yaml`, and nothing enforced it: the
 image sat at v1.63.0 while the workspace resolved 1.65.0, so every browser test failed with
 `browserType.launch: Executable doesn't exist at /ms-playwright/chromium_headless_shell-1254/...` once the container
 could reach the workspace at all (`doc/handover/container-limits-everywhere.md`, section Measured runs on this host).

 @module
 */

import { readFileSync, } from 'node:fs';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

/**
 Repository root, four folders above this package.
 */
const REPOSITORY_ROOT = join(
  import.meta.dirname,
  '..',
  '..',
  '..',
  '..',
);

/**
 Text of the file that builds the Playwright image.
 */
const DOCKERFILE_TEXT = readFileSync(
  join(
    REPOSITORY_ROOT,
    'playwright.Dockerfile',
  ),
  'utf8',
);

/**
 Text of the lockfile that resolves the workspace's Playwright.
 */
const LOCKFILE_TEXT = readFileSync(
  join(
    REPOSITORY_ROOT,
    'pnpm-lock.yaml',
  ),
  'utf8',
);

/**
 First line of the Dockerfile that names the Playwright base image.
 */
const BASE_IMAGE_PREFIX = 'FROM mcr.microsoft.com/playwright:v';

/**
 Lockfile key prefix of the Playwright package whose browsers the image must carry.
 */
const PLAYWRIGHT_LOCK_PREFIX = "'@playwright/test@";

/**
 The version the base image tag names.

 @returns the tag's version, such as `1.65.0`

 @throws Error when the Dockerfile has no Playwright base image

 @example
 ```ts
 const version = imageVersion();
 ```
 */
function imageVersion(): string {
  /**
   Base image line of the Dockerfile.
   */
  const line = DOCKERFILE_TEXT
    .split('\n',)
    .find((candidate,) => candidate.startsWith(BASE_IMAGE_PREFIX,),);
  if (line === undefined)
    throw new Error(`playwright.Dockerfile has no line starting with "${BASE_IMAGE_PREFIX}"`,);
  /**
   Tag text after `v`, such as `1.65.0-noble@sha256:...`.
   */
  const tag = line.slice(BASE_IMAGE_PREFIX.length,);
  return tag.split('-',)[0] ?? '';
}

/**
 The Playwright version the lockfile resolves for the workspace.

 @returns the resolved version, such as `1.65.0`

 @throws Error when the lockfile resolves no `@playwright/test`

 @example
 ```ts
 const version = lockedVersion();
 ```
 */
function lockedVersion(): string {
  /**
   Lockfile entry that names the resolved Playwright release.
   */
  const line = LOCKFILE_TEXT
    .split('\n',)
    .find((candidate,) => candidate.trimStart()
      .startsWith(PLAYWRIGHT_LOCK_PREFIX,),);
  if (line === undefined)
    throw new Error(`pnpm-lock.yaml has no "${PLAYWRIGHT_LOCK_PREFIX}" entry`,);
  /**
   Entry text without indentation, so the trailing quote is the last character.
   */
  const entry = line.trim();
  return entry.slice(
    PLAYWRIGHT_LOCK_PREFIX.length,
    -2,
  );
}

await describe({
  name: 'the Playwright image version',
  children: [
    it({
      name: 'is the @playwright/test version pnpm-lock.yaml resolves',
      fn: async () => {
        expect(imageVersion(),).toBe(lockedVersion(),);
      },
    },),
    it({
      name: 'pins the base image by digest, so a retagged release cannot change it',
      fn: async () => {
        /**
         Base image line of the Dockerfile.
         */
        const line = DOCKERFILE_TEXT
          .split('\n',)
          .find((candidate,) => candidate.startsWith(BASE_IMAGE_PREFIX,),);
        /**
         Digest the line pins.
         */
        const digest = (line ?? '').split('@',)[1] ?? '';
        expect(digest.startsWith('sha256:',),).toBe(true,);
        expect(digest.slice('sha256:'.length,),).toHaveLength(64,);
      },
    },),
  ],
},);
