/**
 Copies one function's identity onto another: own properties, prototype,
 and a `toString` that wraps the source body.
 
 Inlined from [`mimic-function`](https://github.com/sindresorhus/mimic-function)
 by Sindre Sorhus (MIT; notice preserved in `LICENSES/MIT.txt`) at version
 5.0.1 so the fork keeps zero runtime dependencies on the memoization path.
 Call shape follows repository lint (one destructured object parameter),
 and the wrapper's `name` descriptor is derived from the wrapper itself
 rather than read from `Function.prototype.toString.name` (observably
 identical, without referencing a built-in method unbound).
 
 @module
 */

import { PropertyDescriptorMissingError, } from './errors.ts';

//region Captured descriptors

/**
 Descriptor of the `toString` property on `Function.prototype`, captured at
 module load like upstream `mimic-function`, so later patches to `toString`
 cannot change the replacement's property flags.
 */
const TOSTRING_DESCRIPTOR = Object.getOwnPropertyDescriptor(
  Function.prototype,
  'toString',
);

//endregion Captured descriptors

//region Descriptor helpers

/**
 Tests whether copying `fromDescriptor` onto `to` is legal.
 
 `Object.defineProperty()` throws when the target property already exists,
 is not configurable, and either one of its descriptor fields changes or its
 non-writable value would be replaced; this predicate reports exactly those
 legal copies.
 
 @param toDescriptor - Own descriptor of the copy target, absent when the
 property is new.
 
 @param fromDescriptor - Own descriptor of the source property.
 
 @returns `true` when `Object.defineProperty` would accept the copy.
 
 @example
 ```ts
 canCopyProperty({
   toDescriptor: undefined,
   fromDescriptor: Object.getOwnPropertyDescriptor(fn, 'name',),
 },); // => true
 ```
 */
export function canCopyProperty({
  toDescriptor,
  fromDescriptor,
}: {
  // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- mirrors `Object.getOwnPropertyDescriptor`'s documented `PropertyDescriptor | undefined` return for a missing target property
  readonly toDescriptor: PropertyDescriptor | undefined;
  readonly fromDescriptor: PropertyDescriptor;
},): boolean {
  return (toDescriptor === undefined)
    || (toDescriptor.configurable === true)
    || ((toDescriptor.writable === fromDescriptor.writable)
      && ((toDescriptor.enumerable === fromDescriptor.enumerable)
        && ((toDescriptor.configurable === fromDescriptor.configurable)
          && ((toDescriptor.writable === true)
            || (toDescriptor.value === fromDescriptor.value)))));
}

/**
 Copies one own property from source to target when legal.
 
 `length` and `prototype` are skipped because the target keeps its own
 parameter count and its non-writable prototype; `arguments` and `caller`
 are skipped because some runtimes surface them through `Reflect.ownKeys`
 and they must never be copied.
 
 @param to - Function receiving the copied property.
 
 @param from - Function the property is copied from.
 
 @param property - Own property name or symbol to copy.
 
 @param ignoreNonConfigurable - Whether illegal copies are silently skipped
 instead of thrown.
 
 @example
 ```ts
 copyProperty({
   to: memoized,
   from: fn,
   property: 'name',
   ignoreNonConfigurable: true,
 },);
 ```
 */
export function copyProperty({
  to,
  from,
  property,
  ignoreNonConfigurable,
}: {
  readonly to: object;
  readonly from: object;
  readonly property: string | symbol;
  readonly ignoreNonConfigurable: boolean;
},): void {
  if ((property === 'length')
    || (property === 'prototype'))
    return;

  if ((property === 'arguments')
    || (property === 'caller'))
    return;

  /**
   Own descriptor of the copy target, absent when the property is new.
   */
  const toDescriptor = Object.getOwnPropertyDescriptor(
    to,
    property,
  );
  /**
   Own descriptor of the source property, always present because the
   property came from `Reflect.ownKeys(from)`.
   */
  const fromDescriptor = Object.getOwnPropertyDescriptor(
    from,
    property,
  );

  if (fromDescriptor === undefined)
    throw new PropertyDescriptorMissingError();

  /**
   Whether defining this property would throw on the target.
   */
  const copyIsIllegal = !canCopyProperty({
    toDescriptor,
    fromDescriptor,
  },);

  if (copyIsIllegal && ignoreNonConfigurable)
    return;

  Object.defineProperty(
    to,
    property,
    fromDescriptor,
  );
}

//endregion Descriptor helpers

//region Prototype and toString

/**
 Points the target's prototype chain at the source's prototype, so methods
 and `instanceof` checks see the source's lineage.
 
 @param to - Function whose prototype chain is aligned.
 
 @param from - Function whose prototype is inherited.
 
 @example
 ```ts
 changePrototype({
   to: memoized,
   from: fn,
 },);
 ```
 */
export function changePrototype({
  to,
  from,
}: {
  readonly to: object;
  readonly from: object;
},): void {
  /**
   Prototype the source function itself inherits from.
   */
  const fromPrototype = Reflect.getPrototypeOf(from,);

  // mutation-test-disable-next-line conditional -- re-setting an identical prototype is a no-op on every runtime; the guard is a pure fast path
  if (fromPrototype === Reflect.getPrototypeOf(to,))
    return;

  Reflect.setPrototypeOf(
    to,
    fromPrototype,
  );
}

