/**
 Reads the file change one Claude Code PreToolUse event asks for.
 
 Three tools change a file: `Write` replaces its text, `Edit` replaces one
 string, and `MultiEdit` replaces several.
 Their inputs arrive from a model, so each field is read defensively,
 and a field that is not the shape this guard needs answers "no change to judge"
 rather than a refusal.
 
 @module
 */

import type { TextEdit, } from '@monochromatic-dev/agent-harness-shared-shell-command-analyzer/ts';

//region Constants

/**
 Sentinel returned when an event carries no file change this guard can judge.
 */
const NO_FILE_CHANGE: unique symbol = Symbol('event carries no file change to judge',);

/**
 Tool name whose input is a whole file's text.
 */
const WRITE_TOOL_NAME = 'Write';

/**
 Tool name whose input is one replacement.
 */
const EDIT_TOOL_NAME = 'Edit';

/**
 Tool name whose input is several replacements.
 */
const MULTI_EDIT_TOOL_NAME = 'MultiEdit';

/**
 Field naming the file each of those tools changes.
 */
const FILE_PATH_FIELD = 'file_path';

/**
 Field holding the text a write leaves in the file.
 */
const CONTENT_FIELD = 'content';

/**
 Field holding the replacements a multi-edit call makes.
 */
const EDITS_FIELD = 'edits';

/**
 Field holding the text one replacement finds.
 */
const OLD_STRING_FIELD = 'old_string';

/**
 Field holding the text one replacement writes.
 */
const NEW_STRING_FIELD = 'new_string';

/**
 Field saying a replacement covers every occurrence.
 */
const REPLACE_ALL_FIELD = 'replace_all';

//endregion Constants

//region Types

/**
 One replacement a tool call asks for.
 */
type ChangeEdit = {
  /**
   Text the replacement finds.
   */
  readonly oldText: string;
  /**
   Text the replacement writes.
   */
  readonly newText: string;
  /**
   Whether the replacement covers every occurrence.
   */
  readonly replaceAll?: boolean;
};

/**
 One file change a tool call asks for, as the guard judges it.
 */
type FileChangeRequest = {
  /**
   Path of the file the call changes, as the call spells it.
   */
  readonly path: string;
} & (
  | {
    /**
     The call writes the whole file.
     */
    readonly kind: 'write';
    /**
     Text the call leaves in the file.
     */
    readonly content: string;
  }
    | {
    /**
     The call replaces strings inside the file.
     */
      readonly kind: 'edits';
    /**
     Replacements the call makes, in the order it reports them.
     */
      readonly edits: readonly TextEdit[];
    }
);

//endregion Types

//region Readers

/**
 Reports whether a value read from external input is one object.
 
 @param value - value read from a model-authored tool input
 
 @returns whether the value is a record whose fields can be read by name
 
 @example
 ```ts
 isReadonlyRecord({ old_string: 'a', }); // true
 ```
 */
function isReadonlyRecord(value: unknown,): value is Readonly<Record<string, unknown>> {
  return ((typeof value) === 'object')
    && (value !== null);
}

/**
 Reads one field of an external object.
 
 @param input - tool input read defensively
 
 @param field - field name to read
 
 @returns the field's value, or undefined when the object holds none
 
 @example
 ```ts
 fieldOf({ input: { file_path: '/repo/x.sh', }, field: FILE_PATH_FIELD, }); // '/repo/x.sh'
 ```
 */
function fieldOf({
  input,
  field,
}: {
  readonly input: Readonly<Record<string, unknown>>;
  readonly field: string;
},): unknown {
  return Object.hasOwn(
    input,
    field,
  )
    ? input[field]
    : undefined;
}

/**
 Reads one replacement from an external value.
 
 @param value - value read from the edit list
 
 @returns the replacement, or {@link NO_FILE_CHANGE} when the value is not one
 
 @example
 ```ts
 changeEditOf({ old_string: 'a', new_string: 'b', });
 ```
 */
