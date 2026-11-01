# cli-fy

Call any ESM export from the command line.

## Usage

```sh
cli-fy <specifier> <export> [args...]
```

Dynamically imports `<specifier>`,
 accesses its named `<export>`,
calls it with the provided arguments (if it is a function),
 and prints the result.

## Examples

```sh
# Call a function with arguments
cli-fy lodash add 1 1
# => 2

# Use node built-in modules
cli-fy node:path join /tmp test
# => /tmp/test

# Access a non-function export (no args)
cli-fy node:path sep
# => /

# Scoped packages
cli-fy @scope/pkg myFn arg1 arg2

# Subpath exports -- use "default" for default export
cli-fy lodash-es/add default 1 1
```

## Argument coercion

Arguments are coerced via `JSON.parse` with a fallback to raw string:

- `42` becomes the number `42`
- `true` becomes the boolean `true`
- `null` becomes `null`
- `'[1,2]'` becomes the array `[1, 2]`
- `hello` stays the string `"hello"`

## Resolution order

Specifiers are resolved in this order:

1. CWD `node_modules`
2. Monorepo root `node_modules` (walks up from CWD looking for a `package.json` with `workspaces`)
3. Global `node_modules` locations that contain the marker file currently used by the resolver

## Error handling

- Non-existent export:
   lists all available exports from the module
- Non-function export with arguments:
   throws an error explaining the type mismatch
- Unresolvable specifier:
   lists all directories that were searched
