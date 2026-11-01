//region Addition repair rule
// How an accepted addition is repaired, for the repair lane's own editor and
// checker (ledger H4, class one hundred eight's open half). The dispute note
// told every downstream sheet that a detail an accepted addition names is not
// page content "in the archive's wording or any softer one"; the editor and
// checker never read it, so the editor softened an invented event and the
// checker called the softer wording fixed.

/**
 What fixes an accepted addition, and what does not.

 @example
 ```ts
 const rule = `- ${ADDITION_IS_REMOVED_NOT_SOFTENED}`;
 ```
 */
export const ADDITION_IS_REMOVED_NOT_SOFTENED: string = 'An accepted accuracy/addition issue is fixed only by '
  + 'removing the detail the ORIGINAL does not state: restating that detail in vaguer or softer words keeps the '
  + 'addition, however mild the new wording.';

//endregion Addition repair rule
