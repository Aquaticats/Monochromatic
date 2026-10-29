# Module ES - API Refactors Todo

> Stale as of 2026-05-13: many links in this file reference obsolete flat
> source names such as `src/any.toExport.ts` and `src/array.range.ts`. Current
> implementations live under `src/types/**`. Rebuild this inventory from the
> current taxonomy before scheduling API refactor work.

## Major API Design Changes Required

### Critical Breaking Changes for Library Design

These refactors will affect ALL functions in the library and represent major breaking changes
that must be implemented systematically across the entire codebase.

---

## 1. Logger Parameter Requirement

### Overview

**Status**: Critical Priority; affects ALL functions

**Requirement**: All functions must take a `l: Logger` parameter for comprehensive logging support.

### Implementation Strategy

#### Function Signature Patterns

**Single Parameter Functions**:

```typescript
// Before
export function identity<const T,>(x: T,): T;

// After
export function identity<const T,>(x: T, l: Logger,): T;
```

**Two Parameter Functions**:

```typescript
// Before
export function addTwoNumbers(previousValue: number,
  currentValue: number,): number;

// After
export function addTwoNumbers(previousValue: number, currentValue: number,
  l: Logger,): number;
```

**Functions with 3+ Parameters** (see named parameters section):

```typescript
// Before
export function clamp(min: number, max: number, value: number,): number;

// After
export function clamp(
  { min, max, value, l, }: { min: number; max: number; value: number;
    l: Logger; },
): number;
```

### Functions Requiring Logger Parameter Addition

#### Any Utilities (8 functions)

- [ ] [`any.constant()`](src/any.constant.ts:32): Add logger parameter
- [ ] [`any.echo()`](src/any.echo.ts:26): Add logger parameter
- [ ] [`any.identity()`](src/any.identity.ts:25): Add logger parameter
- [ ] [`any.toExport()`](src/any.toExport.ts:45): Add logger parameter
- [ ] [`any.typeOf()`](src/any.typeOf.ts:25): Add logger parameter
- [ ] [`any.when()`](src/any.when.ts:24): Add logger parameter
- [ ] [`any.whenAsync()`](src/any.when.ts:65): Add logger parameter

#### Array Utilities (15+ functions)

- [ ] [`array.range()`](src/array.range.ts:73): Add logger parameter
- [ ] [`array.rangeGen()`](src/array.range.ts:130): Add logger parameter
- [ ] [`arrayOf()`](src/array.of.ts:35): Add logger parameter
- [ ] [`genOf()`](src/array.of.ts:84): Add logger parameter
- [ ] All array type guard functions - Add logger parameters
- [ ] All array utility functions - Add logger parameters

#### Boolean Utilities (3 functions)

- [ ] [`equal()`](src/boolean.equal.ts:174): Add logger parameter
- [ ] [`equalAsync()`](src/boolean.equal.ts:543): Add logger parameter
- [ ] [`isPrimitive()`](src/boolean.equal.ts:69): Add logger parameter
- [ ] [`BooleanNot()`](src/boolean.not.ts:23): Add logger parameter

#### Error Utilities (25+ functions)

- [ ] All assertion functions - Add logger parameters
- [ ] All `not*OrThrow()` functions - Add logger parameters
- [ ] [`isError()`](src/error.is.ts:32): Add logger parameter

#### Function Utilities (20+ functions)

- [ ] All function composition utilities - Add logger parameters
- [ ] All function transformation utilities - Add logger parameters
- [ ] All memoization functions - Add logger parameters
- [ ] All currying and partial application functions - Add logger parameters

#### Iterable Utilities (25+ functions)

- [ ] All sync and async iterable functions - Add logger parameters
- [ ] All filtering and mapping functions - Add logger parameters
- [ ] All reduction and aggregation functions - Add logger parameters

#### String Utilities (20+ functions)

- [ ] All string validation functions - Add logger parameters
- [ ] All string transformation functions - Add logger parameters
- [ ] All string analysis functions - Add logger parameters

#### Numeric Utilities (15+ functions)

- [ ] All numeric validation functions - Add logger parameters
- [ ] All numeric operation functions - Add logger parameters
- [ ] All type guard functions - Add logger parameters

### Logger Integration Considerations

#### Logger Usage Patterns

