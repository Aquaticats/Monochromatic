/**
 Which session file a run may rewrite once its Pi is gone.

 A run that was given a session to resume hands Pi a masked copy and leaves the file it was given exactly as it is: that file is
 another run's, and the other run may still be appending to it. So the file this run may rewrite is decided in one place, and only two
 files can be it: the copy this run made, and the file Pi created for this run. The session the run was given to resume is refused
 here, by name, so no fallback of any kind can reach it, and the refusal is a value a summary can state.

 @packageDocumentation
 */

//region The file this run may rewrite

/**
 Every file this module has decided a run may rewrite.

 A file a run may rewrite is not a bare path, and this set is what says so: the one function below is the only thing that puts a file
 in it, and the masker that rewrites a session refuses a file that is not in it. A path a caller made up is a plain object, and a plain
 object is never in this set, so the session a caller named cannot be rewritten even by code that names it.
 */
const REWRITABLE = new WeakSet<SessionFileThisRunMayRewrite>();

/**
 A session file this run may rewrite: the copy it made, or the file its own Pi created.

 Only {@link sessionFileToMaskAtRest} hands one out, so a function that rewrites a session at rest can only be given a file this module
 decided on, and every other object is refused.
 */
export type SessionFileThisRunMayRewrite = { readonly path: string };

/**
 Says whether a file is one this module decided a run may rewrite.

 @param file - the file to ask about

 @returns whether this module handed it out

 @example
 ```ts
 const decided = sessionFileToMaskAtRest({ copyThisRunMade: '/run/resumed-session.jsonl', },);
 const mayRewrite = ('file' in decided) && isAFileThisRunMayRewrite({ file: decided.file, }); // true
 ```
 */
export function isAFileThisRunMayRewrite({ file, }: { readonly file: SessionFileThisRunMayRewrite },): boolean {
  return REWRITABLE.has(file,);
}

/**
 Names a file as one this run may rewrite, which only this module does.

 @param path - the file

 @returns the file, as one a run may rewrite
 */
function namedByThisRun({ path, }: { readonly path: string },): SessionFileThisRunMayRewrite {
  /**
   The file, frozen so no caller can point it elsewhere after it was decided.
   */
  const file = Object.freeze({ path, },);
  REWRITABLE.add(file,);
  return file;
}

/**
 What decided which session file, if any, this run may rewrite.
 */
export type SessionFileToMaskAtRest =
  | { readonly file: SessionFileThisRunMayRewrite }
  // Neither Pi nor the run named a file: a run that keeps no session has none, and one that keeps one has an ending to state.
    | { readonly absent: true }
  // The only file named is the one this run was given to resume, which belongs to another run and is left as it is.
    | { readonly refused: string };

/**
 Decides which session file this run may rewrite, refusing the session it was given to resume.

 Pi's own statistics name the file Pi kept. A run that resumed a session also knows the copy it made, and hands Pi that copy, so the
 copy is what Pi's own answers name; when Pi names nothing, the copy is still the run's own file and is the answer. The session the
 run was given is never an answer, whether Pi names it or a caller falls back to it.

 @param reported - the session file Pi's own statistics or state named

 @param copyThisRunMade - the masked copy this run made of a session it was given to resume

 @param sessionTheCallerNamed - the session this run was given to resume

 @returns the file this run may rewrite, that there is none, or why the only file named is not this run's

 @example
 ```ts
 const session = sessionFileToMaskAtRest({ copyThisRunMade: '/run/resumed-session.jsonl', },);
 ```
 */
export function sessionFileToMaskAtRest({
  reported,
  copyThisRunMade,
  sessionTheCallerNamed,
}: {
  readonly reported?: string;
  readonly copyThisRunMade?: string;
  readonly sessionTheCallerNamed?: string;
},): SessionFileToMaskAtRest {
  // A copy made here is what Pi was handed, so the session the caller named is another run's file from that moment on, and no answer
  // from Pi, and no fallback of any kind, may name it as this run's to rewrite.
  if ((copyThisRunMade !== undefined) && (reported === sessionTheCallerNamed)) {
    return {
      refused: `Pi named the session this run was given to resume, ${reported}, which belongs to another run and holds its entries: `
        + 'it is left exactly as it is, so no value of it was masked at rest',
    };
  }
  if (reported !== undefined) {
    return { file: namedByThisRun({ path: reported, },), };
  }
  // No answer from Pi: the copy is this run's own file, made here from what the run was given, and the run was given it to write.
  if (copyThisRunMade !== undefined) {
    return { file: namedByThisRun({ path: copyThisRunMade, },), };
  }
  return { absent: true, };
}

/**
 Reads the session file Pi named, in its own statistics or in the state it answered when it started.

 Pi's statistics are the last thing it says, so they win; the state it answered when it started names the same file.

 @param sessionStats - what Pi answered when it was asked for its statistics, of any shape

 @param state - the session file Pi named in its own state, when it named one

 @returns the one path Pi named, or nothing

 @example
 ```ts
 const reported = reportedSessionFile({ sessionStats: { sessionFile: '/runs/a-1/s.jsonl', }, },);
 ```
 */
export function reportedSessionFile({
  sessionStats,
  state,
}: {
  readonly sessionStats: unknown;
  readonly state?: string;
},): { readonly reported?: string } {
  /**
   The path Pi put in its statistics, of any shape.
   */
  const reported: unknown = ((typeof sessionStats) === 'object') && (sessionStats !== null)
    ? Reflect.get(
      sessionStats,
      'sessionFile',
    )
    : undefined;
  if ((typeof reported) === 'string') {
    return { reported, };
  }
  return state === undefined ? {} : { reported: state, };
}

//endregion The file this run may rewrite
