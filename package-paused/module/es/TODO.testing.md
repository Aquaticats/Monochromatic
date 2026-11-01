# Module ES - Testing Todo

## Known browser test failures

- [ ] `redirectingTo.browser.test.ts` fails in both Chromium and Firefox:
      `globalThis.moduleEs.onLoadRedirectingTo` is not a function.
      The test harness imports `* as moduleEs` from the neutral bundle,
      but `onLoadRedirectingTo` is not a direct named export; it is nested
      under `deprecated.dom`. Either the test needs to access the correct path
      or the function needs re-exporting at the top level.

## Type testing coverage analysis

### Type Testing Requirements (Critical)

#### Universal Type Testing Requirement

**Status**: Critical Priority: ALL exports need type testing

**Requirement**: All functions, constants, types, and exports must have type testing with `expectTypeOf`.

**Scope**: Every single export from the module requires type validation:

- **150+ functions**: Parameter and return type validation
- **50+ constants**: Type constant and value constant validation
- **25+ type utilities**: Type-level computation validation
- **All interfaces**: Object shape and property validation

#### Functions Requiring Type Testing (ALL Functions)

**Any Utilities**: Type tests needed:

- [ ] [`any.constant()`](src/any.constant.ts:32): Test return function type and closure type
- [ ] [`any.echo()`](src/any.echo.ts:26): Test generator type and yield type
- [ ] [`any.identity()`](src/any.identity.ts:25): Test perfect type preservation
- [ ] [`any.toExport()`](src/any.toExport.ts:45): Test string return and any input handling
- [ ] [`any.typeOf()`](src/any.typeOf.ts:25): Test string literal return types
- [ ] [`any.when()`](src/any.when.ts:24): Test conditional type inference
- [ ] [`any.whenAsync()`](src/any.when.ts:65): Test Promise type wrapping

**Array Utilities**: Type tests needed:

- [ ] [`arrayRange()`](src/array.range.ts:73): Test number array return type
- [ ] [`arrayRangeGen()`](src/array.range.ts:130): Test generator type inference
- [ ] [`arrayOf()`](src/array.of.ts:35): Test tuple type preservation
- [ ] [`genOf()`](src/array.of.ts:84): Test generator element type inference
- [ ] All array type guards - Test type narrowing and predicate types

**Boolean Utilities**: Type tests needed:

- [ ] [`equal()`](src/boolean.equal.ts:174): Test boolean return with any inputs
- [ ] [`equalAsync()`](src/boolean.equal.ts:543): Test Promise<boolean> return
- [ ] [`isPrimitive()`](src/boolean.equal.ts:69): Test type predicate functionality
- [ ] [`BooleanNot()`](src/boolean.not.ts:23): Test boolean return type

**Error Utilities**: Type tests needed:

- [ ] All assertion functions - Test void return and throwing behavior
- [ ] All `not*OrThrow()` functions - Test type narrowing and never returns
- [ ] [`isError()`](src/error.is.ts:32): Test Error type predicate

**Function Utilities**: Type tests needed:

- [ ] [`pipe()`](src/function.pipe.ts:1329) and overloads - Test type flow through composition
- [ ] [`piped()`](src/function.pipe.ts:280) and overloads - Test immediate composition types
- [ ] [`pipeAsync()`](src/function.pipe.ts:815) and overloads - Test async composition types
- [ ] [`pipedAsync()`](src/function.pipe.ts:19) and overloads - Test async immediate composition
- [ ] [`curry()`](src/function.curry.ts:1): Test curried function type inference
- [ ] [`partial()`](src/function.partial.ts:46): Test partial application types
- [ ] All other function utilities - Test type preservation and inference

**Iterable Utilities**: Type tests needed:

- [ ] All sync iterable functions - Test element type preservation
- [ ] All async iterable functions - Test async element type handling
- [ ] All generator functions - Test yield and return types
- [ ] All filtering functions - Test type narrowing with predicates

**String Utilities**: Type tests needed:

- [ ] All string validation functions - Test type predicate behavior
- [ ] All string transformation functions - Test string return types

**Numeric Utilities**: Type tests needed:

- [ ] All numeric type guards - Test type predicate functionality
- [ ] All numeric operations - Test number/bigint type preservation
- [ ] All validation functions - Test type narrowing behavior

#### Constants Requiring Type Testing

**Type Constants**:

