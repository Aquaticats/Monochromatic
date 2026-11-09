/**
 A Playwright `test` whose browser context keeps its website data on disk.

 WebKit serves the origin private file system only from a website data store that persists. An ephemeral store,
 which is what `browser.newContext()` creates for every browser, has no storage directory, so
 `navigator.storage.getDirectory()` rejects with `UnknownError` ("The operation failed for an unknown transient
 reason (e.g. out of memory)") without a single byte being written. Chromium and Firefox serve OPFS from memory, so
 the ephemeral default hides the difference: a test that asserts OPFS availability passes on two browsers for a
 reason the third cannot share (measurements and the WebKit call chain:
 `doc/troubleshooting/webkit-opfs-needs-a-persistent-data-store.md`).

 A test that exercises the origin private file system therefore imports this `test` instead of the one from
 `@playwright/test`. Every browser then runs against the kind of website data store a real session has, in a
 directory of this test's own under the scratch root, removed with the context, so no run inherits another one's
 storage.

 @example
 ```ts
 import { expect, test, } from './opfs-browser-fixture.ts';
 ```
 */

import {
  mkdtemp,
  rm,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  chromium,
  firefox,
  test as base,
  webkit,
  type BrowserContext,
  type BrowserType,
  type PlaywrightTestArgs,
  type PlaywrightTestOptions,
  type PlaywrightWorkerArgs,
  type PlaywrightWorkerOptions,
  type TestType,
} from '@playwright/test';

//region Browser types

/**
 Thrown when a project runs a browser this fixture has no type for.
 */
class UnknownBrowserNameError extends Error {
  /**
   Puts the browser name in the message a failed run shows.

   @param typeName - Playwright's name for the browser a project runs
   */
  constructor(typeName: string,) {
    super(`No browser type is named ${typeName}, so this test cannot launch a persistent context.`,);
    this.name = 'UnknownBrowserNameError';
  }
}

/**
 Browser type of the project under test.

 @param typeName - Playwright's name for the browser a project runs

 @returns the browser type of that name

 @throws when no browser type carries the name
 */
function browserTypeNamed(typeName: string,): BrowserType {
  if (typeName === 'chromium')
    return chromium;
  if (typeName === 'firefox')
    return firefox;
  if (typeName === 'webkit')
    return webkit;
  throw new UnknownBrowserNameError(typeName,);
}

//endregion Browser types

//region Fixture

/**
 Playwright `test` whose `context` keeps its website data on disk, in a scratch directory of its own per test.

 The runner's own context options (base URL, viewport, user agent, locale) reach the launched context through the
 same hook `browser.newContext()` goes through, so `page.goto('/')` keeps resolving against the configured base URL.
 */
export const test: TestType<PlaywrightTestArgs & PlaywrightTestOptions & { context: BrowserContext; }, PlaywrightWorkerArgs & PlaywrightWorkerOptions> = base.extend<{ context: BrowserContext; }>({
  context: async function persistentContext(
    { browserName, },
    use,
  ): Promise<void> {
    /**
     Directory holding every byte of this context's website data.
     */
    const userDataDirectory = await mkdtemp(join(
      scratchRoot(),
      'monochromatic-browser-context-',
    ),);
    /**
     Removes that directory when this test's scope ends, declared before the context so the context closes first,
     since disposables run in reverse declaration order.
     */
    await using _directory = {
      async [Symbol.asyncDispose](): Promise<void> {
        await rm(
          userDataDirectory,
          {
            recursive: true,
            force: true,
          },
        );
      },
    };
    /**
     Context launched on disk, which is what gives WebKit an origin private file system to open.
     */
    const context = await browserTypeNamed(browserName,)
      .launchPersistentContext(userDataDirectory,);
    /**
     Closes the context when this test's scope ends, before the directory holding its data goes away.
     */
    await using _context = {
      async [Symbol.asyncDispose](): Promise<void> {
        await context.close();
      },
    };
    await use(context,);
  },
},);

//endregion Fixture

export { expect, } from '@playwright/test';
