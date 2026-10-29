import type * as ModuleLogger from '@monochromatic-dev/logger-core';
import type * as ModuleLoggerBrowser from '@monochromatic-dev/logger-core/browser';

declare global {
  interface Window {
    moduleLogger: typeof ModuleLogger;
    moduleLoggerBrowser: typeof ModuleLoggerBrowser;
  }
}
