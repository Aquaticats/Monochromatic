/**
 Stub value `index.cjs` exports: callable, constructable, and answering every property read with itself, mirroring the
 runtime `Proxy`.

 Recursive because one level of typing reports `stub.foo.bar()` as `unknown` and rejects the `new stub()` the README
 documents, while the runtime answers both with the stub itself. A consumer type-checking under
 `noUncheckedIndexedAccess` additionally reads each unknown key as possibly `undefined`, since that flag adds the union
 to every index-signature read.
 */
type SilentStub =
  & ((...args: readonly unknown[]) => SilentStub)
    & (new (...args: readonly unknown[]) => SilentStub)
    & { readonly [key: PropertyKey]: SilentStub };

declare const _: SilentStub;
export = _;
