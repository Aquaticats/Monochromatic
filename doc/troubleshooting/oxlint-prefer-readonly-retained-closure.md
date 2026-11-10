# A parameter is withheld because a closure carrying it was handed to something

You expected `prefer-readonly-parameter-type` to offer `readonly` for a parameter,
and instead it reported that the parameter is used by a call it cannot inspect,
or said nothing at all about it.
Somewhere in the function,
a closure that reads that parameter was handed to a callee whose body this rule cannot read.

This page covers what decides that,
what the rule is protecting against,
and every way to get the offer back.

## What the rule is protecting against

Handing a closure to an uninspectable callee is not by itself a problem.
The callee may keep the closure,
invoke it now,
invoke it later,
store what it produced,
or write through what it produced.
Every one of those reaches your parameter through a value the closure handed back,
so a closure that hands back nothing writable exposes nothing whatever the callee does with it.

That is why this keeps its offer:

```ts
// The closure hands back a string, so retaining it exposes nothing.
export function readLabels(config: Config, rows: readonly Row[],): readonly string[] {
  return rows.map((row,): string => config.row.label,);
}
```

and why this cannot:

```ts
// The closure hands back the caller's own row, and the registry outlives the call.
export function keepProducer(config: Config, registry: Registry,): void {
  registry.keep((): Row => config.row,);
}
```

An annotation of `ReadonlyDeep<Config>` on the second would not stop the escape.
TypeScript ignores `readonly` property modifiers in assignability,
so a callee declaring `Row` accepts a `ReadonlyDeep<Row>` and hands back something writable.
The offer would be false rather than merely optimistic.

## The two cases that surprise people

Both come from the same question,
which is what the closure's completion hands back,
and both are asked of the callable that actually runs rather than of a declared type.

### A `void` result is trusted only when it describes a body

TypeScript permits assigning a value-returning function where a `void`-returning one is expected,
and permits no other such substitution.
So `() => void` on a **type** says nothing about what the value returns:

```ts
function produceRow(): Row {
  return ownedRow;
}

// Legal. No cast, no error.
const erased: () => void = produceRow;
```

A call through a parameter,
a mutable local,
or a member of an interface is therefore treated as able to hand back state,
because any of those can hold a function whose real return type is wider than the annotation.
A call to a function declaration keeps its `void` at face value,
because nothing can substitute a different body for a declaration.

The consequence you are most likely to hit:

```ts
// Withheld. `console.log` resolves to a member of the `Console` interface,
// not to a function declaration, so its `void` describes a slot.
registry.keep((): void => console.log(config.row.label,),);

// Offered. `reportLabel` is a declaration whose own body hands nothing back.
function reportLabel(label: string,): void {
  logSink.push(label,);
}
registry.keep((): void => reportLabel(config.row.label,),);
```

If a logging closure is what withheld your parameter,
the second form is the fix,
and it is a real improvement rather than a workaround:
it gives the rule a body to read instead of a signature to trust.

### A parameter default does not stand for every value the parameter can hold

Where the rule can name the callables an expression might be,
it treats them as evidence and joins their answer with the declared result type.
It does not narrow to them.

```ts
// Withheld. The default hands back a leaf, but a caller can pass a producer that does not,
// and the declared result admits `Row`.
export function forwardProducer(
  registry: Registry,
  producer: () => Row | string = (): string => 'leaf',
): void {
  registry.keep((): Row | string => producer(),);
}
```

Narrowing the declared type is what changes this,
not changing the default.

## Getting the offer back

In rough order of how often each is the right answer.

Narrow what the closure hands back.
If the closure only needs a label,
complete with the label rather than with the row:

```ts
registry.keep((): string => config.row.label,);
```

This is the strongest fix because it changes what is true,
not what the rule can see.
A closure that hands back a primitive cannot leak your object however it is retained.

Narrow the declared result type of the callable being invoked.
`() => string` instead of `() => Row | string` removes the possibility the rule is withholding for.

Give the rule a body instead of a signature.
Replace a call through a parameter,
a property,
or an interface member with a call to a function declaration in the configured project.
The rule reads the body and judges by what it actually returns.

Bring the callee into the analysis,
but only if it is a plain function or a `static` method.
Including a repository-owned callee in the nearest `tsconfig.json` lets the rule resolve it,
after which the closure is judged against what that callee really does with it.

This does **not** work for an ordinary instance method,
however visible its body is.
An instance method can be overridden by a subclass that need not exist in this project,
and an override can retain what the base only reads,
so the rule treats every overridable method as unresolved on purpose.
Adding the file to a `tsconfig.json` changes nothing for `registry.keep(...)` where `keep` is an instance
method.
Making the method `static`,
or extracting the retaining logic into a plain function,
is what makes it resolvable.

Hand over a snapshot rather than a capability.
Compute the value before the closure exists and let the closure close over the copy:

```ts
const label = config.row.label;
registry.keep((): string => label,);
```

Accept the withholding.
A withheld offer costs nothing at runtime and nothing in type safety.
It means this rule could not prove a `readonly` annotation would be sound,
and leaving the parameter mutable is always sound.

## What will not help

Annotating the parameter `readonly` or `ReadonlyDeep<...>` by hand.
That is the annotation the rule declined to offer,
and applying it does not stop the escape,
for the assignability reason given in "What the rule is protecting against".

Adding `@mutates` to the retaining callee.
A contract documents effects that are known;
it cannot make an unreadable implementation safe,
and this rule says so in the diagnostic itself.

Changing the closure's own return **annotation** while its body still completes with the parameter's state.
The rule asks the body,
not the annotation.
That asymmetry is deliberate,
and it is the same one that makes the `void` case above unsound to trust.

## Related

- `package/oxlint-plugin/prefer-readonly-parameter-type/README.md`,
   the rule's contract
- `doc/troubleshooting/oxlint-prefer-readonly-foreign-provenance.md`,
   for ownership-marker questions
- `doc/decision/prefer-readonly-result-provenance.md`,
   for why returning caller state is permitted
  while storing it is not
