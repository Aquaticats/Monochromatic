# ECMAScript observation calls can hide user-code effects from readonly analysis

## Symptom

`prefer-readonly-parameter-type/prefer-readonly-parameter-types` reports `opaqueEffect` when a parameter crosses an
uncatalogued ECMAScript call.
Blanket treatment of serialization and reflection as observational would remove useful diagnostics but would be
unsound.

The audited outcomes are:

- `Array.isArray(value)` and `Object.is(left, right)` are observational;
- primitive String transforms and searches are observational for typed primitive inputs and outputs;
- global `String(value)` is observational only for primitive `value`;
  object conversion can invoke user-defined coercion hooks;
- Array identity searches,
  array and collection iterators,
  collection lookups,
  and collection `has` checks are observational;
- Array callback methods expose receiver-reachable values to callbacks,
  so callback effects propagate back to receiver origin;
- TypedArray identity searches and indexed reads are observational;
  TypedArray callback methods propagate receiver-reachable values,
  and `subarray` preserves receiver provenance through its shared view;
- `DataView.prototype.getUint16` and `getUint32` read viewed bytes without changing them;
  `setUint16` and `setUint32` write viewed bytes and therefore mutate the receiver view;
- `Array.prototype.with` allocates a copy and is observational;
- `Array.prototype.reduce` exposes accumulator,
  element,
  index,
  and receiver-reachable values through its callback relation;
- `Array.prototype.join` is observational only when every reachable element is primitive;
- `Error.isError(value)` is observational;
- Error construction with an options object can invoke `cause` proxy or getter behavior;
- `RegExp.prototype.test` can change receiver `lastIndex` for global or sticky expressions;
- `Date.prototype.toLocaleString()` and `Date.prototype.toLocaleDateString()` observe the receiver's
  `[[DateValue]]`;
  supplied locales and options remain opaque;
- `JSON.stringify(value)` remains opaque because it can invoke `toJSON`,
   accessors,
   proxy behavior,
   and a replacer;
- `Object.entries`,
  `Object.getPrototypeOf`,
  `Object.hasOwn`,
  `Object.keys`,
  and `Object.values` can dispatch caller-owned proxy or accessor hooks and therefore target argument 0;
- `Object.freeze` changes supplied property descriptors and targets argument 0;
- `Reflect.get` and `Reflect.ownKeys` can dispatch target proxy or accessor hooks;
  three-argument `Reflect.get` can also expose its explicit receiver to accessors;
- other integrity and property reflection calls remain opaque until audited individually.

## Root cause

ECMA-262 commit `1355a23e48aaf2b1d7b6cbfad0fb98bce999cfd1` defines `Array.isArray` as only an `IsArray` call in
`spec.html:40633`:

```html
<emu-clause id="sec-array.isarray" type="built-in function">
  <emu-alg>
    1. Return ? IsArray(_arg_).
  </emu-alg>
</emu-clause>
```

`IsArray` inspects internal identity and recursively unwraps proxies without invoking a proxy handler
(`spec.html:5798`):

```html
1. If _arg_ is not an Object, return *false*.
1. If _arg_ is an Array exotic object, return *true*.
1. If _arg_ is a Proxy exotic object, then
  1. Perform ? ValidateNonRevokedProxy(_arg_).
  1. Let _proxyTarget_ be _arg_.[[ProxyTarget]].
  1. Return ? IsArray(_proxyTarget_).
1. Return *false*.
```

Array and collection callback operations do not mutate their receiver directly,
but they call user code with receiver-reachable values.
The catalog therefore records callback argument positions separately from direct mutation targets.
A zero-target callback operation is not complete until call analysis propagates callback mutation and opacity.

`DataView.prototype.getUint16` and `getUint32` delegate to `GetViewValue`,
which validates the receiver and reads bytes from its viewed buffer.
Neither read operation writes receiver or buffer state.
`DataView.prototype.setUint16` and `setUint32` instead delegate to `SetViewValue`,
which writes numeric bytes into the viewed buffer.
Their catalogue entries therefore target the receiver.

