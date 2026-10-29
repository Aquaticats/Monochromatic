// `@types/sinon` 22.0.1 publishes an `index.d.mts` that only default-exports
// the runtime namespace, so named imports of `sinon` members fail under
// TypeScript (TS2614) while working at runtime and under oxlint's resolver.
// This declaration restores the named surface for every program that includes
// it via the `/// <reference path>` in `sinon.ts`. Members not listed here are
// invisible to named imports; add them as the test harness names more.
// See doc/troubleshooting/sinon-types-esm-named-imports.md.

import sinon from 'sinon';

declare module 'sinon' {
  export const createSandbox: typeof sinon.createSandbox;
  export const match: typeof sinon.match;
  export type SinonMatcher = sinon.SinonMatcher;
  export type SinonSandbox = sinon.SinonSandbox;
  export type SinonSandboxConfig = sinon.SinonSandboxConfig;
  export type SinonSpy = sinon.SinonSpy;
  export type SinonStub = sinon.SinonStub;
}