- [ ] Define standard logging patterns for different function types
- [ ] Create logging guidelines for pure vs effectful functions
- [ ] Establish performance vs debugging tradeoffs
- [ ] Define log levels for different types of operations

#### Backward Compatibility

- [ ] Create migration strategy for existing function calls
- [ ] Consider function overloads for gradual migration
- [ ] Document breaking changes and migration guide
- [ ] Plan major version bump with comprehensive changelog

---

## 2. Named Parameters for 3+ Parameter Functions

### Overview

**Status**: Critical Priority; affects many function signatures

**Requirement**: All functions taking more than 2 parameters must use named parameter syntax
(destructured object parameters).

### Current Functions Needing Conversion

#### Functions with 3+ Parameters Requiring Conversion

**Array Functions**:

- [ ] [`chunksArray()`](src/iterable.chunks.ts:76): Convert to named parameters
- [ ] [`chunksIterable()`](src/iterable.chunks.ts:199): Convert to named parameters
- [ ] [`replicateElementAsParentContent()`](src/dom.duplicateElement.ts:33): Convert to named parameters

**Future Implementations Requiring Named Parameters**:

- [ ] **`object.get(path, obj, default?, l)`** → `object.get({ path, obj, default, l })`
- [ ] **`object.set(path, value, obj, l)`** → `object.set({ path, value, obj, l })`
- [ ] **`date.add(amount, unit, date, l)`** → `date.add({ amount, unit, date, l })`
- [ ] **`math.clamp(min, max, value, l)`** → `math.clamp({ min, max, value, l })`
- [ ] **`validate.range(min, max, value, l)`** → `validate.range({ min, max, value, l })`

### Named Parameter Design Patterns

#### Standard Pattern for 3+ Parameters

```typescript
// Pattern for functions with required parameters
export function functionName<const T,>({
  param1,
  param2,
  param3,
  l,
}: {
  param1: Type1;
  param2: Type2;
  param3: Type3;
  l: Logger;
},): ReturnType;

// Pattern with optional parameters
export function functionName<const T,>({
  param1,
  param2,
  param3,
  optional1 = defaultValue,
  l,
}: {
  param1: Type1;
  param2: Type2;
  param3: Type3;
  optional1?: Type4;
  l: Logger;
},): ReturnType;
```

#### Complex Parameter Objects

```typescript
// For functions with many parameters, group logically
export function complexFunction({
  input: { data, format, options, },
  processing: { algorithm, concurrency, timeout, },
  output: { format: outputFormat, compression, },
  l,
}: {
  input: {
    data: InputData;
    format: InputFormat;
    options?: InputOptions;
  };
  processing: {
    algorithm: AlgorithmType;
    concurrency?: number;
    timeout?: number;
  };
  output: {
    format: OutputFormat;
    compression?: boolean;
  };
  l: Logger;
},): OutputData;
```

### Migration Tasks

#### Phase 1: Audit Existing Functions

- [ ] Identify all functions with 3+ parameters in current codebase
- [ ] Create migration plan for each function signature
- [ ] Design consistent parameter grouping patterns
- [ ] Plan backward compatibility strategy

#### Phase 2: Convert Existing Functions

- [ ] Convert identified functions to named parameter syntax
- [ ] Update all usage sites across the codebase
- [ ] Update all test files to use new syntax
- [ ] Update all documentation and examples

#### Phase 3: Establish Patterns for New Functions

- [ ] Create templates for named parameter functions
- [ ] Document best practices for parameter grouping
- [ ] Add linting rules to enforce named parameter usage
- [ ] Create tooling to validate parameter patterns

---

## 3. Type Testing with `@monochromatic-dev/test-harness-core`

### Overview

**Status**: Critical Priority; quality assurance for ALL exports

**Requirement**: All functions, constants, types, and exports should be tested
using type testing capabilities with `expectTypeOf`.

### Comprehensive Type Testing Requirements

#### ALL Functions Need Type Testing (150+ functions affected)

**Function Return Type Testing**:

- [ ] **All functions** must have type tests validating return types
- [ ] **Generic functions** must test type inference across different inputs
- [ ] **Overloaded functions** must test all overload signatures
- [ ] **Async functions** must test Promise resolution types

**Function Parameter Type Testing**:

- [ ] **All functions** must validate parameter type constraints
- [ ] **Generic constraints** must be tested for proper type rejection
- [ ] **Optional parameters** must test both provided and default scenarios
- [ ] **Rest parameters** must test array type inference

