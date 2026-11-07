/**
 Reads parsed JSON and caught failures without asserting a type: `JSON.parse` returns `any` and a caught value is
 `unknown`, so every field is checked for the type the caller needs.

 @module
 */

/**
 Parses JSON text and keeps the fields whose value is text.

 @param text - JSON text, normally an object

 @returns the text fields by name; empty for JSON that is not an object

 @throws SyntaxError when the text is not JSON

 @example
 ```ts
 import { parseTextFields, } from './json-text-fields.ts';

 const fields = parseTextFields('{"uuid":"u-a","size":3}',);
 // { uuid: 'u-a' }
 ```
 */
export function parseTextFields(text: string,): Readonly<Record<string, string>> {
  /**
   The parsed value, of any shape.
   */
  const parsed: unknown = JSON.parse(text,);
  if (((typeof parsed) !== 'object') || (parsed === null)) {
    return {};
  }
  return Object.fromEntries(Object.keys(parsed,)
    .flatMap(function textEntry(key,): readonly (readonly [
      string,
      string,
    ])[] {
      /**
       The field's value.
       */
      const value: unknown = Reflect.get(
        parsed,
        key,
      );
      return (typeof value) === 'string' ? [[
        key,
        value,
      ],] : [];
    },),);
}

/**
 What a failed child process carries, read from the caught value.
 */
export type ChildFailure = {

  /**
   Exit status (a number) or system error code (text); the empty text when the value has none.
   */
  readonly code: number | string;
  readonly message: string;
  readonly stdout: string;
  readonly stderr: string;
};

/**
 Keeps a value that is text.

 @param value - any value

 @returns the text, or the empty text for any other value
 */
function textOf(value: unknown,): string {
  return (typeof value) === 'string' ? value : '';
}

/**
 Keeps a value that is an exit status or a system error code.

 @param value - any value

 @returns the number or the text, or the empty text for any other value
 */
function statusOf(value: unknown,): number | string {
  if ((typeof value) === 'number') {
    return value;
  }
  return (typeof value) === 'string' ? value : '';
}

/**
 Reads the fields of a failed child process out of a caught value.

 @param error - the caught value

 @returns the failure; every field the value lacks is the empty text

 @example
 ```ts
 import { childFailureOf, } from './json-text-fields.ts';

 const failure = childFailureOf(new Error('boom',),);
 ```
 */
export function childFailureOf(error: unknown,): ChildFailure {
  if (((typeof error) !== 'object') || (error === null)) {
    return {
      code: '',
      message: String(error,),
      stdout: '',
      stderr: '',
    };
  }
  /**
   The caught value, known to be an object here.
   */
  const source: object = error;
  /**
   Reads one field of the caught value as unknown.

   @param key - the field name

   @returns the value
   */
  function fieldOf(key: string,): unknown {
    return Reflect.get(
      source,
      key,
    );
  }
  return {
    code: statusOf(fieldOf('code',),),
    message: textOf(fieldOf('message',),),
    stdout: textOf(fieldOf('stdout',),),
    stderr: textOf(fieldOf('stderr',),),
  };
}
