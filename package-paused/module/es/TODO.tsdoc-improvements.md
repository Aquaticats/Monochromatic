# Module ES - TSDoc Improvements Todo

## TSDoc Documentation Gaps

### Files Needing Complete TSDoc Documentation

#### High Priority - Core Functions Missing Documentation

- [ ] **[`any.echo.ts`](src/any.echo.ts:26)**: [`echo()`](src/any.echo.ts:26)
  - Missing complete TSDoc with examples
  - Need to document infinite generator behavior
  - Add usage examples with `Array.from()` and breaking conditions

- [ ] **[`function.always.ts`](src/function.always.ts:1)**: [`alwaysTrue()`](src/function.always.ts:1)
  - Missing TSDoc completely
  - Need to document use cases in filtering and predicate scenarios
  - Add examples with array methods

- [ ] **[`object.is.ts`](src/object.is.ts:30)**: [`isObject()`](src/object.is.ts:30)
  - Basic documentation, needs examples
  - Document difference from `typeof obj === 'object'`
  - Add examples with class instances vs plain objects

- [ ] **[`map.is.ts`](src/map.is.ts:28)**: [`isMap()`](src/map.is.ts:28) and [`isWeakMap()`](src/map.is.ts:56)
  - Missing comprehensive documentation
  - Need examples showing type narrowing
  - Document use cases in type guards

#### Array Utilities Needing Better Documentation

- [ ] **[`array.is.ts`](src/array.is.ts:51)**: Multiple functions need enhancement:
  - [`isEmptyArray()`](src/array.is.ts:51): Add type narrowing examples
  - [`isArrayEmpty()`](src/array.is.ts:83): Document vs `isEmptyArray()` differences
  - [`isArrayNonEmpty()`](src/array.is.ts:117): Add generic type examples
  - [`arrayIsNonEmpty()`](src/array.is.ts:189): Document functional usage patterns

- [ ] **[`array.range.ts`](src/array.range.ts:73)**: Functions need better docs:
  - [`arrayRange()`](src/array.range.ts:73): Add performance vs generator comparison
  - [`arrayRangeGen()`](src/array.range.ts:130): Document memory efficiency benefits

#### String Utilities Documentation Gaps

- [ ] **[`string.trim.ts`](src/string.trim.ts:20)**: Multiple functions:
  - [`trimEndWith()`](src/string.trim.ts:20): Add examples with different trimmers
  - [`trimStartWith()`](src/string.trim.ts:59): Document edge cases

- [ ] **[`string.capitalize.ts`](src/string.capitalize.ts:87)**: [`capitalizeString()`](src/string.capitalize.ts:87)
  - Missing locale-specific examples
  - Document Unicode handling capabilities

#### Function Utilities Documentation

- [ ] **[`function.deConcurrency.ts`](src/function.deConcurrency.ts:1)**:
  [`deConcurrency()`](src/function.deConcurrency.ts:1)
  - Missing comprehensive documentation
  - Need examples of concurrent execution prevention
  - Document performance implications

- [ ] **[`function.ignoreExtraArgs.ts`](src/function.ignoreExtraArgs.ts:40)**:
  [`ignoreExtraArgs()`](src/function.ignoreExtraArgs.ts:40)
  - Good documentation, needs usage examples
  - Add examples with callback scenarios

#### File System Utilities Documentation

- [ ] **[`fs.emptyPath.ts`](src/fs.emptyPath.ts:20)**: Multiple functions need docs:
  - [`emptyPath()`](src/fs.emptyPath.ts:20): Document query parameter handling
  - [`emptyDir()`](src/fs.emptyPath.ts:35): Add directory structure examples
  - [`emptyFile()`](src/fs.emptyPath.ts:49): Document file creation behavior
  - [`removeEmptyFilesInDir()`](src/fs.emptyPath.ts:58): Add cleanup examples

- [ ] **[`fs.pathJoin.shared.ts`](src/fs.pathJoin.shared.ts:1)**: Utility functions:
  - [`trimPathTrailingSlash()`](src/fs.pathJoin.shared.ts:1): Add edge case examples
  - [`trimPathLeadingSlash()`](src/fs.pathJoin.shared.ts:5): Document root path handling