#### All Constants Need Type Testing

**Exported Constants**:

- [ ] **Type constants** like [`Int`](src/numeric.type.int.ts:1),
  [`PositiveInt`](src/numeric.type.int.ts:1): Test type definitions
- [ ] **Value constants**: Test literal type inference and immutability
- [ ] **Function constants** like [`alwaysTrue()`](src/function.always.ts:1): Test return type consistency

#### All Type Exports Need Testing

**Type Utilities** (existing requirement expanded):

- [ ] **[`array.type.fixedLength.ts`](src/array.type.fixedLength.ts:1)**: Has good type testing ✓
- [ ] **[`array.type.mapTo.ts`](src/array.type.mapTo.ts:16)**: Create comprehensive type tests
- [ ] **[`array.type.tuple.ts`](src/array.type.tuple.ts:17)**: Create type tests
- [ ] **[`array.type.withoutFirst.ts`](src/array.type.withoutFirst.ts:16)**: Create type tests
- [ ] **[`iterable.type.maybe.ts`](src/iterable.type.maybe.ts:1)**: Create type tests
- [ ] **[`promise.type.ts`](src/promise.type.ts:1)**: Create type tests

**Interface and Object Type Testing**:

- [ ] **Store types** in [`any.store.shared.ts`](src/any.store.shared.ts:8): Test type definitions
- [ ] **Error assertion types** in [`error.assert.equal.type.ts`](src/error.assert.equal.type.ts:1):
  Enhance existing tests
- [ ] **Configuration object types**: Test all option object types

### Comprehensive Type Testing Patterns

#### Function Type Testing Pattern

```typescript
import {
  addTwoNumbers,
  equal,
  identity,
  // all functions to test
} from '@monochromatic-dev/module-es';
import {
  describe,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core';

describe('Function Type Testing', () => {
  test('identity preserves input type exactly', () => {
    expectTypeOf(identity,).parameter(0,).toEqualTypeOf<unknown>();
    expectTypeOf(identity<string>,).returns.toEqualTypeOf<string>();
    expectTypeOf(identity<number>,).returns.toEqualTypeOf<number>();
  });

  test('addTwoNumbers returns number type', () => {
    expectTypeOf(addTwoNumbers,).parameter(0,).toEqualTypeOf<number>();
    expectTypeOf(addTwoNumbers,).parameter(1,).toEqualTypeOf<number>();
    expectTypeOf(addTwoNumbers,).returns.toEqualTypeOf<number>();
  });

  test('equal function type inference', () => {
    expectTypeOf(equal,).parameter(0,).toEqualTypeOf<unknown>();
    expectTypeOf(equal,).parameter(1,).toEqualTypeOf<unknown>();
    expectTypeOf(equal,).returns.toEqualTypeOf<boolean>();
  });
});
```

#### Constant Type Testing Pattern

```typescript
describe('Constant Type Testing', () => {
  test('numeric type constants', () => {
    expectTypeOf<Int>().toEqualTypeOf<number>();
    expectTypeOf<PositiveInt>().toEqualTypeOf<number>();
    expectTypeOf<NegativeInt>().toEqualTypeOf<number>();
  });

  test('function constants preserve types', () => {
    expectTypeOf(alwaysTrue,).returns.toEqualTypeOf<true>();
    expectTypeOf(alwaysFalse,).returns.toEqualTypeOf<false>();
  });
});
```

#### Generic Function Type Testing

```typescript
describe('Generic Function Type Testing', () => {
  test('pipe functions preserve type flow', () => {
    const pipeline = pipe(
      (x: number,) => x.toString(),
      (s: string,) => s.length,
      (n: number,) => n > 0,
    );
    expectTypeOf(pipeline,).parameter(0,).toEqualTypeOf<number>();
    expectTypeOf(pipeline,).returns.toEqualTypeOf<boolean>();
  });

  test('array type inference', () => {
    expectTypeOf(arrayOf(1, 2, 3,),).toEqualTypeOf<readonly [1, 2, 3,]>();
    expectTypeOf(arrayRange,).parameter(0,).toEqualTypeOf<number>();
    expectTypeOf(arrayRange,).returns.toEqualTypeOf<number[]>();
  });
});
```

### Implementation Plan for Universal Type Testing