The same distinction applies to `%TypedArray%` algorithms.
The pinned clauses for `at`,
`includes`,
and `indexOf` inspect integer-indexed elements without changing them.
`every` and `findLastIndex` call user code with element,
index,
and receiver values.
An optional `thisArg` remains opaque because the callback receives it as its `this` value.
`subarray` creates a new view over the same buffer,
so it is nonmutating but its result retains receiver provenance.
The source clause IDs are
`sec-%typedarray%.prototype.at`,
`sec-%typedarray%.prototype.every`,
`sec-%typedarray%.prototype.findlastindex`,
`sec-%typedarray%.prototype.includes`,
`sec-%typedarray%.prototype.indexof`,
and `sec-%typedarray%.prototype.subarray`.

Error construction delegates options handling to `InstallErrorCause` in
`sec-installerrorcause`.
That algorithm calls `HasProperty(options, "cause")` and then `Get(options, "cause")` when present.
Those operations do not directly change the options object,
but proxy traps or a `cause` getter can execute caller-owned behavior.
The project contract therefore records possible effects through `super` rather than classifying constructor options as
observational.

`Object.is` delegates only to `SameValue` (`spec.html:31394`):

```html
<emu-clause id="sec-object.is" type="built-in function">
  <emu-alg>
    1. Return SameValue(_value1_, _value2_).
  </emu-alg>
</emu-clause>
```

Global `String(value)` is also different from methods on primitive strings.
`String(value)` calls `ToString(value)` in `spec.html:36077`.
For an object,
`ToString` calls `ToPrimitive` with string preference (`spec.html:5650`).
`ToPrimitive` first gets and calls `value[Symbol.toPrimitive]` when present,
then `OrdinaryToPrimitive` gets and calls `toString` and `valueOf` (`spec.html:5049`).
Any of those caller-owned functions can change state.
Primitive arguments take direct conversion branches and cannot expose caller-owned mutable state.

`JSON.stringify` is different.
 Its `SerializeJSONProperty` operation reads `toJSON` and calls it when callable
(`spec.html:48387`):

```html
1. Let _value_ be ? Get(_holder_, _key_).
1. If _value_ is an Object or _value_ is a BigInt, then
  1. Let _toJSON_ be ? GetV(_value_, *"toJSON"*).
  1. If IsCallable(_toJSON_) is *true*, then
    1. Set _value_ to ? Call(_toJSON_, _value_, « _key_ »).
1. If _state_.[[ReplacerFunction]] is not *undefined*, then
  1. Set _value_ to ? Call(_state_.[[ReplacerFunction]], _holder_, « _key_, _value_ »).
```

The earlier idea that serialization and reflection calls could share one observational catalog class was wrong.
The algorithms differ at the exact user-code dispatch boundary,
 so each callable requires its own audit.

ECMA-402 commit `5273ed81c1a81cd87aaaaf87df48e7084d38259c`
defines `Date.prototype.toLocaleString` in
[`spec/locale-sensitive-functions.html`][ecma402-date-locale].
The source digest is
`744cb1e2a095414799a94d33f1b0caaef6561769d95bf5387716668cd93d52cd`.
The exact `toLocaleString` algorithm fragment digest is
`0b2ffb5786b37094c13c77a2d7ee13e439522fe457a95fe44b4b7ef5fa6ad659`.
The exact source-clause digest for `toLocaleDateString` is
`427041594b4e89f86680311d42472efc2aea9786d3f4b9cd2327197e3d1c1120`.

Both algorithms require the receiver's `[[DateValue]]` slot,
read that number,
create an `Intl.DateTimeFormat`,
and format the number.
They do not change receiver state.
`CreateDateTimeFormat` receives optional locales and options,
which can expose caller-owned iteration,
property access,
and coercion hooks.
The catalog therefore keeps argument positions opaque while accepting calls with no supplied arguments as observational.

## Verification

The source under test is TC39 ECMA-262 commit
`1355a23e48aaf2b1d7b6cbfad0fb98bce999cfd1`,
 cloned on 2026-07-13.