- [ ] **Numeric types**: [`Int`](src/numeric.type.int.ts:1),
  [`PositiveInt`](src/numeric.type.int.ts:1), [`NegativeInt`](src/numeric.type.int.ts:1), etc.
- [ ] **Array types**: [`Tuple`](src/array.type.tuple.ts:17), [`ArrayFixedLength`](src/array.type.fixedLength.ts:1),
  etc.
- [ ] **String types**: [`DigitString`](src/string.digits.ts:1), [`LangString`](src/string.language.ts:1), etc.

**Value Constants**:

- [ ] **Function constants**: [`alwaysTrue()`](src/function.always.ts:1), [`emptyFunction()`](src/function.is.ts:101)
- [ ] **Utility constants**: Any exported constant values

### Type Testing Implementation Examples

#### Function Type Testing

```typescript
describe('Function Type Testing', () => {
  test('identity function type preservation', () => {
    // Test generic type preservation
    expectTypeOf(identity<string>,).parameter(0,).toEqualTypeOf<string>();
    expectTypeOf(identity<string>,).returns.toEqualTypeOf<string>();
    expectTypeOf(identity<number>,).parameter(0,).toEqualTypeOf<number>();
    expectTypeOf(identity<number>,).returns.toEqualTypeOf<number>();

    // Test literal type preservation
    expectTypeOf(identity<'hello'>,).returns.toEqualTypeOf<'hello'>();
  });

  test('type guard function behavior', () => {
    expectTypeOf(isError,).parameter(0,).toEqualTypeOf<unknown>();
    expectTypeOf(isError,).returns.toEqualTypeOf<boolean>();

    // Test type predicate functionality would need runtime testing
    // but we can test the function signature types
  });

  test('async function type handling', () => {
    expectTypeOf(equalAsync,).parameter(0,).toEqualTypeOf<any>();
    expectTypeOf(equalAsync,).parameter(1,).toEqualTypeOf<any>();
    expectTypeOf(equalAsync,).returns.toEqualTypeOf<Promise<boolean>>();
  });
});
```

#### Generic Function Type Testing

```typescript
describe('Generic Function Type Testing', () => {
  test('pipe function type flow', () => {
    const result = pipe(
      (x: number,) => x.toString(),
      (s: string,) => s.length,
    );
    expectTypeOf(result,).parameter(0,).toEqualTypeOf<number>();
    expectTypeOf(result,).returns.toEqualTypeOf<number>();
  });

  test('array function generic preservation', () => {
    expectTypeOf(arrayOf<1, 2, 3>,).returns.toEqualTypeOf<
      readonly [1, 2, 3,]
    >();
    expectTypeOf(arrayRange,).returns.toEqualTypeOf<number[]>();
  });
});
```

#### Constant and Type Testing

```typescript
describe('Type Constant Testing', () => {
  test('numeric type definitions', () => {
    expectTypeOf<Int>().toEqualTypeOf<number>();
    expectTypeOf<PositiveInt>().toMatchTypeOf<Int>();
    expectTypeOf<NegativeInt>().toMatchTypeOf<Int>();
  });

  test('array type utilities', () => {
    expectTypeOf<Tuple<[1, 2, 3,]>>().toEqualTypeOf<readonly [1, 2, 3,]>();
    expectTypeOf<WithoutFirst<[1, 2, 3,]>>().toEqualTypeOf<[2, 3,]>();
  });
});
```

### Type Testing Files to Create/Enhance

#### New Type Testing Files Required

- [ ] **`any.type.test.ts`**: Type tests for all any utilities
- [ ] **`array.type.test.ts`**: Type tests for all array functions and types
- [ ] **`boolean.type.test.ts`**: Type tests for boolean operations
- [ ] **`error.type.test.ts`**: Type tests for error utilities
- [ ] **`function.type.test.ts`**: Type tests for function utilities
- [ ] **`iterable.type.test.ts`**: Type tests for iterable functions
- [ ] **`string.type.test.ts`**: Type tests for string utilities
- [ ] **`numeric.type.test.ts`**: Type tests for numeric functions

#### Enhanced Type Testing for Complex Functions

- [ ] **Function composition chains**: Test complex type inference scenarios
- [ ] **Generic constraint validation**: Test that invalid types are rejected
- [ ] **Overload resolution**: Test that correct overloads are selected
- [ ] **Async type handling**: Test Promise type wrapping and unwrapping

### Success Criteria for Universal Type Testing

