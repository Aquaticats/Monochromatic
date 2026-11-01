# @monochromatic-dev/module-es

A comprehensive functional programming utility library for TypeScript,
designed to provide every thinkable utility function
with excellent type safety,
performance,
and developer experience.

> Split-warning, 2026-05-28: `@monochromatic-dev/module-es` is planned to be
> split into focused packages. Prefer existing focused packages for new imports,
> and treat new root exports as migration staging rather than stable expansion.
>
> Stale-warning, 2026-05-13: this README predates the current `types` taxonomy
> and package export map. Treat broad examples below as aspirational unless they
> use actual subpaths from `package.json`.

## Vision

This library aims to be the definitive TypeScript functional programming toolkit, providing:

- **Complete type safety** with advanced TypeScript generics and inference
- **Comprehensive utility coverage** across all data types and operations
- **Immutable-first design** with no mutational methods
- **Dual platform support** for both Node.js and browser environments
- **Performance optimization** for production applications
- **Minimal workspace runtime dependencies** for portability

## Current State vs Complete Vision

### ✅ Well-Implemented Categories

- **Boolean utilities**: Equality, logical operations, type predicates
- **Error utilities**: Comprehensive error handling and assertion functions
- **Function utilities**: Composition, currying, and functional patterns
- **Numeric utilities**: Addition, type guards, range validation, BigInt support
- **String utilities**: Validation, transformation, and formatting
- **Basic array utilities**: Type guards, range generation, basic operations

### 🟡 Partially Implemented Categories

- **Iterable utilities**: Good sync support, missing many async variants
- **Array utilities**: Basic operations exist, missing advanced algorithms
- **Promise utilities**: Basic support, missing advanced async patterns
- **Type utilities**: Some type-level programming, needs expansion

### 🔴 Missing Categories (Critical Gaps)

- **Object utilities**: Record pick, omit, and merge exist; transform and deep operations remain missing
- **Date/time utilities**: Parsing, formatting, arithmetic, timezone handling
- **Math utilities**: Statistics, interpolation, geometric operations
- **Validation utilities**: Schema validation, input sanitization
- **Collection utilities**: Set operations, Map transformations
- **Stream utilities**: Async stream processing and transformation
- **Parser utilities**: Text parsing, tokenization, grammar handling
- **Crypto utilities**: Hashing, encoding, secure random generation
- **Network utilities**: URL manipulation, query string handling
- **Geometry utilities**: Point, vector, shape operations
- **Color utilities**: Color space conversion, manipulation
- **Tree/graph utilities**: Tree traversal, graph algorithms
- **Lens/optics utilities**: Functional data access and manipulation

## Design Principles

### Immutability First

Methods that mutate the original value passed in or called on aren't implemented.
All operations return new values, preserving the original data.

### No `this` Context

The usage of "this" isn't implemented where applicable.
All functions are pure and don't rely on execution context.

### TypeScript Excellence

- Advanced generic constraints for precise type inference
- Branded types for domain-specific safety
- Comprehensive type-level programming utilities
- Zero `any` types in public APIs

### Performance Conscious

- Optimized algorithms for common operations
- Memory-efficient implementations
- Lazy evaluation where beneficial
- Performance benchmarks for critical functions

## Library Organization

### Core Categories

#### Data Type Utilities

```typescript
// Current root namespace
import { types, } from '@monochromatic-dev/module-es/ts';

// Utilities split from module-es now live in focused packages:
// @monochromatic-dev/async-iter-core
// @monochromatic-dev/function-arity-core
// @monochromatic-dev/kv-store-core
// @monochromatic-dev/kv-store-memoize
// @monochromatic-dev/observable-core
// @monochromatic-dev/pipe-core
```

#### Functional Programming Patterns

```typescript
// Function composition split into @monochromatic-dev/pipe-core.
import {
  pipe,
  pipeAsync,
  piped,
  pipedAsync,
} from '@monochromatic-dev/pipe-core';

// Function transformation
import {
  booleanfy,
  curry,
  partial,
} from '@monochromatic-dev/module-es';

// Conditional execution
import {
  when,
  whenAsync,
} from '@monochromatic-dev/module-es';
```

#### Async Programming

```typescript
// Promise utilities
import {
  nonPromiseAll,
  wait,
} from '@monochromatic-dev/module-es';

// Async iteration helpers split into @monochromatic-dev/async-iter-core.
import { mapIterableAsync, } from '@monochromatic-dev/async-iter-core';

// Concurrency control
import { deConcurrency, } from '@monochromatic-dev/module-es';
```

#### Type-Level Programming

```typescript
// Array type utilities
import type {
  ArrayFixedLength,
  Tuple,
  WithoutFirst,
} from '@monochromatic-dev/module-es';

// Numeric type utilities
import type {
  Int,
  NegativeInt,
  PositiveInt,
} from '@monochromatic-dev/module-es';

// Advanced type utilities (planned)
import type {
  DeepMerge,
  DeepOmit,
  DeepPick,
} from '@monochromatic-dev/module-es';
```

## Roadmap to Completeness

### Phase 1: Foundation (Immediate)

- **Current export map documentation**: Document the taxonomy-based `types` namespace and real subpaths
- **Object utilities expansion**: Deep object manipulation functions beyond existing record pick, omit, and merge
- **Async iterator completion**: Full async iterable ecosystem

### Phase 2: Core Expansion (Next Quarter)

