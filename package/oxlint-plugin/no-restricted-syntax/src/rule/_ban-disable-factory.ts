import type {
  Comment,
  Context,
  CreateOnceRule,
  VisitorWithHooks,
} from '@oxlint/plugins';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

/**
 Text prefix common to every oxlint disable directive.
 */
const DISABLE_DIRECTIVE_PREFIX = 'oxlint-disable';

/**
 Checks whether a comment contains an oxlint disable directive.
 
 `oxlint-disable` and `oxlint-disable-next-line` both start with the
 same {@link DISABLE_DIRECTIVE_PREFIX}. A string includes check is
 sufficient because the follow-up rule-id check still gates reports to
 real target suppressions.
 
 @param value - comment text without comment delimiters
 
 @returns whether comment text contains an oxlint disable directive
 
 @example
 ```ts
 hasOxlintDisableDirective({ value: ' oxlint-disable-next-line no-switch' });
 ```
 */
function hasOxlintDisableDirective({ value, }: { readonly value: string; },): boolean {
  return value.includes(DISABLE_DIRECTIVE_PREFIX,);
}

/**
 Creates an oxlint rule that bans inline disable directives
 targeting a specific rule ID. Config-level overrides in `oxlint.config.ts`
 are unaffected; only inline suppression comments are flagged.
 
 @param ruleId - full rule ID to ban (e.g. `tsdoc/require-tsdoc`)
 
 @param description - human-readable description for the rule's docs
 
 @param message - error message shown when the banned suppression is found
 
 @returns oxlint `CreateOnceRule` that scans all comments for the banned
 suppression via {@link hasOxlintDisableDirective}
 
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
              if (hasOxlintDisableDirective({ value: comment.value, },)
                && comment
                  .value
                  .includes(ruleId,))
                {
                context.report({
                  node: comment,
                  messageId: 'forbidden',
                },);
              }
            },);
        },
      };
    },
  };
}