- [ ] **100% of functions** have type tests validating signatures and behavior
- [ ] **100% of constants** have type validation tests
- [ ] **100% of exported types** have proper type assertion tests
- [ ] **All generic functions** tested with various input types
- [ ] **All type predicates** tested for proper narrowing behavior
- [ ] **All overloaded functions** tested for signature resolution
- [ ] **Complex composition scenarios** validated for type correctness
- [ ] **Error conditions** produce expected type-level errors

## Testing Coverage Analysis

### Files Missing Unit Tests (High Priority)

**Status**: Critical; Many core functions lack test coverage

#### Core Utilities Missing Tests

- [ ] **[`any.echo.ts`](src/any.echo.ts:26)**: Generator that yields same value infinitely
- [ ] **[`any.when.ts`](src/any.when.ts:24)**: Conditional execution utilities
- [ ] **[`array.is.ts`](src/array.is.ts:51)**: Array type guards and validation
- [ ] **[`function.always.ts`](src/function.always.ts:1)**: Always-true/false functions
- [ ] **[`function.curry.ts`](src/function.curry.ts:1)**: Function currying utilities
- [ ] **[`function.ignoreExtraArgs.ts`](src/function.ignoreExtraArgs.ts:40)**: Argument filtering wrapper
- [ ] **[`generator.is.ts`](src/generator.is.ts:1)**: Generator type guards
- [ ] **[`map.is.ts`](src/map.is.ts:28)**: Map type guards and validation
- [ ] **[`object.is.ts`](src/object.is.ts:30)**: Object type guards
- [ ] **[`set.is.ts`](src/set.is.ts:27)**: Set type guards and validation

#### Type-Only Files Missing Tests

- [ ] **[`array.type.mapTo.ts`](src/array.type.mapTo.ts:16)**: Array mapping type utilities
- [ ] **[`array.type.tuple.ts`](src/array.type.tuple.ts:17)**: Tuple type definitions
- [ ] **[`array.type.withoutFirst.ts`](src/array.type.withoutFirst.ts:16)**: Array tail type utilities
- [ ] **[`iterable.type.maybe.ts`](src/iterable.type.maybe.ts:1)**: Maybe async iterable types
- [ ] **[`promise.type.ts`](src/promise.type.ts:1)**: Promise type utilities

#### Platform-Specific Missing Tests

- [ ] **[`fs.default.ts`](src/fs.default.ts:1)**: Default filesystem implementation
- [ ] **[`fs.fs.default.ts`](src/fs.fs.default.ts:18)**: Browser filesystem polyfill
- [ ] **[`fs.pathJoin.shared.ts`](src/fs.pathJoin.shared.ts:1)**: Shared path utilities
- [ ] **[`fs.pathParse.default.ts`](src/fs.pathParse.default.ts:3)**: Default path parsing
- [ ] **[`fs.pathParse.node.ts`](src/fs.pathParse.node.ts:1)**: Node.js path parsing
- [ ] **[`string.fs.default.ts`](src/string.fs.default.ts:40)**: Browser string filesystem
- [ ] **[`string.fs.node.ts`](src/string.fs.node.ts:42)**: Node.js string filesystem
- [ ] **[`string.fs.shared.ts`](src/string.fs.shared.ts:59)**: Shared string filesystem

#### DOM Utilities Missing Tests

- [ ] **[`dom.duplicateElement.ts`](src/dom.duplicateElement.ts:33)**: Element replication utilities
- [ ] **[`dom.prompt.ts`](src/dom.prompt.ts:22)**: Custom prompt dialog
- [ ] **[`dom.setCssFromParam.ts`](src/dom.setCssFromParam.ts:30)**: CSS parameter injection

#### Advanced Features Missing Tests

- [ ] **[`any.ReplicatingStore.ts`](src/any.ReplicatingStore.ts:201)**: Distributed storage system
- [ ] **[`any.store.shared.ts`](src/any.store.shared.ts:8)**: Store type definitions
- [ ] **[`deprecated.testing.ts`](src/deprecated.testing.ts:70)**: Legacy testing utilities
- [ ] **[`indexedDb.executeTransaction.ts`](src/indexedDb.executeTransaction.ts:13)**: IndexedDB transactions
- [ ] **[`indexedDb.open.ts`](src/indexedDb.open.ts:7)**: IndexedDB connection utilities

#### CLI and Build Tools Missing Tests

- [ ] **[`cli.append.ts`](src/cli.append.ts:36)**: File append CLI utility
- [ ] **[`dirent.path.ts`](src/dirent.path.ts:5)**: Directory entry path utilities
- [ ] **[`testLogger.index.ts`](src/testLogger.index.ts:1)**: Test logging utilities