- **Date/time utilities**: Comprehensive temporal operations
- **Math utilities**: Statistical and geometric functions
- **Validation framework**: Type-safe input validation

### Phase 3: Advanced Features (Future)

- **Stream processing**: Async stream utilities
- **Parser combinators**: Text parsing and grammar tools
- **Lens/optics system**: Functional data access patterns

### Phase 4: Specialized Domains (Long-term)

- **Geometry utilities**: Mathematical shape and vector operations
- **Color utilities**: Color space manipulation and conversion
- **Network utilities**: URL, HTTP, and networking helpers
- **Crypto utilities**: Cryptographic operations and secure random

## Current Implementation Status

<table>
<thead>
<tr>
<th>Category</th>
<th>Functions</th>
<th>Tests</th>
<th>Documentation</th>
<th>Completeness</th>
</tr>
</thead>
<tbody>
<tr>
<td>**Any**</td>
<td>8/10</td>
<td>60%</td>
<td>70%</td>
<td>🟡 Partial</td>
</tr>
<tr>
<td>**Array**</td>
<td>12/50+</td>
<td>80%</td>
<td>85%</td>
<td>🟡 Basic</td>
</tr>
<tr>
<td>**Boolean**</td>
<td>3/5</td>
<td>100%</td>
<td>90%</td>
<td>🟢 Good</td>
</tr>
<tr>
<td>**Error**</td>
<td>25/30</td>
<td>90%</td>
<td>85%</td>
<td>🟢 Excellent</td>
</tr>
<tr>
<td>**Function**</td>
<td>20/30</td>
<td>75%</td>
<td>70%</td>
<td>🟢 Good</td>
</tr>
<tr>
<td>**Iterable**</td>
<td>25/60+</td>
<td>65%</td>
<td>60%</td>
<td>🟡 Partial</td>
</tr>
<tr>
<td>**Numeric**</td>
<td>15/25</td>
<td>85%</td>
<td>80%</td>
<td>🟢 Good</td>
</tr>
<tr>
<td>**String**</td>
<td>20/40+</td>
<td>85%</td>
<td>75%</td>
<td>🟢 Good</td>
</tr>
<tr>
<td>**Object**</td>
<td>Partial</td>
<td>Partial</td>
<td>Partial</td>
<td>🟡 Record pick, omit, and merge exist</td>
</tr>
<tr>
<td>**Date**</td>
<td>0/25+</td>
<td>0%</td>
<td>0%</td>
<td>🔴 Missing</td>
</tr>
<tr>
<td>**Math**</td>
<td>0/20+</td>
<td>0%</td>
<td>0%</td>
<td>🔴 Missing</td>
</tr>
<tr>
<td>**Collections**</td>
<td>5/20+</td>
<td>40%</td>
<td>50%</td>
<td>🔴 Basic</td>
</tr>
</tbody>
</table>

**Target**: 500+ utility functions across 20+ categories

## Philosophy: What's Not Included

### Mutational Methods

Methods that mutate the original value passed in or called on aren't implemented.

#### Array.prototype.copyWithin

Don't see a use case for this.
Plus, it's a mutational method.

Didn't implement.

### External Library Integration

We prefer focused external libraries for some complex domains:

#### deepmerge

Use [`jsr:@rebeccastevens/deepmerge`](https://github.com/RebeccaStevens/deepmerge-ts) instead.
*Note: We may implement basic merge utilities for simple cases.*

#### match

Use [`ts-pattern`](https://github.com/gvergnaud/ts-pattern) instead.
*Note: We're considering our own async-capable pattern matching library.*

#### jsonc

Use [`jsonc.min`](https://www.npmjs.com/package/jsonc.min/) instead.

#### Observable

Use `@monochromatic-dev/observable-core` for workspace observable needs.
It provides `createObservable` and `createObservableAsync` with `getValue()` and `setValue()` methods.
For an external observable implementation, evaluate [`@therapy/observable`](https://jsr.io/@therapy/observable).

## Contributing to Completeness

This library is designed to eventually contain every useful functional programming utility. Areas for contribution:

### High-Impact Missing Categories

1. **Object utilities**: Core data manipulation functions
2. **Date/time operations**: Temporal calculations and formatting
3. **Advanced async patterns**: Concurrency control and streaming
4. **Mathematical operations**: Statistics, geometry, algorithms

### Implementation Guidelines

- **Pure functions only**: no side effects or mutations
- **Type safety first**: leverage TypeScript's type system fully
- **Performance matters**: optimize for common use cases
- **Document comprehensively**: include examples and edge cases
- **Test thoroughly**: aim for 100% coverage with edge case testing

### Quality Standards

- All functions must have comprehensive TSDoc documentation
- Type inference should be excellent without explicit type annotations
- Performance characteristics should be documented for complex operations
- Browser and Node.js compatibility maintained where applicable
- Security considerations addressed for any user-input processing

## Related TODO Files

- [**Export Fixes**](TODO.exports-fixes.md): Critical compilation errors (immediate priority)
- [**Missing Implementations**](TODO.missing-implementations.md): Functions to implement for completeness
- [**Testing Coverage**](TODO.testing.md): Test gaps and coverage improvements
- [**TSDoc Improvements**](TODO.tsdoc-improvements.md): Documentation enhancement
- [**Function Improvements**](TODO.improvements.md): Performance and API improvements
- [**Package TODO Index**](TODO.md): Complete roadmap and priority overview

For the complete vision and implementation roadmap, see [**TODO.index.md**](TODO.md).
