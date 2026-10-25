/**
 The clauses a judge must be able to read in the sentence that tells it what a decline does over wording that cannot
 ship, written as the meaning each carries rather than as a copy of the constant's words (re-check 3, N1 and N7).

 A copied substring fails on any rewording, including one that keeps every meaning. Each clause here is a pattern that
 any wording of the meaning satisfies, so a case reading them fails when a clause is lost or turned round and not when
 a clause is put differently.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

//region Decline clauses
// One entry per thing the sentence must say, in the order a judge meets them.

/**
 What a judge must be told, each as a pattern over the sentence.

 THE FIRST CLAUSE IS WHAT KEEPS A JUDGE FROM READING ITS DECLINE AS THE WAY TO THE STAY: declining alone does not keep
 the wording in place. The second says the same decline names no candidate, so it adds to no candidate's count. The
 third and fourth are the two outcomes, separated by `;` in the sentence so each pattern reads one clause: none named by
 two or more judges leaves the wording in place and the page reports it, and one or more named by two or more judges
 ships the preferred of those by a fixed order. The last is that fixed order, which is the behaviour of
 `rankedByPreference` in `translate-stage/src/translate-runoff-tie.ts`.

 @example
 ```ts
 const unmet = DECLINE_CLAUSES.filter(function missing({ pattern, },) {
   return !pattern.test(sheetText,);
 },);
 ```
 */
export const DECLINE_CLAUSES: readonly {
  readonly meaning: string;
  readonly pattern: RegExp;
}[] = [
  {
    meaning: 'declining does not by itself keep the wording in place',
    pattern: /declin(?:e|ing)\s+(?:does|will)\s+not\s+(?:by\s+itself|alone|on\s+its\s+own)\s+keep\s+the\s+wording/i,
  },
  {
    meaning: 'a judge\'s own decline names no candidate',
    pattern: /(?:your\s+own|a\s+judge's\s+own)\s+decline\s+names\s+no\s+candidate/i,
  },
  {
    meaning: 'no candidate named by two or more judges: the wording in place stays and the page reports it',
    pattern: /no\s+candidate\s+(?:is|was)\s+named\s+by\s+two\s+or\s+more\s+judges[^;]*?\bstays?\b[^;]*?\breports?\b/i,
  },
  {
    meaning: 'one or more candidates named by two or more judges: the preferred of those ships',
    pattern: /named\s+by\s+two\s+or\s+more\s+judges[^;]*?\bpreferred\b[^;]*?\bships?\b/i,
  },
  {
    meaning: 'the preferred ships by a fixed order, the repair lane first, then the translate lane, then the slate',
    pattern: /fixed\s+order[^;]*?repair\s+lane[^;]*?then\s+the\s+translate\s+lane[^;]*?then\s+the\s+first\s+on\s+the\s+slate/i,
  },
];

/**
 The meanings of the clauses a text does not carry, in sentence order.

 @param text - sentence or whole sheet a judge reads

 @returns The meanings that no part of the text satisfies, empty when every clause is read

 @example
 ```ts
 const unmet = unmetDeclineClauses(sheetText,);
 ```
 */
export function unmetDeclineClauses(text: string,): readonly string[] {
  return DECLINE_CLAUSES
    .filter(function isMissing({ pattern, },): boolean {
      return !pattern.test(text,);
    },)
    .map(function toMeaning({ meaning, },): string {
      return meaning;
    },);
}

//endregion Decline clauses