Node.
js `v26.5.0` produced the recorded runtime result:

```js
let traps = 0;
const target = [];
const proxy = new Proxy(target, {
  get(value, key, receiver) {
    traps += 1;
    return Reflect.get(value, key, receiver);
  },
  ownKeys(value) {
    traps += 1;
    return Reflect.ownKeys(value);
  },
  getOwnPropertyDescriptor(value, key) {
    traps += 1;
    return Reflect.getOwnPropertyDescriptor(value, key);
  },
});

console.log(JSON.stringify({
  isArray: Array.isArray(proxy),
  same: Object.is(proxy, proxy),
  traps,
}));
JSON.stringify(proxy);
console.log(JSON.stringify({ trapsAfterStringify: traps }));

const state = {
  value: 0,
  toJSON() {
    this.value += 1;
    return { value: this.value };
  },
};
console.log(JSON.stringify({ json: JSON.stringify(state), valueAfter: state.value }));
```

```text
{"isArray":true,"same":true,"traps":0}
{"trapsAfterStringify":2}
{"json":"{\"value\":1}","valueAfter":1}
```

### Calls verified as observational

- `Array.isArray(proxy)` did not invoke configured proxy traps.
- `Object.is(proxy, proxy)` did not invoke configured proxy traps.
- Exact standard-library String transforms and searches return primitives without mutating their string receiver.
- Exact Array `includes`,
  `indexOf`,
  and `lastIndexOf` calls compare identity without mutating receiver.
- Exact Array `at`,
  `entries`,
  `keys`,
  `slice`,
  and `values` calls observe receiver without invoking user callbacks.
- Exact Array `every`,
  `filter`,
  `find`,
  `findIndex`,
  `flatMap`,
  `forEach`,
  `map`,
  `reduce`,
  and `some` calls propagate callback effects from receiver-reachable element and collection arguments.
- Exact Array `with` returns a new array without changing its receiver.
- Exact TypedArray `at`,
  `includes`,
  and `indexOf` calls observe indexed values;
  `every` and `findLastIndex` propagate callback effects;
  `subarray` returns a shared view without directly mutating the receiver.
- Exact Map,
  Set,
  WeakMap,
  and WeakSet `has` calls test identity without mutating receiver.
- Exact Map and Set iterator calls observe collection without invoking user callbacks.
- Exact collection `forEach` calls propagate callback effects from receiver-reachable keys,
  values,
  and collection arguments.
- Exact Map `get` observes stored identity without mutating receiver.
- Exact Array `join` observes primitive elements,
  while object-element coercion remains opaque.
- Exact `Error.isError` inspects error identity without mutating its argument.
- Error construction options remain effectful because `cause` presence and value reads can invoke caller-owned hooks.

### Calls verified as effectful or opaque

- `String(object)` property reads can invoke getters and proxy traps;
  callable `Symbol.toPrimitive`,
  `toString`,
  and `valueOf` values are then invoked.
- `Object.entries`,
  `Object.getPrototypeOf`,
  `Object.hasOwn`,
  `Object.keys`,
  and `Object.values` retain argument-0 effects because proxy and accessor hooks are caller-owned code.
- `Object.freeze` retains argument-0 mutation because it changes supplied descriptors.
- `JSON.stringify(proxy)` invoked proxy behavior.
- `JSON.stringify(state)` invoked authored `toJSON` and mutated `state.value`.
- `JSON.stringify(value, replacer)` has an additional explicit callback path in `spec.html:48392`.

## Verified workarounds

The rule catalog records exact standard-library owner and member identities with empty mutation targets.
This covers `ArrayConstructor.isArray`,
`ObjectConstructor.is`,
`Array.prototype.with`,
primitive String methods with primitive-only typed inputs and outputs,
Array identity searches and iterators,
collection membership and iterator operations,
callback operations with explicit callback relations including `reduce`,
primitive-element Array `join`,
and `ErrorConstructor.isError`.
Exact declaration provenance prevents same-named project methods from inheriting this treatment.

