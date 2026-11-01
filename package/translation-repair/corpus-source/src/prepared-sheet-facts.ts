import type { PreparedDocumentPair, } from './prepared-document-pair.ts';

//region Prepared sheet facts
// WHAT A PREPARED PAGE TELLS EVERY JUDGING SHEET AND EVERY CACHE KEY ABOUT THE
// PAGE AS A WHOLE, gathered in one place: the declared names, the pages the
// original cites, whether the original's entry records a death, and the names the
// translation declares for its subject. Four call sites carried the same four
// conditional spreads, and a fifth fact added to one and not to the others would
// have reached some sheets and not their keys.

/**
 The page-level facts a stage input carries, each present only when the page has it.

 @example
 ```ts
 const facts: PreparedSheetFacts = { lifeEnded: false, };
 ```
 */
export type PreparedSheetFacts = {
  /**
   Names and handles both documents declare, absent when neither declares any.
   */
  readonly identityContext?: string;

  /**
   Whether the original's entry records a death, present only when it does.
   */
  readonly lifeEnded?: boolean;

  /**
   Names the translation declares for its subject, its display name and its
   aliases, present only with a recorded death and a declared name or alias.
   */
  readonly subjectNames?: readonly string[];

  /**
   What the pages the original cites say, absent when it cites none.
   */
  readonly referenceContext?: string;
};

/**
 Reads the page-level facts off a preparation, leaving out each the page lacks.

 @param prepared - preparation of the page

 @returns The facts, ready to spread into a stage input or a run shape

 @example
 ```ts
 const input = { ...preparedSheetFactsOf({ prepared, },), lineStructured: false, };
 ```
 */
export function preparedSheetFactsOf(
  { prepared, }: { readonly prepared: PreparedDocumentPair; },
): PreparedSheetFacts {
  return {
    ...((prepared.identityContext === undefined) ? {} : { identityContext: prepared.identityContext, }),
    ...((prepared.lifeEnded === true) ? { lifeEnded: true, } : {}),
    ...((prepared.subjectNames === undefined) ? {} : { subjectNames: prepared.subjectNames, }),
    ...((prepared.referenceContext === undefined) ? {} : { referenceContext: prepared.referenceContext, }),
  };
}

//endregion Prepared sheet facts
