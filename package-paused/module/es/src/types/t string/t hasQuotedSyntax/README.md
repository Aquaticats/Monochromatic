# hasQuotedSyntax

TypeScript branded types for identifying strings with specific quoted syntax patterns.

## Overview

The `hasQuotedSyntax` type utility provides a type-safe way
to distinguish between strings that contain different types of quotes
in TypeScript code.
This is particularly useful for:

- Code analysis and parsing
- Template processing
- String manipulation with quote-aware logic
- Type-safe quote detection in development tools

## Types

### Base Type

- **`hasQuotedSyntax`**: A branded string type indicating the presence of quoted syntax

### Specific Quote Types

- **`singleQuote`**: Strings containing single quotes (`'`)
- **`doubleQuote`**: Strings containing double quotes (`"`)
- **`backtick`**: Strings containing backticks/template literals (`` ` ``)

## Usage

```ts
// Base type for any quoted syntax
type QuotedString = hasQuotedSyntax.type.$;

// Specific quote types
type SingleQuotedString = hasQuotedSyntax.singleQuote.type.$;
type DoubleQuotedString = hasQuotedSyntax.doubleQuote.type.$;
type BacktickString = hasQuotedSyntax.backtick.type.$;
```

## Examples

### Single Quote Detection

```ts
// These would be considered single-quoted strings
const singleQuoted: SingleQuotedString = "const a = 'b'" as SingleQuotedString;
const nestedSingle: SingleQuotedString =
  "const obj = { key: 'value' }" as SingleQuotedString;

// These would NOT be single-quoted strings
// const notSingle = 'const a = "b"'; // Contains double quotes instead
```

### Double Quote Detection

```ts
// These would be considered double-quoted strings
const doubleQuoted: DoubleQuotedString = '{"a": "b"}' as DoubleQuotedString;
const jsonLike: DoubleQuotedString =
  'const obj = {"key": "value"}' as DoubleQuotedString;

// These would NOT be double-quoted strings
// const notDouble = "{'a': 'b'}"; // Contains single quotes instead
```

### Backtick/Template Literal Detection

```ts
// These would be considered backtick strings
const templateLiteral: BacktickString = 'const a = `c${1}`' as BacktickString;
const multiline: BacktickString =
  'const template = `\n  Hello ${name}\n`' as BacktickString;

// These would NOT be backtick strings
// const notBacktick = "const a = 'b'"; // Contains single quotes instead
```

## Type Structure

The branded types use the following structure:

```ts
type hasQuotedSyntax = string & {
  __brand: {
    hasQuotedSyntax: true;
  };
};

type singleQuote = hasQuotedSyntax & {
  __brand: {
    quotesType: "'";
  };
};

type doubleQuote = hasQuotedSyntax & {
  __brand: {
    quotesType: '"';
  };
};

type backtick = hasQuotedSyntax & {
  __brand: {
    quotesType: '`';
  };
};
```

## Use Cases

### Code Analysis

```ts
function analyzeQuoteStyle(code: hasQuotedSyntax.type.$,): string {
  // Type-safe analysis of quoted strings in code
  if (isDoubleQuoted(code,))
    return 'Uses JSON-style double quotes';
  if (isSingleQuoted(code,))
    return 'Uses single quotes';
  if (isBacktick(code,))
    return 'Uses template literals';
  return 'Mixed or unknown quote style';
}
```

### Template Processing

```ts
function processTemplate(template: hasQuotedSyntax.backtick.type.$,): string {
  // Type guarantees this string contains backtick syntax
  return template.replace(/\$\{([^}]+)\}/g, (match, expr,) => {
    return evaluateExpression(expr,);
  },);
}
```