#### Utilities Missing Tests

- [ ] **[`iterable.toString.ts`](src/iterable.toString.ts:1)**: Iterable string conversion
- [ ] **[`iterable.trim.ts`](src/iterable.trim.ts:25)**: Iterable trimming utilities
- [ ] **[`iterable.merge.ts`](src/iterable.merge.ts:53)**: Iterable merging (commented out?)

### Missing Test Files (Must Be Created)

Based on implementation files that have no corresponding `.unit.test.ts`:

#### Essential Test Files to Create

1. **`any.echo.unit.test.ts`**: Test infinite value generator
2. **`any.when.unit.test.ts`**: Test conditional execution utilities
3. **`array.is.unit.test.ts`**: Test array type guards
4. **`function.always.unit.test.ts`**: Test always-true/false functions
5. **`function.curry.unit.test.ts`**: Test function currying
6. **`function.ignoreExtraArgs.unit.test.ts`**: Test argument filtering
7. **`generator.is.unit.test.ts`**: Test generator type guards
8. **`map.is.unit.test.ts`**: Test Map type guards
9. **`object.is.unit.test.ts`**: Test object type guards
10. **`set.is.unit.test.ts`**: Test Set type guards

#### Type-Level Testing Files to Create

11. **`array.type.mapTo.unit.test.ts`**: Test array mapping types
12. **`array.type.tuple.unit.test.ts`**: Test tuple type definitions
13. **`array.type.withoutFirst.unit.test.ts`**: Test array tail types
14. **`iterable.type.maybe.unit.test.ts`**: Test maybe async types
15. **`promise.type.unit.test.ts`**: Test promise type utilities

#### Platform-Specific Test Files to Create

16. **`fs.default.unit.test.ts`**: Test default filesystem
17. **`fs.fs.default.unit.test.ts`**: Test browser filesystem polyfill
18. **`fs.pathJoin.shared.unit.test.ts`**: Test shared path utilities
19. **`fs.pathParse.default.unit.test.ts`**: Test default path parsing
20. **`fs.pathParse.node.unit.test.ts`**: Test Node.js path parsing

### Functions Needing Improved Test Coverage

#### Existing Tests That Need Enhancement

- [ ] **[`boolean.equal.unit.test.ts`](src/boolean.equal.unit.test.ts:16)**: Comprehensive but complex
  - Add edge case coverage for async comparisons
  - Add performance tests for large data structures
  - Add memory leak tests for recursive structures

- [ ] **[`any.toExport.unit.test.ts`](src/any.toExport.unit.test.ts:17)**: Good coverage
  - Add performance tests for large objects
  - Add security tests for malicious input

- [ ] **[`function.pipe.unit.test.ts`](src/function.pipe.unit.test.ts:1)**: Comprehensive
  - Add error propagation tests
  - Add performance tests for long pipelines

#### Files with Partial Test Coverage

- [ ] **[`error.assert.equal.ts`](src/error.assert.equal.ts:28)**: Missing test file
  - Has type tests in [`error.assert.equal.type.ts`](src/error.assert.equal.type.ts:1)
  - Needs runtime behavior tests

- [ ] **[`iterable.every.ts`](src/iterable.every.ts:90)**: Missing dedicated test
  - Only tested via [`iterable.everyFail.unit.test.ts`](src/iterable.everyFail.unit.test.ts:1)

- [ ] **[`numeric.type.ints.ts`](src/numeric.type.ints.ts:1)**: Missing test file
  - Has index file [`numeric.type.ints.index.ts`](src/numeric.type.ints.index.ts:1) but no tests

- [ ] **[`numeric.type.negative.ts`](src/numeric.type.negative.ts:1)**: Missing test file
  - Type definitions need runtime validation tests

### Browser-Specific Test Files Missing

#### Files That Need Browser Testing

- [ ] **[`dom.redirectingTo.ts`](src/dom.redirectingTo.ts:33)**:
  Has browser test: ✓ [`dom.redirectingTo.browser.test.ts`](src/dom.redirectingTo.browser.test.ts:1)
- [ ] **[`fs.ensurePath.ts`](src/fs.ensurePath.ts:17)**:
  Has browser test: ✓ [`fs.ensurePath.browser.test.ts`](src/fs.ensurePath.browser.test.ts:1)

#### DOM Functions Needing Browser Tests

