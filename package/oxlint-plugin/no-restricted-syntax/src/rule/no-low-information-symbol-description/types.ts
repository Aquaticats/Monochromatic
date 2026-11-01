/**
 Structural class of a single character: digit, cased-letter upper or lower,
 or separator (everything else, including punctuation and whitespace).
 */
export type CharKind = 'digit' | 'upper' | 'lower' | 'separator';

/**
 Result of namespace analysis: whether a leading `prefix/` or `prefix:` exists,
 its words, and the remaining tail words.
 */
export type NamespaceParts = {
  /**
   Whether a space-free namespace prefix preceded a `/` or `:` delimiter.
   */
  readonly isNamespaced: boolean;
  /**
   Words of namespace prefix; empty when not namespaced.
   */
  readonly namespaceWords: readonly string[];
  /**
   Words after delimiter, or whole description when not namespaced.
   */
  readonly tailWords: readonly string[];
};

/**
 Message id for each failure branch; the visitor reports the matching one.
 */
export type FailureMessageId =
  | 'tooFewWords'
    | 'allUppercase'
    | 'bareCamelIdentifier'
    | 'repeatedMeaningfulWord'
    | 'shortNamespacedTail'
    | 'startsWithNoWithoutMarker'
    | 'startsWithNotWithoutMarker'
    | 'shortPhraseLacksSpecificityMarker';

/**
 Verdict for a static description: a pass, or a fail carrying its message id.
 */
export type SymbolDescriptionVerdict =
  | { readonly status: 'pass'; }
    | {
      readonly status: 'fail';
      readonly messageId: FailureMessageId;
    };
