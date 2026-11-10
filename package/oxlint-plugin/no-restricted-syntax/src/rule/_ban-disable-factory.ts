import type {
  Comment,
  Context,
  CreateOnceRule,
  VisitorWithHooks,
} from '@oxlint/plugins';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

//region Directive vocabulary
// The disable-directive prefix and every keyword spelling oxlint reads, longest
// first so `-next-line` is never cut down to `-line`.

/**
 Text prefix common to every oxlint disable directive.
 */
const DISABLE_DIRECTIVE_PREFIX = 'oxlint-disable';

/**
 Every disable directive spelling oxlint reads, longest first so the
 `-next-line` form is not cut down to the `-line` form.
 */
const DISABLE_DIRECTIVE_KEYWORDS = [
  'oxlint-disable-next-line',
  'oxlint-disable-line',
  DISABLE_DIRECTIVE_PREFIX,
] as const;

//endregion Directive vocabulary

//region Reading a directive
// Cutting an entry at its first whitespace, so only the rule id survives, and
// listing the ids after a keyword in the comma-separated tail.

/**
 Cuts text at its first whitespace character, so a directive entry keeps only
 its rule id and drops any trailing description or reason.

 @param text - already trimmed directive entry text

 @returns leading text before the first whitespace character, possibly empty

 @example
 ```ts
 firstWord({ text: 'no-switch -- reason' }); // 'no-switch'
 ```
 */
function firstWord({ text, }: { readonly text: string; },): string {
  for (let index = 0; index < text.length; index += 1) {
    /**
     Character at the scanned position.
     */
    const character = text.charAt(index,);
    if (character.trim() === '')
      return text.slice(
        0,
        index,
      );
  }
  return text;
}

/**
 Reads the rule ids named by the comma-separated list after a directive
 keyword.

 @param tail - directive text after its keyword

 @returns rule ids, each cut at its first whitespace character

 @example
 ```ts
 ruleIdsInDirectiveTail({ tail: ' no-unused-vars, no-switch -- reason' }); // ['no-unused-vars', 'no-switch']
 ```
 */
function ruleIdsInDirectiveTail({ tail, }: { readonly tail: string; },): readonly string[] {
  return tail
    .split(',',)
    .map(function entryRuleId(entry,): string {
      return firstWord({ text: entry.trim(), },);
    },)
    .filter(function nonEmpty(ruleId,): boolean {
      return ruleId.length > 0;
    },);
}

//endregion Reading a directive

//region Matching the banned rule
// Whether an entry names the banned rule under any plugin prefix, and the ids a
// comment disables as a whole.

/**
 Checks whether a directive entry names the banned rule, under any plugin
 prefix, so a bare banned id such as `no-useless-return` also matches
 `eslint/no-useless-return` while a longer id stays out.

 @param entryRuleId - one rule id read out of a directive

 @param ruleId - full rule id the ban rule targets

 @returns whether the directive names the banned rule

 @example
 ```ts
 namesBannedRule({ entryRuleId: 'eslint/no-useless-return', ruleId: 'no-useless-return' }); // true
 namesBannedRule({ entryRuleId: 'no-switch-extra', ruleId: 'no-switch' }); // false
 ```
 */
function namesBannedRule(
  {
    entryRuleId,
    ruleId,
  }: {
    readonly entryRuleId: string;
    readonly ruleId: string;
  },
): boolean {
  return (entryRuleId === ruleId) || entryRuleId.endsWith(`/${ruleId}`,);
}

/**
 Reads the rule ids a comment disables, which is empty for a comment that
 carries no directive.

 Oxlint reads a directive only when it begins the comment text, so prose that
 mentions `oxlint-disable`, a `*`-prefixed block comment line, and a directive
 naming a longer rule id all disable nothing.

 @param value - comment text without comment delimiters

 @returns rule ids the comment disables, each a whole id

 @example
 ```ts
 disabledRuleIds({ value: ' oxlint-disable-next-line no-switch -- reason' }); // ['no-switch']
 disabledRuleIds({ value: ' Fixture: oxlint-disable for no-switch' }); // []
 ```
 */
function disabledRuleIds({ value, }: { readonly value: string; },): readonly string[] {
  /**
   Comment text with its leading whitespace removed.
   */
  const text = value.trimStart();
  for (const keyword of DISABLE_DIRECTIVE_KEYWORDS) {
    if (!text.startsWith(keyword,))
      continue;
    /**
     Directive text after its keyword.
     */
    const tail = text.slice(keyword.length,);
    /**
     First character of the directive tail, empty when the tail is empty.
     */
    const firstCharacter = tail.charAt(0,)
      .trim();
    if ((tail.length > 0) && (firstCharacter.length > 0))
      continue;
    return ruleIdsInDirectiveTail({ tail, },);
  }
  return [];
}

//endregion Matching the banned rule

//region The factory
// The exported factory, which builds one ban rule per banned rule id.

/**
 Creates an oxlint rule that bans inline disable directives targeting a
 specific rule ID. Config-level overrides in `oxlint.config.ts` are unaffected;
 only inline suppression comments are flagged.

 @param ruleId - full rule ID to ban (e.g. `tsdoc/require-tsdoc`); a
 directive naming it behind another plugin prefix also matches

 @param description - human-readable description for the rule's docs

 @param message - error message shown when the banned suppression is found

 @returns oxlint `CreateOnceRule` that scans all comments for a directive
 naming the banned rule via {@link disabledRuleIds}

 @example
 ```ts
 export const noDisableNoSwitch = banDisableRule({
   ruleId: 'no-restricted-syntax/no-switch',
   description: 'Disallow disabling no-switch.',
   message: 'Disabling no-switch is not allowed.',
 });
 ```
 */
export function banDisableRule({
  ruleId,
  description,
  message,
}: {
  readonly ruleId: string;
  readonly description: string;
  readonly message: string;
},): CreateOnceRule {
  return {
    meta: {
      type: 'problem',
      docs: {
        description,
        recommended: true,
      },
      messages: {
        forbidden: message,
      },
    },
    /**
     Handles foreign Oxlint callback.

     @param context - Foreign rule context receiving diagnostics.

     @mutates context - Emits Oxlint diagnostics through foreign rule context.

     @example
     ```ts
     createOnce(context);
     ```
     */
    createOnce(context: ForeignBorrowed<Context>,): VisitorWithHooks {
      return {
        Program(): void {
          context.sourceCode
            .getAllComments()
            .forEach(function checkComment(comment: ForeignBorrowed<Comment>,) {
              if (!disabledRuleIds({ value: comment.value, },)
                .some(function namesRule(entryRuleId,): boolean {
                  return namesBannedRule({
                    entryRuleId,
                    ruleId,
                  },);
                },))
                return;
              context.report({
                node: comment,
                messageId: 'forbidden',
              },);
            },);
        },
      };
    },
  };
}

//endregion The factory
