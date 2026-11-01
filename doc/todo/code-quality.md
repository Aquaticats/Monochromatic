# Code Quality & Patterns Todo

## Cross-References

- [**Build System Issues**](build-system.md#missing-export-issues-new---high-priority):
   TypeScript compilation and export fixes
- [**Security Practices**](security.md#development-security):
   Secure coding guidelines and practices
- [**Performance Patterns**](performance.md#runtime-performance):
   Performance-focused coding patterns
- [**Package Standards**](packages.md#cross-package-improvements):
   Cross-package coding standards
- [**Automation Tools**](automation.md#code-quality-automation):
   Automated code quality improvements

## Current Linting Issues

### High Priority Lint Fixes

#### Code Quality Issues (Fix in code, not config)

1. **Variable abbreviations** (~135 total occurrences,
    down from 147)
   - Use descriptive names instead of abbreviations
   - **NEVER use single-letter variables like `i`,
      `j`,
      `k`**:
      they provide no semantic meaning
   - Exception:
      Mathematical formulas where single letters have established meaning

#### TypeScript Compilation Errors (New - Critical Priority)

**Status**:
 Critical (blocks builds)

Recent analysis shows critical TypeScript compilation errors that must be fixed:

- [ ] **Missing exports in type modules** → See [Build System Todo](build-system.md#missing-export-issues-new---high-priority)
  - `iterable.is.ts` missing multiple export declarations
  - `numeric.type.int.ts` export name mismatches
  - Module export consistency issues across packages

- [ ] **Missing function implementations**
  - `getRandomId` function missing in `any.ReplicatingStore.ts`
  - Type constraint issues in array type definitions
  - Pattern matcher type compatibility issues

**Cross-Reference**:
 See [Build System Todo](build-system.md#missing-export-issues-new---high-priority) for complete list and fixes.

#### Remaining Files to Fix

- Files with void expression issues in test assertions
- Re-inventory variable abbreviations before opening new cleanup work.
  The package-wide loop-counter and `window.` sweeps are complete:
  `rg --line-number 'for \\(let [ijk]' packages` and
  `rg --line-number '\\bwindow\\.' packages` return no matches on 2026-06-01.

### Medium Priority Issues

#### Test File Issues

- Missing JSDoc comments in non-test files (expected for internal utilities)

### Next Steps for Lint Cleanup

1. Re-inventory abbreviation usage beyond package C-style loop counters.
2. Fix remaining void expression issues in test assertions.

**Cross-Reference**:
 See [Automation Todo](automation.md#code-quality-automation) for automated linting improvements.

## Code Patterns and Best Practices

### Meilisearch Task Polling Implementation Evolution

**Learning Example**:
 The user guided progressive simplification through "Do you really need..." questions:

1. Started with mutable `let taskStatus` and `while` loop with inline constants
2. "Do you really need a mutable variable?
   " → Moved to immutable `const` inside loop
3. "Do you really need a while(true) break pattern?
   " → Changed to `while` with proper condition
4. "Do you really need a while loop at all?
   " → Changed to `for` loop with calculated iterations
5. "Do you really need a for loop?
   " → Changed to recursive helper function

#### Final Implementation: Simple Array with `findAsync`

```typescript
// Create array of 100 nulls as a counter
const polls = new Array(100,).fill(null,);

// Use findAsync where the predicate does all the work
try {
  const completedStatus = await findAsync(
    polls,
    async () => {
      const status = await client.tasks.getTask(task.taskUid,);
      if (!isTaskPending(status.status,))
        return status;
      await wait(TASK_POLL_INTERVAL_MS,);
      return false;
    },
  );

  if (completedStatus) {
    if (completedStatus.status !== 'succeeded') {
      console.error(`Task ${task.taskUid} failed:`, completedStatus.error,);
      allTasksSuccessful = false;
    }
  }
  else {
    console.error(`Task ${task.taskUid} timed out after ${TASK_TIMEOUT_MS}ms`,);
    allTasksSuccessful = false;
  }
}
catch (error) {
  console.error(error,);
  allTasksSuccessful = false;
}
```

#### Potential Future Improvements (1am Ideas)

1. **Different utility function**:
    Perhaps a `times` or `repeat` async function:
   - `const status = await repeatAsync(100, async () => { /* poll logic */ })`
   - This would eliminate the dummy array entirely

2. **Parallel task validation**:
    Instead of polling tasks sequentially:
   - Create an array of all taskUids from the batch
   - Use `Promise.all` with `findAsync` for each task
   - Or better:
      a single `findAsync` that polls ALL tasks in each iteration

**Cross-Reference**:
 See [Packages Todo](packages.md#async-iterator-utilities) for async utility function development.

### Lessons from "Do you really need..."

This questioning pattern teaches:

1. **Question every construct**:
    Each programming construct adds complexity
2. **Prefer immutability**:
    Mutable variables should be eliminated when possible
3. **Prefer declarative over imperative**:
    Loops can often be replaced with higher-order functions
4. **Extract and name concepts**:
    Helper functions like `isTaskPending` improve readability
5. **Think functionally first**:
    There's often a functional solution that's cleaner
6. **Simplify progressively**:
    Don't stop at the first working solution

**Cross-Reference**:
 See [Performance Todo](performance.runtime.md#javascript-performance) for performance-focused patterns.

## Testing Requirements and Standards

### General Testing Guidelines

- Write a corresponding test file (using `@monochromatic-dev/test-harness-core`) that aims for 100% test coverage
- Tests can only be run from workspace root using `mise run test`
- To run tests for specific patterns:
  - `mise run test:unit -- package/module/es/src/boolean.equal.unit.test.ts`
  - `mise run test:browser -- package/module/es/src/boolean.equal.browser.test.ts`

**Cross-Reference**:
 See [Automation Todo](automation.md#testing-automation) for comprehensive testing automation.

### Coverage Requirements

If certain lines or branches can't be tested,
 use V8 ignore comments:

```typescript
/* v8 ignore next -- @preserve */
if (impossibleCondition)
  throw new Error('This should never happen',);

// For multiple lines:
/* v8 ignore next 3 -- @preserve */
if (untestableCondition) {
  console.error('Untestable path',);
  return fallbackValue;
}
```

### Testing Intentional Violations

When tests intentionally violate a rule to verify behavior:

```typescript
// BAD: Adding data to satisfy the linter
expect(isError(new Error('test message',),),).toBe(true,);

// GOOD: Use disable comments for intentional violations
// oxlint-disable-next-line unicorn/error-message -- Testing error without message
expect(isError(new Error(),),).toBe(true,);
```

## TypeScript Standards and Patterns

### Function Patterns

- Always name functions.
   Prefer function declarations
- Always use parentheses around arrow function parameters (even single parameter)
- Provide explicit parameter and return types for all functions
- Use function overloads for functions with multiple call signatures

### Variable and Immutability Standards

- Prefer `const` over `let` to encourage immutability
- Strive for immutability:
   avoid reassigning variables and modifying objects in place
- **NEVER use single-letter variables**:
   they provide no semantic meaning
- **Prefer functional approaches over imperative loops**
- Remove unused variables or prefix them with underscore
- Use `satisfies` operator for type checking without widening

### Error Handling Best Practices

- **NEVER use process.
  exit()**:
   throw errors instead
- **Combine console.
  log/error messages into thrown errors**
- **Use string-dedent for multi-line error messages**
- **Always log errors in catch blocks**
- **Document expected errors with comments**

**Cross-Reference**:
 See [Security Todo](security.md#secure-coding-practices) for security-focused error handling.

## Linting Configuration Philosophy

### When to Fix vs Configure

- **High Priority**:
   Fix legitimate code issues in the codebase
- **Medium Priority**:
   Adjust configuration for team workflow
- **Never**:
   Disable rules just to avoid fixing code

### Rule Categories

1. **Code Quality Issues**:
    Always fix in code,
    not config
2. **Style Preferences**:
    Can be configured for team consistency
3. **Framework-specific**:
    May need configuration for specific use cases

### Testing-Specific Linting

- Use disable comments for intentional rule violations in tests
- Different rules may apply to test files vs production code
- Document why test files need different linting rules

## Performance and Code Efficiency

### Async Programming Standards

- Prefer `async/await` over explicit promise creation
- Use `wait()` from module-es instead of `new Promise(resolve => setTimeout(...))`
- Avoid await in loops where logically sound
- Use `Promise.all()` for concurrent operations

**Cross-Reference**:
 See [Performance Todo](performance.runtime.md#async-performance) for comprehensive async optimization.

### Functional Programming Utilities

Function composition lives in `@monochromatic-dev/pipe-core`,
 not module-es:

- Use `piped` for eager synchronous function composition
- Use `pipedAsync` for eager async function composition
- Use `pipe`/`pipeAsync` for deferred,
   reusable function pipelines
- Functions generally don't require `.bind()` for `this` context

**Cross-Reference**:
 See [Packages Todo](packages.md#module-library-packagemodulees) for functional utility expansion.

## Code Organization Standards

### File Organization

- Use `region` markers to delineate logical sections
- Group imports in specific order (built-in,
   external,
   workspace,
   relative,
   type-only)
- Always include file extensions in imports
- Use absolute imports for workspace packages

### Export Patterns

- Avoid `Object.assign` for extending typed objects
- Export at the end after all object construction is complete
- Prefer immediately invoked function expressions over mutable variables

## Security-Focused Code Quality

### Secure Coding Practices

**Status**:
 High Priority (security integration)

- [ ] Implement input validation in all user-facing functions
- [ ] Add proper error handling without information disclosure
- [ ] Use type-safe patterns to prevent runtime errors
- [ ] Implement proper logging for security events
- [ ] Add code review focus on security patterns

**Cross-Reference**:
 See [Security Todo](security.md#secure-coding-practices) for comprehensive security guidelines.

### Dependency Security Integration

- [ ] Add dependency vulnerability scanning to pre-commit hooks
- [ ] Implement automated dependency update validation
- [ ] Add license compliance checking
- [ ] Create secure dependency usage patterns

**Cross-Reference**:
 See [Security Todo](security.md#dependency-security) for dependency security management.

## Future Quality Improvements

### Automated Code Quality

- Implement automated code review checks
- Set up quality gates in CI/CD pipeline
- Monitor code quality metrics over time
- Integrate with development workflow

**Cross-Reference**:
 See [Automation Todo](automation.md#code-quality-automation) for comprehensive automation.

### Developer Experience

- Provide clear error messages for common issues
- Document coding standards and best practices
- Create code templates and snippets
- Implement automated refactoring tools

**Cross-Reference**:
 See [Development Todo](development.md#development-workflow-improvements) for developer experience enhancements.

### Performance Integration

- [ ] Add performance-focused linting rules
- [ ] Implement performance regression detection in code review
- [ ] Create performance-aware coding guidelines
- [ ] Add performance impact analysis for code changes

**Cross-Reference**:
 See [Performance Todo](performance.md#monitoring--metrics) for performance measurement integration.

## Success Criteria

- [ ] All critical TypeScript compilation errors resolved
- [ ] Lint violations reduced to zero for high-priority rules
- [ ] Automated code quality checking integrated into development workflow
- [ ] Security-focused code patterns implemented and enforced
- [ ] Performance-aware coding standards established and followed
- [ ] Comprehensive testing coverage with quality automation
- [ ] Developer productivity enhanced through quality tooling
- [ ] Code review process enhanced with automated quality checks