#### Phase 1: Infrastructure (Week 1)

- [ ] **Create type testing standards** and patterns for all export types
- [ ] **Design test organization** for functions, constants, and types
- [ ] **Create templates** for different types of exports
- [ ] **Set up automated validation** for type test coverage

#### Phase 2: Existing Functions (Weeks 2-4)

- [ ] **Week 2**: Add type tests for all `any.*`, `boolean.*`, `error.*` functions
- [ ] **Week 3**: Add type tests for all `function.*`, `array.*`, `string.*` functions
- [ ] **Week 4**: Add type tests for all `iterable.*`, `numeric.*`, remaining functions

#### Phase 3: Constants and Types (Week 5)

- [ ] **Add type tests** for all exported type constants
- [ ] **Add type tests** for all exported interfaces and type aliases
- [ ] **Add type tests** for all branded types and type utilities

#### Phase 4: Integration Testing (Week 6)

- [ ] **Test function composition** type inference chains
- [ ] **Test complex generic scenarios** with real-world usage patterns
- [ ] **Test edge cases** and error conditions in type system
- [ ] **Validate type performance** for complex inference scenarios

### Type Testing Coverage Goals

**Target Coverage**:

- [ ] **100% of exported functions** have comprehensive type tests
- [ ] **100% of exported constants** have type validation tests
- [ ] **100% of exported types** have proper type assertion tests
- [ ] **All generic constraints** tested with valid and invalid inputs
- [ ] **All function overloads** tested for proper type resolution
- [ ] **All branded types** tested for proper type safety

### Success Criteria for Universal Type Testing

- [ ] **Every export** has corresponding type test validation
- [ ] **Type inference** works correctly in all composition scenarios
- [ ] **Generic constraints** properly reject invalid types
- [ ] **Function overloads** resolve to correct signatures
- [ ] **Error conditions** produce expected type errors
- [ ] **Performance** of type inference remains acceptable
- [ ] **Type safety** prevents runtime type errors through compile-time validation

---

## Implementation Priority for Major Refactors

### Phase 1: Design and Planning (Week 1)

1. **Design logger integration patterns** for different function types
2. **Create named parameter conventions** and grouping strategies
3. **Establish type testing standards** and patterns
4. **Plan migration strategy** for breaking changes

### Phase 2: Infrastructure (Week 2)

1. **Create utility functions** for logger integration
2. **Design parameter object types** for named parameters
3. **Create type testing infrastructure** and templates
4. **Plan backward compatibility** approach

### Phase 3: Systematic Implementation (Months 1-3)

1. **Month 1**: Convert all core utilities (any, error, function categories)
2. **Month 2**: Convert all iterable and array utilities
3. **Month 3**: Convert all string, numeric, and remaining utilities

### Phase 4: Validation and Documentation (Month 4)

1. **Comprehensive testing** of all refactored functions
2. **Documentation updates** for all new signatures
3. **Migration guide creation** for existing users
4. **Performance validation** after refactoring

---

## Success Criteria for API Refactors

- [ ] **100% of functions** have logger parameter integration
- [ ] **All 3+ parameter functions** use named parameter syntax
- [ ] **All type utilities** have comprehensive type testing
- [ ] **Zero regression** in functionality after refactoring
- [ ] **Performance maintained** or improved after changes
- [ ] **Comprehensive documentation** updated for all signature changes
- [ ] **Migration guide** provided for breaking changes
- [ ] **Backward compatibility** strategy documented and implemented

## Breaking Change Management

### Version Planning

- [ ] **Major version bump** required (1.0.0 → 2.0.0)
- [ ] **Comprehensive changelog** documenting all breaking changes
- [ ] **Migration guide** with before/after examples
- [ ] **Deprecation period** for gradual migration

### Communication Strategy

- [ ] **Early communication** about planned breaking changes
- [ ] **Beta release** with new API for early feedback
- [ ] **Documentation** of migration process and timeline
- [ ] **Support period** for old API during transition

---

## Cross-References

- [**Missing Implementations Todo**](TODO.missing-implementations.md): All new functions must follow these patterns
- [**Testing Todo**](TODO.testing.md): Testing updates required for new signatures
- [**TSDoc Todo**](TODO.tsdoc-improvements.md): Documentation updates for new API patterns
- [**Main Build System**](../../TODO.build-system.md): Build system implications of major refactors