/**
 Builds the wrapped `toString` body: a marker naming the wrapper followed
 by the source function's own `toString` output.
 
 @param withName - Wrapper name marker embedded in the wrapped body.
 
 @param fromBody - Pre-captured source body text.
 
 @returns Wrapped body text.
 
 @example
 ```ts
 wrappedToString({
   withName: 'with memoized() ',
   fromBody: 'function fetchUser() {}',
 },);
 ```
 */
export function wrappedToString({
  withName,
  fromBody,
}: {
  readonly withName: string;
  readonly fromBody: string;
},): string {
  return `/* Wrapped ${withName}*/\n${fromBody}`;
}

/**
 Installs a `toString` on the target that reports the wrapped source body.
 
 The source body is captured now (not lazily) so the source function can be
 garbage collected, and the bound wrapper keeps repeated `toString()` calls
 cheap. The wrapper's `name` becomes `toString` with the standard
 function-name flags, matching upstream's copy of
 `Function.prototype.toString.name`.
 
 @param to - Function receiving the wrapped `toString`.
 
 @param from - Function whose body text is wrapped.
 
 @param name - Target name embedded in the wrapper marker.
 
 @example
 ```ts
 changeToString({
   to: memoized,
   from: fn,
   name: 'memoized',
 },);
 ```
 */
export function changeToString({
  to,
  from,
  name,
}: {
  readonly to: object;
  readonly from: {
    readonly toString: () => string;
  };
  readonly name: string;
},): void {
  /**
   Name marker shown in the wrapped `toString` output.
   */
  const withName = (name === '')
    ? ''
    : `with ${name.trim()}() `;
  /**
   `toString` replacement bound to the marker and the captured source body.
   */
  const newToString = wrappedToString.bind(
    null,
    {
      withName,
      fromBody: from.toString(),
    },
  );

  /**
   Own name descriptor of the bound wrapper, carrying the standard
   function-name flags.
   */
  const wrapperNameDescriptor = Object.getOwnPropertyDescriptor(
    newToString,
    'name',
  );

  // mutation-test-disable-next-line conditional -- every function carries a name descriptor, so the guard is unreachable; it exists to fail loud if that invariant breaks
  if (wrapperNameDescriptor === undefined)
    throw new PropertyDescriptorMissingError();

  Object.defineProperty(
    newToString,
    'name',
    {
      ...wrapperNameDescriptor,
      value: 'toString',
    },
  );

  if (TOSTRING_DESCRIPTOR === undefined)
    throw new PropertyDescriptorMissingError();

  /**
   Replacement descriptor copying `Function.prototype.toString`'s flags;
   absent flags become `false`, which is what `Object.defineProperty` does
   with upstream's explicit `undefined` fields.
   */
  const replacementDescriptor: PropertyDescriptor = {
    value: newToString,
    // mutation-test-disable-next-line boolean -- `Function.prototype.toString` always carries full data-descriptor flags, so this fallback is unreachable on conforming runtimes
    writable: TOSTRING_DESCRIPTOR.writable ?? false,
    // mutation-test-disable-next-line boolean -- same unreachable fallback as `writable` above
    enumerable: TOSTRING_DESCRIPTOR.enumerable ?? false,
    // mutation-test-disable-next-line boolean -- same unreachable fallback as `writable` above
    configurable: TOSTRING_DESCRIPTOR.configurable ?? false,
  };

  Object.defineProperty(
    to,
    'toString',
    replacementDescriptor,
  );
}

//endregion Prototype and toString

//region Mimic

/**
 Copies the source function's identity onto the target function.
 
 Target keeps its own body and parameter count (`length` is never copied),
 while name, own properties, prototype, and wrapped `toString` follow the
 source, so the wrapper is indistinguishable from the wrapped function to
 reflection-based callers.
 
 @param to - Target function to extend.
 
 @param from - Source function to copy from.
 
 @param ignoreNonConfigurable - Whether illegal property copies are skipped
 instead of thrown.
 
 @returns The extended target function.
 
 @example
 ```ts
 mimicFunction({
   to: memoized,
   from: fn,
   ignoreNonConfigurable: true,
 },);
 ```
 */
export function mimicFunction<Target extends { readonly name: string; }>({
  to,
  from,
  ignoreNonConfigurable = false,
}: {
  readonly to: Target;
  readonly from: {
    readonly toString: () => string;
  };
  readonly ignoreNonConfigurable?: boolean;
},): Target {
  /**
   Target name captured before copying overwrites it; reused by the wrapped
   `toString` marker.
   */
  const { name, } = to;

  for (const property of Reflect.ownKeys(from,))
    copyProperty({
      to,
      from,
      property,
      ignoreNonConfigurable,
    },);

  changePrototype({
    to,
    from,
  },);
  changeToString({
    to,
    from,
    name,
  },);

  return to;
}

//endregion Mimic