### Functions with Inadequate TSDoc

#### Complex Functions Needing Enhanced Documentation

- [ ] **[`boolean.equal.ts`](src/boolean.equal.ts:174)**:
  [`equal()`](src/boolean.equal.ts:174) and [`equalAsync()`](src/boolean.equal.ts:543)
  - Has basic documentation, needs comprehensive examples
  - Document deep equality algorithm
  - Add performance considerations for large objects
  - Document async comparison behavior

- [ ] **[`any.toExport.ts`](src/any.toExport.ts:45)**: [`toExport()`](src/any.toExport.ts:45)
  - Missing detailed documentation
  - Need examples of code generation use cases
  - Document security considerations for eval-safe output

- [ ] **[`string.limitedGetComputedCss.ts`](src/string.limitedGetComputedCss.ts:533)**: Multiple functions:
  - All `lGCC_*` functions need better documentation
  - [`limitedGetComputedCss()`](src/string.limitedGetComputedCss.ts:533): Main function needs comprehensive docs
  - Document CSS parsing limitations and security model

#### Async Functions Needing Better Documentation

- [ ] **[`any.when.ts`](src/any.when.ts:24)**: Conditional functions:
  - [`when()`](src/any.when.ts:24): Add functional programming examples
  - [`whenAsync()`](src/any.when.ts:65): Document async predicate patterns

#### Error Handling Documentation

- [ ] **[`error.throw.ts`](src/error.throw.ts:34)**: Multiple functions need enhancement:
  - All `not*OrThrow()` functions have basic docs but need more examples
  - Document type narrowing behavior
  - Add examples of proper error handling patterns

### Platform-Specific Documentation Gaps

#### Node.js Platform Functions

- [ ] **[`fs.fs.node.ts`](src/fs.fs.node.ts:1)**: Node.js filesystem functions
  - Missing TSDoc for all exported functions
  - Need Node.js-specific examples and platform limitations

#### Browser Platform Functions

- [ ] **[`fs.fs.default.ts`](src/fs.fs.default.ts:18)**: Browser filesystem polyfill
  - [`getFsPromises()`](src/fs.fs.default.ts:18): Missing complete documentation
  - Document browser limitations and fallback behavior

- [ ] **[`dom.*.ts` files](src/dom.duplicateElement.ts:33)**: All DOM utilities:
  - [`replicateElementAsParentContent()`](src/dom.duplicateElement.ts:33)
  - [`deepCloneNode()`](src/dom.duplicateElement.ts:57)
  - [`replicateElementAsContentOf()`](src/dom.duplicateElement.ts:61)
  - [`onLoadRedirectingTo()`](src/dom.redirectingTo.ts:33)
  - [`onLoadSetCssFromUrlParams()`](src/dom.setCssFromParam.ts:30)

### Missing Example Patterns

#### Common TSDoc Issues Across Files

- [ ] **Missing `@example` tags**: Many functions lack practical examples
- [ ] **Incomplete parameter documentation**: Parameters described but no context
- [ ] **Missing return value details**: Return types documented but not return value meaning
- [ ] **Missing error documentation**: Functions that throw need `@throws` tags
- [ ] **Missing performance notes**: Complex functions need performance characteristics
- [ ] **Missing browser compatibility**: Platform-specific functions need compatibility notes

## TSDoc Standards Compliance

### Functions Not Following TSDoc Standards

#### Parameter Documentation Issues

**Status**: High Priority; Consistency

Functions with poor parameter documentation:

- [ ] Review all functions for parameter descriptions following the rule:
  "Avoid `the`, `a`, `an` in `@param` descriptions"
- [ ] Ensure parameter descriptions add context beyond just repeating the name
- [ ] Fix parameter descriptions that just restate the parameter name

#### Return Value Documentation Issues

**Status**: Normal Priority; Clarity

Functions missing meaningful return value documentation:

- [ ] Add meaningful `@returns` descriptions that explain what the return value represents
- [ ] For async functions, document the resolved value, not just "Promise"
- [ ] Document special return values (null, undefined, empty arrays, etc.)

#### Missing Example Documentation

**Status**: Normal Priority; Developer experience

Functions that need `@example` tags with practical usage:

- [ ] All type guard functions - show type narrowing in action
- [ ] Complex utility functions - show real-world usage scenarios
- [ ] Async functions - show proper await usage and error handling
- [ ] Composition functions - show chaining and pipeline examples

### Complex Functions Needing Comprehensive Documentation

#### Advanced Features

- [ ] **[`any.ReplicatingStore.ts`](src/any.ReplicatingStore.ts:201)**: Entire class needs documentation:
  - Class-level documentation explaining distributed storage concept
  - Method-level documentation for all public methods
  - Property documentation for configuration options
  - Examples of storage configuration and usage patterns

- [ ] **[`deprecated.testing.ts`](src/deprecated.testing.ts:70)**: Testing framework:
  - [`suite()`](src/deprecated.testing.ts:70) function overloads need documentation
  - [`test()`](src/deprecated.testing.ts:129) function overloads need examples
  - Document migration path from deprecated functions

#### Mathematical and Algorithmic Functions

- [ ] **[`numeric.add.ts`](src/numeric.add.ts:34)**: Addition utilities:
  - Document mathematical properties and associativity
  - Add examples with different numeric types
  - Document precision considerations for large numbers

## Documentation Quality Issues

### Inconsistent Documentation Patterns

#### Format Inconsistencies

**Status**: Normal Priority; Standardization

- [ ] **Inconsistent example formatting** across functions
- [ ] **Inconsistent parameter naming** in documentation vs implementation
- [ ] **Inconsistent return value descriptions**: some detailed, others minimal

#### Missing Cross-References

**Status**: Low Priority; Navigation

- [ ] **Related function references** missing in TSDoc
- [ ] **See-also tags** for similar functions
- [ ] **Links to related concepts** in other modules

### Advanced Documentation Features

#### Missing Specialized Tags

**Status**: Low Priority; Enhanced developer experience

- [ ] **`@since` tags** for version information
- [ ] **`@deprecated` tags** for functions being phased out
- [ ] **`@internal` tags** for implementation details
- [ ] **`@alpha`, `@beta` tags** for experimental features

## Success Criteria

- [ ] All exported functions have comprehensive TSDoc documentation
- [ ] All complex functions have practical usage examples
- [ ] All parameters and return values clearly documented
- [ ] Platform-specific functions have compatibility notes
- [ ] Error conditions documented with `@throws` tags
- [ ] Performance characteristics documented for complex functions
- [ ] Cross-references between related functions established
- [ ] Documentation follows established patterns consistently

## Implementation Priority

### Phase 1: Critical Functions (Week 1)

1. **Core utilities missing documentation**: `any.echo`, `function.always`, `object.is`
2. **Type guards**: All `is*()` functions in various modules
3. **Platform-specific functions**: DOM, filesystem, logging functions

### Phase 2: Complex Features (Week 2)

1. **Advanced classes**: `any.ReplicatingStore` comprehensive documentation
2. **Complex algorithms**: `boolean.equal`, `any.toExport`, CSS parsing functions
3. **Async patterns**: All async functions with proper async documentation

### Phase 3: Enhancement (Week 3)

1. **Add missing examples** to all functions lacking `@example` tags
2. **Improve parameter descriptions** to add meaningful context
3. **Add performance notes** for computationally intensive functions

### Phase 4: Polish (Week 4)

1. **Cross-reference related functions** with see-also documentation
2. **Add version information** where relevant
3. **Standardize documentation format** across all functions

## Cross-References

- [**Code Quality Todo**](../../TODO.code-quality.md#testing-requirements-and-standards):
  TSDoc standards and linting rules
- [**Documentation Todo**](../../TODO.documentation.md#api-documentation-automation): Automated documentation generation
- [**Build System Todo**](../../TODO.build-system.md#validation-and-testing): Documentation validation in build process
