/**
 Comment-preserving TOML read, edit, and write utility.
 
 Wraps `toml-eslint-parser` and adds a serializer plus a free-function edit API
 over an immutable `TomlEditState`.
 
 Two fidelity modes:
 - `'splice'` preserves the original bytes for unmutated regions (default).
 - `'canonical'` rebuilds text from the AST with consistent formatting.
 
 The state is one editable document tree; every mutating function returns a
 fresh tree sharing unchanged nodes by reference. Reads, writes, and emit all
 operate on that single always-current tree, so a value queried after a
 `tomlSet` reflects the change and serialization stays consistent with reads.
 
 @example
 Round-trip a file unchanged (splice mode, byte-identical):
 ```ts
 import { parseTomlEdit, tomlStringify } from '\@monochromatic-dev/toml-edit-core';
 import { readFile } from 'node:fs/promises';
 
 const source = await readFile('mise.toml', 'utf8',);
 const edit = parseTomlEdit({ source, },);
 const text = tomlStringify({ edit, },);
 // text === source
 ```
 
 @packageDocumentation
 */

export type {
  CanonicalOptions,
  CanonicalOptionsOverride,
  TomlComment,
  TomlEditMode,
  TomlEditOptions,
  TomlEditState,
  TomlEmptyOptions,
  TomlPath,
  TomlValueInput,
  TomlWrappedInput,
} from './types.ts';

export {
  TomlEditError,
  TomlImmutableNodeError,
  TomlPathNotFoundError,
  TomlSpliceUnavailableError,
  TomlTypeError,
} from './errors.ts';

export { parseTomlEdit, } from './parse-toml-edit.ts';

export { emptyTomlEdit, } from './empty-toml-edit.ts';

export { tomlHas, } from './toml-has.ts';

export { tomlGet, } from './toml-get.ts';

export { tomlGetValue, } from './toml-get-value.ts';

export { tomlGetRaw, } from './toml-get-raw.ts';

export { tomlGetNode, } from './toml-get-node.ts';

export { tomlKeys, } from './toml-keys.ts';

export { tomlGetComments, } from './toml-get-comments.ts';

export { tomlGetCommentsBefore, } from './toml-get-comments-before.ts';

export { tomlGetCommentAfter, } from './toml-get-comment-after.ts';

export { tomlSet, } from './toml-set.ts';

export {
  tomlFloat,
  tomlInteger,
  tomlLocalDate,
  tomlLocalDateTime,
  tomlLocalTime,
} from './wrappers.ts';

export { tomlDelete, } from './toml-delete.ts';

export { tomlSetHeaderComment, } from './toml-set-header-comment.ts';

export { tomlInsertCommentBefore, } from './toml-insert-comment-before.ts';

export { tomlInsertCommentAfter, } from './toml-insert-comment-after.ts';

export { tomlStringify, } from './toml-stringify.ts';

export { normalizeTableHeaderKeyQuoting, } from './normalize-table-header-key-quoting.ts';

export {
  getStaticTOMLValue,
  parseTOML,
} from 'toml-eslint-parser';

//region Unstable fuzzing seams

/**
 Unstable internal seam exported for observability and fuzzing only.
 
 The `_`-prefixed exports below carry no compatibility promise: they exist so
 the property-based fuzz suite can exercise internal encoders and emitters
 through the built package artifact rather than sibling source imports. Their
 signatures may change without a major version bump. Do not depend on them
 from application code. See `doc/decision/toml-edit-fuzzing.md`.
 */
export { encodeKey as _encodeKey, } from './keys.ts';

/**
 Unstable path formatter exported for built-artifact tests.

 @example
 ```ts
 _formatPath({ path: ['a', 0,], }); // 'a[0]'
 ```
 */
export { formatPath as _formatPath, } from './path.ts';

/**
 Unstable strict-prefix predicate exported for built-artifact tests.

 @example
 ```ts
 _isStrictPrefix({ candidate: ['a',], path: ['a', 'b',], }); // true
 ```
 */
export { isStrictPrefix as _isStrictPrefix, } from './path-prefix.ts';

/**
 Unstable comment-gap classifier exported for built-artifact tests.

 @example
 ```ts
 _isAttachedGap('\n');
 ```
 */
export { isAttachedGap as _isAttachedGap, } from './build-comments.ts';

/**
 {@inheritDoc _encodeKey}
 */
export { jsValueToTomlText as _jsValueToTomlText, } from './values.ts';

/**
 {@inheritDoc _encodeKey}
 */
export { emitContentNode as _emitContentNode, } from './emit-value.ts';

/**
 {@inheritDoc _encodeKey}
 */
export { emitStringValue as _emitStringValue, } from './emit-value-string.ts';

/**
 {@inheritDoc _encodeKey}
 */
export { emitDocument as _emitDocument, } from './emit-document.ts';

//endregion Unstable fuzzing seams
