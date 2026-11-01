# Module ES - Export Fixes Todo

> Stale as of 2026-05-13: this file references obsolete flat source files
> such as `src/iterable.is.ts` and `src/boolean.equal.ts`. The current package
> uses the nested `src/types/**` taxonomy and the root entry exports the `types`
> namespace plus package subpaths from `package.json`. Re-audit current
> `mise run //packages/module/es:lint:types` output before acting on any item below.

## Critical Export Issues

### Missing Exports in `iterable.is.ts`

**Status**: Critical: Blocking TypeScript compilation

The following exports are referenced by other modules but missing from [`iterable.is.ts`](src/iterable.is.ts:1):

- [ ] **`isEmptyArray`**: Referenced in:
  - [`iterable.reduce.ts`](src/iterable.reduce.ts:1)
  - [`error.throw.ts`](src/error.throw.ts:7)

- [ ] **`isAsyncGenerator`**: Referenced in:
  - [`boolean.equal.ts`](src/boolean.equal.ts:3)

- [ ] **`isGenerator`**: Referenced in:
  - [`boolean.equal.ts`](src/boolean.equal.ts:5)

- [ ] **`isMap`**: Referenced in:
  - [`boolean.equal.ts`](src/boolean.equal.ts:6)

- [ ] **`isArray`**: Referenced in:
  - [`promises.some.ts`](src/promises.some.ts:3)

- [ ] **`isArrayEmpty`**: Referenced in:
  - [`iterables.intersection.ts`](src/iterables.intersection.ts:4)

- [ ] **`isArrayOfLength1`**: Referenced in:
  - [`iterables.intersection.ts`](src/iterables.intersection.ts:5)

- [ ] **`arrayIsNonEmpty`**: Referenced in:
  - [`cli.command.ts`](src/cli.command.ts:27)

### Missing Exports in `numeric.type.int.ts`

**Status**: Critical: Type naming inconsistency

- [ ] **`isPositiveInt` vs `PositiveInt`**: Referenced in:
  - [`numeric.range.ts`](src/numeric.range.ts:7) expects `isPositiveInt` but export may be `PositiveInt`
  - Verify correct export name and fix references

### Missing Functions

**Status**: Critical: Undefined functions

- [ ] **`getRandomId`**: Referenced in:
  - [`any.ReplicatingStore.ts`](src/any.ReplicatingStore.ts:250)
  - [`any.ReplicatingStore.ts`](src/any.ReplicatingStore.ts:274)
  - Need to implement or import this function

### Module Export Consistency Issues

**Status**: High Priority: Module resolution

- [ ] **`isInt` vs `Int`** in module exports:
  - [`numeric.is.unit.test.ts`](src/numeric.is.unit.test.ts:6) expects `isInt` but may be exported as `Int`
  - Audit [`index.ts`](src/index.ts:1) exports for consistency

- [ ] **Type constraint issues**:
  - [`array.type.fixedLength.unit.test.ts`](src/array.type.fixedLength.unit.test.ts:35)
    has `TupleArray` constraint issue
  - [`mise.index-claude-user-messages.ts`](src/mise.index-claude-user-messages.ts:162) has `PatternMatcher` type issue

## Implementation Plan

### Phase 1: Add Missing Exports (1-2 hours)

1. **Audit `iterable.is.ts`** and add all missing exports
2. **Check `numeric.type.int.ts`** for correct export names
3. **Update `index.ts`** to export any missing functions
4. **Verify all import statements** resolve correctly

### Phase 2: Implement Missing Functions (2-4 hours)

1. **Implement `getRandomId`** function or identify correct import source
2. **Fix type constraint issues** in test files
3. **Resolve pattern matcher type compatibility**

### Phase 3: Validation (30 minutes)

1. **Run TypeScript compilation** to verify all errors resolved
2. **Run tests** to ensure no regression
3. **Update exports in `index.ts`** if needed

## Success Criteria

- [ ] All TypeScript compilation errors resolved
- [ ] All import statements resolve correctly
- [ ] All referenced functions and types are properly exported
- [ ] No regression in existing functionality
- [ ] Tests pass without modification
- [ ] Build system works without errors

## Cross-References

- [**Build System Todo**](../../TODO.build-system.md#missing-export-issues): Related TypeScript compilation fixes
- [**Code Quality Todo**](../../TODO.code-quality.md#typescript-compilation-errors): TypeScript error resolution
