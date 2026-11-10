/**
 Recognizes the one filesystem answer that means "nothing stands here".

 The statusline reads three paths a render can legitimately find absent: a home
 with no `~/.claude/settings.json`, a session whose transcript does not exist
 yet, and a session with no remembered activity word. Those are answers, not
 faults, so they must not be reported the same way a refused permission or a
 directory where a file belongs is.

 The same check exists in `package/translation-repair/typography`, whose domain
 this command has no business importing, so it stands here on its own.

 @module
 */

/**
 Whether a caught value is the filesystem's answer that no entry stands at the
 path.

 @param error - caught value

 @returns whether the value is a Node error carrying `ENOENT`

 @example
 ```ts
 isAbsentFileError(Object.assign(new Error('gone'), { code: 'ENOENT' })); // true
 ```
 */
function isAbsentFileError(error: unknown,): boolean {
  return Error.isError(error,)
    && ('code' in error)
    && (error.code === 'ENOENT');
}

export { isAbsentFileError, };
