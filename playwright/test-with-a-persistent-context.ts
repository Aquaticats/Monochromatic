/**
 A Playwright `test` whose browser context keeps its website data on disk.

 WebKit serves the origin private file system only from a website data store that persists. An ephemeral store,
 which is what `browser.newContext()` creates for every browser, has no storage directory, so
 `navigator.storage.getDirectory()` rejects with `UnknownError` ("The operation failed for an unknown transient
 reason (e.g. out of memory)") without a single byte being written. Chromium and Firefox serve OPFS from memory, so
 the ephemeral default hides the difference: a test that asserts OPFS availability passes on two browsers for a
 reason the third cannot share (measurements and the WebKit call chain:
 `doc/troubleshooting/webkit-opfs-needs-a-persistent-data-store.md`).

 A file that exercises the origin private file system therefore imports this `test` instead of the one from
 `@playwright/test`. Every browser then runs against the kind of website data store a real session has, in a
 temporary directory of this test's own, removed with the context, so no run inherits another one's storage.

 @example
 ```ts
 import { expect, test, } from '../../../playwright/test-with-a-persistent-context.ts';
 ```
 */

import { mkdtemp, rm, } from 'node:fs/promises';
import { tmpdir, } from 'node:os';
import { join, } from 'node:path';

import {
  chromium,
  firefox,
  test as base,
  webkit,
} from '@playwright/test';

import type {
  BrowserContext,
  BrowserType,
} from '@playwright/test';

//region Browser types

/**
 Browser types a project can name, so this fixture launches the browser the project asks for.

 A project whose `browserName` is missing has no type to launch, and the fixture says so instead of guessing.
 */
const browserTypeByName: Readonly<Record<string, BrowserType | undefined>> = {
  chromium,
  firefox,
  webkit,
};

//endregion Browser types

//region Fixture

/**
 Playwright `test` whose `context` keeps its website data on disk, in a fresh temporary directory per test.

 The runner's own context options (base URL, viewport, user agent, locale) reach the launched context through the
 same hook `browser.newContext()` goes through, so `page.goto('/')` keeps resolving against the configured base URL.
 */
export const test = base.extend<{ context: BrowserContext; }>({
  context: async ({ browserName, }, use,) => {
    /**
     Browser type this project runs.
     */
    const browserType = browserTypeByName[browserName];
    if (browserType === undefined)
      throw new Error(`No browser type is named ${browserName}, so this test cannot launch a persistent context.`,);
    /**
     Directory holding every byte of this context's website data.
     */
    const userDataDirectory = await mkdtemp(
      join(
        tmpdir(),
        'monochromatic-browser-context-',
      ),
    );
    try {
      /**
       Context launched on disk, which is what gives WebKit an origin private file system to open.
       */
      const context = await browserType.launchPersistentContext(userDataDirectory,);
      try {
        await use(context,);
      }
      finally {
        await context.close();
      }
    }
    finally {
      await rm(
        userDataDirectory,
        {
          recursive: true,
          force: true,
        },
      );
    }
  },
},);

//endregion Fixture

export { expect, } from '@playwright/test';
