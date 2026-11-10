/**
 Stub value `index.cjs` exports: callable, constructable, and answering every property read with itself, mirroring the
 runtime `Proxy`.

 Recursive because one level of typing reports `stub.foo.bar()` as `unknown` and rejects the `new stub()` the README
 documents, while the runtime answers both with the stub itself. A consumer type-checking under
 `noUncheckedIndexedAccess` additionally reads each unknown key as possibly `undefined`, since that flag adds the union
 to every index-signature read.

 The declared type stays permissive for the keys the runtime answers differently, because one index signature cannot
 carve them out without typing every read as possibly `undefined`: `then`, `catch`, and `finally` read as `undefined`,
 and `Symbol.toPrimitive` reads as a function producing the stub's primitive text.
 */
type SilentStub =
  & ((...args: readonly unknown[]) => SilentStub)
    & (new (...args: readonly unknown[]) => SilentStub)
    & { readonly [key: PropertyKey]: SilentStub };

declare const _: SilentStub;
export = _;
