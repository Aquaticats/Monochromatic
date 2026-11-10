/**
 Globals the browser test harness (`playwright/test-harness.html`) installs on `window`.

 Both specifiers name the package's `/ts` source entry rather than its built
 output, so the declared shapes follow live source (rule ST3,
 `doc/decision/workspace-ts-source-imports.md`). They are type-only, so nothing
 loads them at runtime.
 */

import type * as ModuleLogger from '@monochromatic-dev/logger-core/ts';
import type * as ModuleLoggerBrowser from '@monochromatic-dev/logger-core/ts/browser.ts';

declare global {
  interface Window {
    moduleLogger: typeof ModuleLogger;
    moduleLoggerBrowser: typeof ModuleLoggerBrowser;
  }
}