- [ ] **`dom.duplicateElement.ts`**: Create `dom.duplicateElement.browser.test.ts`
- [ ] **`dom.prompt.ts`**: Create `dom.prompt.browser.test.ts`
- [ ] **`dom.setCssFromParam.ts`**: Create `dom.setCssFromParam.browser.test.ts`
- [ ] **`string.limitedGetComputedCss.ts`**: Has unit test, may need browser test

### Performance Benchmark Files

#### Files with Benchmarks

- [ ] **[`promises.some.bench.ts`](src/promises.some.bench.ts:1)**: Performance benchmark exists ✓

#### Functions Needing Benchmarks

- [ ] **Array utilities**: [`array.range.ts`](src/array.range.ts:73) vs generator version
- [ ] **Iteration utilities**: Sync vs async performance comparisons
- [ ] **Function composition**: [`function.pipe.ts`](src/function.pipe.ts:19) performance with long pipelines
- [ ] **Equality comparisons**: [`boolean.equal.ts`](src/boolean.equal.ts:174) for large structures

## Test Quality Issues

### Files Needing Better TSDoc in Tests

**Status**: Normal Priority; Documentation in tests

- [ ] **[`any.unit.test.ts`](src/any.unit.test.ts:18)**: Multiple functions, needs organization
- [ ] **[`boolean.equal.unit.test.ts`](src/boolean.equal.unit.test.ts:16)**: Complex, needs better documentation
- [ ] **[`function.pipe.unit.test.ts`](src/function.pipe.unit.test.ts:1)**: Many overloads, needs examples

### Test Organization Issues

**Status**: Normal Priority; Test maintainability

- [ ] **Split combined test files** like [`any.unit.test.ts`](src/any.unit.test.ts:18)
  - Contains tests for multiple functions
  - Should be split into function-specific test files

- [ ] **Add missing describe blocks** for functions in combined files
  - Some functions tested but not properly organized

### Test Performance Issues

**Status**: Low Priority; Test efficiency

- [ ] **Long-running tests** need optimization
- [ ] **Large data structure tests** need efficiency improvements
- [ ] **Async test timeout configuration** needs review

## Testing Infrastructure Issues

### Missing Test Setup Files

**Status**: Normal Priority; Test infrastructure

- [ ] **Test fixtures** need organization and reuse
- [ ] **Test utilities** could be extracted and shared
- [ ] **Mock factories** for complex objects like `Store`

### Platform-Specific Testing

**Status**: Normal Priority; Multi-platform validation

- [ ] **Node.js-specific tests** for platform functions
- [ ] **Browser-specific tests** for DOM and Web APIs
- [ ] **Cross-platform compatibility tests**

## Success Criteria

- [ ] All implementation files have corresponding unit test files
- [ ] Test coverage reaches 95%+ for all modules
- [ ] All type-level utilities have type tests
- [ ] Platform-specific functions have appropriate browser/node tests
- [ ] Performance-critical functions have benchmark tests
- [ ] Test files are well-organized and documented
- [ ] Test infrastructure supports efficient development
- [ ] Tests run reliably across all supported platforms

## Implementation Priority

### Phase 1: Critical Missing Tests (Week 1)

1. **Core utilities**: `any.echo`, `any.when`
2. **Type guards**: `array.is`, `map.is`, `object.is`, `set.is`
3. **Function utilities**: `function.always`, `function.curry`, `function.ignoreExtraArgs`

### Phase 2: Platform-Specific Tests (Week 2)

1. **Filesystem utilities**: All `fs.*` and `string.fs.*` functions
2. **DOM utilities**: All `dom.*` functions with browser tests
3. **IndexedDB utilities**: Database interaction functions

### Phase 3: Advanced Features (Week 3)

1. **Storage system**: `any.ReplicatingStore` comprehensive testing
2. **CLI utilities**: `cli.*` functions with integration tests
3. **Build tools**: `mise.*` scripts with integration tests

### Phase 4: Test Quality (Week 4)

1. **Split combined test files** into function-specific files
2. **Add performance benchmarks** for critical functions
3. **Enhance test documentation** and organization

## Cross-References

- [**Build System Todo**](../../TODO.build-system.md#validation-and-testing): Related build system testing
- [**Code Quality Todo**](../../TODO.code-quality.md#testing-requirements-and-standards):
  Testing standards and requirements
- [**Performance Todo**](../../TODO.performance.md#testing-automation): Performance testing integration
