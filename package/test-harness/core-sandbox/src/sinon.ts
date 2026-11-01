/// <reference path="./sinon-named-exports.d.ts" />
import {
  createSandbox,
  type SinonSandbox,
  type SinonSandboxConfig,
} from 'sinon';

/**
 Sinon sandbox extended with `Symbol.dispose` and `Symbol.asyncDispose`
 for use with `await using` / `using` syntax.
 */
export type DisposableSandbox = SinonSandbox & {
  [Symbol.asyncDispose]: () => Promise<void>;
  [Symbol.dispose]: () => void;
};

/**
 Creates a sinon sandbox that auto-restores on scope exit
 when used with `await using` or `using`.
 
 @param config - Optional sinon sandbox configuration
 
 @returns sandbox with dispose symbols attached
 
 @mutates config - `createSandbox` may inspect or retain caller-provided configuration capabilities.
 
 @example
 ```ts
 await using sandbox = createSinon();
 sandbox.stub(console, 'log').returns(undefined);
 // sandbox.restore() called automatically at scope exit
 ```
 */
export function createSinon(config?: SinonSandboxConfig,): DisposableSandbox {
  /* oxlint-disable no-unsafe-type-assertion -- sinon sandbox matches SinonSandbox but lacks dispose symbols */
  /**
   Local binding so dispose symbols can be installed before returning.
   */
  const sandbox = createSandbox(config,) as DisposableSandbox;
  /* oxlint-enable no-unsafe-type-assertion */

  sandbox[Symbol.dispose] = function dispose(): void {
    sandbox.restore();
  };

  sandbox[Symbol.asyncDispose] = function asyncDispose(): Promise<void> {
    sandbox.restore();
    return Promise.resolve();
  };

  return sandbox;
}
