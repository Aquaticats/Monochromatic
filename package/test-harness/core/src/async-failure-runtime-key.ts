/**
 Realm-wide identity of the async-failure runtime registry. @module
 */

/**
 Protocol key shared by the Node observer, source imports, and separately bundled artifact copies.

 Kept in a runtime-neutral module so the browser fixture probes the symbol the observer installs rather than a
 repeated literal that a version bump would leave behind.

 @internal
 */
export const RUNTIME_KEY: unique symbol = Symbol.for('@monochromatic-dev/test-harness-core/async-failure-runtime/v1',);