For global `String(value)`,
the rule now identifies exact `globalThis.String` declaration provenance and inspects argument type.
It accepts primitive unions,
`symbol`,
and type-branded primitives,
then emits a dedicated object-coercion diagnostic for object-capable input.
Same-named imported callables retain ordinary external-call treatment.

Global `String` does not reassign its argument and is not itself a mutator.
The effect comes from reading conversion properties and invoking caller-owned code.
A TypeScript object type cannot prove that code absent at runtime because accessors,
method overrides,
and proxies remain assignable.
The diagnostic names those operations and lists every supported remediation rather than describing `String` as an
unknown external call.

First narrow primitive branches and format nonprimitive values without coercion.
`package/page-weight/cli/src/error-format.ts` returns `Error.message` for errors,
returns thrown strings directly,
and reports only runtime category for other values.
Its regression test proves ordinary `toString`,
`valueOf`,
`Symbol.toPrimitive`,
conversion-property accessors,
and proxy property traps are not invoked.

If object coercion is intentional,
use an explicit complete contract:

```typescript
/**
 * Converts caller value with deliberate coercion hooks.
 *
 * @param value - Caller value allowed to define conversion behavior.
 *
 * @returns caller-defined text conversion.
 *
 * @mutates value - String may invoke getters, proxy traps, Symbol.toPrimitive, toString, or valueOf on this input.
 */
function deliberatelyCoerce(value: unknown,): string {
  return String(value,);
}
```

The rule verifies that contract against exact global `String` provenance and propagates a known mutation effect.
Incomplete `@mutates` descriptions remain unresolved diagnostics.

For `JSON.stringify`,
first move serialization to the boundary where value ownership or construction is known,
then pass serialized primitive text through generic transports.
This avoids falsely declaring that a transport mutates a message which its caller already proved to be plain data.
`package/mcp/stdio/src/transport.ts` follows this shape:
call sites serialize owned response values,
while `writeSerializedMessage` receives only text and documents only output-writer mutation.

Use a verified local adapter with `@mutates` only when caller-owned hooks can actually run.
Its contract must name `JSON.stringify` and every parameter whose reachable hooks remain possible.

## What does not work

- Cataloging global `String` as observational for unknown or object inputs fails because coercion dispatches user code.
- Cataloging `JSON.stringify` as observational because no replacer argument is present fails because `toJSON`,
   getters,
  and proxy behavior remain reachable.
- Cataloging all reflection functions together fails because operations such as integrity checks can dispatch proxy
  internal methods.
- Treating callback methods as complete zero-effect observations misses mutation through callback parameters.
- Treating Array `join` as unconditionally observational misses object coercion hooks.
- Treating `Object.entries`,
  `Object.keys`,
  `Object.values`,
  `Object.getPrototypeOf`,
  or `Object.hasOwn` over unknown values as observational misses accessors and proxy traps.
  Their exact catalogue entries target argument 0.
- Treating `Object.freeze` as observation misses descriptor mutation on supplied object.
- Matching only member text such as `is` or `isArray` would bless unrelated project code.
   The catalog must retain owner,
  member,
   declaration provenance,
   and package-major identity.

## Upstream filing artifact

### Upstream filing decision

1. **Is it really upstream's fault?
   ** No. The observed behavior is required ECMAScript semantics.
2. **Can upstream fix it?
   ** No applicable defect exists;
    removing hook dispatch would change JavaScript behavior.
3. **Are they supporting this use case?
   ** The specification supports the runtime operations,
    not static effect analysis.
4. **Would the repo welcome our contribution?
   ** Not applicable because no specification defect was found.
5. **Will they likely fix it?
   ** Not applicable because the algorithms are behaving as specified.
6. **Have we prototyped a minimal fix compatible with their architecture?
   ** No upstream fix is appropriate.
    The
   consumer-side exact intrinsic catalog is the verified remedy.

Nothing should be filed upstream.

[ecma402-date-locale]: https://github.com/tc39/ecma402/blob/5273ed81c1a81cd87aaaaf87df48e7084d38259c/spec/locale-sensitive-functions.html
