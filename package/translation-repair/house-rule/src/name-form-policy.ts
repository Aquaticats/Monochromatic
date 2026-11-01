/**
 Limits the declared-name exemption to references, not language forms being discussed.
 Mio12's judges used the declared name to replace a different questioned character.
 The exception belongs inside faithfulness, where that replacement was defended.

 @example
 ```ts
 const criterion = `Faithfulness: ${DECLARED_NAME_REFERENCE_EXEMPTION}`;
 ```
 */
export const DECLARED_NAME_REFERENCE_EXEMPTION =
  'When a name is used TO REFER TO a person, a declared name or handle is not an addition even if the ORIGINAL only says "she". This does not authorize replacing a word, character, spelling, or sound that the ORIGINAL is talking ABOUT. Preserve the actual forms being questioned, contrasted, or denied: normalizing them to the declared name can change what the sentence means.';

/**
 Distinguishes referring to an entity from mentioning its written or spoken form.
 Shared inside existing name rules rather than appended as a competing exception.
 Keeps ordinary source-language prose outside the literal-form carve-out.

 @example
 ```ts
 const nameCriterion = NAME_FORM_SCOPE_RULE;
 ```
 */
export const NAME_FORM_SCOPE_RULE =
  'Use declared names, handles and established archive terminology when referring to their entities. Distinguish those references from mentioning a particular word, character, spelling, or sound: retain the form under discussion, even when its script differs from the declared name, and translate the surrounding prose normally. This form-preservation requirement governs such mentions instead of ordinary name normalization; it does not license leaving an entire source-language quotation untranslated.';

/**
 Keeps established proper names distinct from generic terms needing explanation.
 The inline-gloss rule still covers ordinary terminology and explicit definitions.

 @example
 ```ts
 const vocabularyRule = `${ARCHIVE_NAME_GLOSS_SCOPE} A generic term with no English equivalent ...`;
 ```
 */
export const ARCHIVE_NAME_GLOSS_SCOPE =
  'When referring to a person, group or place, an established translation-side name is not a generic community term. Preserve that name unless a declared authoritative name supplies another spelling. Do not require a source-script copy, fresh literal translation or explanation of the name’s component words merely because its spelling is unfamiliar. This does not exempt ordinary terminology or an explicit source definition from translation.';