function changeEditOf(value: unknown,): ChangeEdit | typeof NO_FILE_CHANGE {
  if (!isReadonlyRecord(value,))
    return NO_FILE_CHANGE;

  /**
   Candidate replacement read as an unknown record.
   */
  const record = value;
  /**
   Text the replacement finds.
   */
  const oldText = fieldOf({
    input: record,
    field: OLD_STRING_FIELD,
  },);
  /**
   Text the replacement writes.
   */
  const newText = fieldOf({
    input: record,
    field: NEW_STRING_FIELD,
  },);
  if ((((typeof oldText) !== 'string') || ((typeof newText) !== 'string')))
    return NO_FILE_CHANGE;
  return {
    oldText,
    newText,
    ...(fieldOf({
      input: record,
      field: REPLACE_ALL_FIELD,
    },) === true ? { replaceAll: true, } : {}),
  };
}

/**
 Reads every replacement a multi-edit call reports.
 
 @param input - tool input read defensively
 
 @returns the replacements, or {@link NO_FILE_CHANGE} when one is not a replacement
 
 @example
 ```ts
 multiEditChangesOf({ edits: [{ old_string: 'a', new_string: 'b', },], });
 ```
 */
function multiEditChangesOf(
  input: Readonly<Record<string, unknown>>,
): readonly TextEdit[] | typeof NO_FILE_CHANGE {
  /**
   Value of the list field the call reports.
   */
  const listed = fieldOf({
    input,
    field: EDITS_FIELD,
  },);
  if (!Array.isArray(listed,))
    return NO_FILE_CHANGE;

  /**
   Replacements read from that list.
   */
  const edits: TextEdit[] = [];
  for (const value of listed) {
    /**
     One replacement read from the list.
     */
    const edit = changeEditOf(value,);
    if ((typeof edit) === 'symbol')
      return NO_FILE_CHANGE;
    edits.push(edit,);
  }
  return edits;
}

/**
 Reads the file change one tool call asks for.
 
 @param toolName - name Claude Code reported for the tool
 
 @param input - tool input read defensively
 
 @returns the change to judge, or {@link NO_FILE_CHANGE} when the call changes no file
 
 @example
 ```ts
 fileChangeOf({ toolName: 'Edit', input: { file_path: '/repo/clean.sh', old_string: 'a', new_string: 'b', } });
 ```
 */
function fileChangeOf({
  toolName,
  input,
}: {
  readonly toolName: string;
  readonly input: Readonly<Record<string, unknown>>;
},): FileChangeRequest | typeof NO_FILE_CHANGE {
  /**
   Path of the file the call changes.
   */
  const path = fieldOf({
    input,
    field: FILE_PATH_FIELD,
  },);
  if (((typeof path) !== 'string'))
    return NO_FILE_CHANGE;

  if (toolName === WRITE_TOOL_NAME) {
    /**
     Text the call leaves in the file.
     */
    const content = fieldOf({
      input,
      field: CONTENT_FIELD,
    },);
    if (((typeof content) !== 'string'))
      return NO_FILE_CHANGE;
    return {
      kind: 'write',
      path,
      content,
    };
  }

  if (toolName === EDIT_TOOL_NAME) {
    /**
     The call's single replacement.
     */
    const edit = changeEditOf(input,);
    if ((typeof edit) === 'symbol')
      return NO_FILE_CHANGE;
    return {
      kind: 'edits',
      path,
      edits: [edit,],
    };
  }

  if (toolName === MULTI_EDIT_TOOL_NAME) {
    /**
     The call's replacements, in the order it reports them.
     */
    const edits = multiEditChangesOf(input,);
    if ((typeof edits) === 'symbol')
      return NO_FILE_CHANGE;
    return {
      kind: 'edits',
      path,
      edits,
    };
  }

  return NO_FILE_CHANGE;
}

//endregion Readers

export {
  fileChangeOf,
  isReadonlyRecord,
  NO_FILE_CHANGE,
};
export type { ChangeEdit, };
export type { FileChangeRequest, };
