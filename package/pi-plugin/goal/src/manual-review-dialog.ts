/**
 Mandatory combined TUI dialog for reviewer-exhaustion fallback.
 
 @module
 */

import type { ExtensionContext, } from '@earendil-works/pi-coding-agent';
import {
  Editor,
  type EditorTheme,
  Key,
  matchesKey,
  type TUI,
} from '@earendil-works/pi-tui';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

/**
 Explicit manual approval or rejection with optional reason.
 */
type ManualGoalReviewDecision =
  | { readonly action: 'accept'; }
    | {
      readonly action: 'reject';
      readonly reason: string;
    };

/**
 Injectable manual-review dialog capability.
 */
type ManualGoalReviewPrompt = (
  input: {
    readonly context: ForeignBorrowed<ExtensionContext>;
    readonly diagnostic: string;
  },
) => Promise<ManualGoalReviewDecision>;

/**
 Request manual-review component render after local state change.
 
 @param tui - Pi TUI render controller
 
 @mutates tui - tui.requestRender schedules component repaint
 
 @example
 ```ts
 requestManualReviewRender(tui);
 ```
 */
function requestManualReviewRender(tui: ForeignBorrowed<TUI>,): void {
  tui.requestRender();
}

/**
 Show non-cancellable combined accept or reject-with-reason dialog.
 
 Escape returns from reason editing to the decision list and never settles dialog.
 
 @param context - TUI extension context
 
 @param diagnostic - normalized model-review failure summary
 
 @returns explicit user decision
 
 @mutates context - context.ui.custom temporarily owns TUI input and rendering
 
 @example
 ```ts
 await promptManualGoalReview({ context, diagnostic });
 ```
 */
async function promptManualGoalReview(
  {
    context,
    diagnostic,
  }: {
    readonly context: ForeignBorrowed<ExtensionContext>;
    readonly diagnostic: string;
  },
): Promise<ManualGoalReviewDecision> {
  return await context.ui
    .custom<ManualGoalReviewDecision>(
    function manualReviewComponent(
      tui: ForeignBorrowed<TUI>,
      theme,
      _keybindings,
      done,
    ) {
      /**
       Dialog-local selection and rejection editor state.
       */
      const state = {
        optionIndex: 0,
        editingReason: false,
      };
      /**
       Theme adapter for optional rejection-reason editor.
       */
      const editorTheme: EditorTheme = {
        borderColor(text,) {
          return theme.fg(
            'accent',
            text,
          );
        },
        selectList: {
          selectedPrefix(text,) {
            return theme.fg(
              'accent',
              text,
            );
          },
          selectedText(text,) {
            return theme.fg(
              'accent',
              text,
            );
          },
          description(text,) {
            return theme.fg(
              'muted',
              text,
            );
          },
          scrollInfo(text,) {
            return theme.fg(
              'dim',
              text,
            );
          },
          noMatch(text,) {
            return theme.fg(
              'warning',
              text,
            );
          },
        },
      };
      /**
       Inline optional rejection-reason editor.
       */
      const editor = new Editor(
        tui,
        editorTheme,
      );
      editor.onSubmit = function submitRejection(reason,): void {
        done({
          action: 'reject',
          reason: reason.trim(),
        },);
      };
      return {
        render(width: number,) {
          /**
           Fixed dialog header and normalized model failures.
           */
          const header = [
            theme.fg(
              'accent',
              theme.bold('Independent review unavailable',),
            ),
            theme.fg(
              'warning',
              diagnostic,
            ),
            '',
          ];
          if (state.editingReason) {
            return [
              ...header,
              theme.fg(
                'accent',
                'Reject reason (optional):',
              ),
              ...editor.render(width,),
              theme.fg(
                'dim',
                'enter reject • esc return to choices',
              ),
            ];
          }
          /**
           Two semantic decisions with current selection styling.
           */
          const options = [
            'Accept',
            'Reject [optional reason]',
          ].map(function renderOption(
            label,
            index,
          ) {
            /**
             Selection marker for current option row.
             */
            const prefix = index === state.optionIndex ? '› ' : '  ';
            return index === state.optionIndex
              ? theme.fg(
                'accent',
                `${prefix}${label}`,
              )
              : `${prefix}${label}`;
          },);
          return [
            ...header,
            ...options,
            '',
            theme.fg(
              'dim',
              '↑↓ choose • enter activate • esc ignored',
            ),
          ];
        },
        invalidate() {
          editor.invalidate();
        },
        handleInput(data: string,) {
          if (state.editingReason) {
            if (matchesKey(
              data,
              Key.escape,
            )) {
              state.editingReason = false;
              editor.setText('',);
              requestManualReviewRender(tui,);
              return;
            }
            editor.handleInput(data,);
            requestManualReviewRender(tui,);
            return;
          }
          if (matchesKey(
            data,
            Key.up,
          )) {
            state.optionIndex = 0;
            requestManualReviewRender(tui,);
            return;
          }
          if (matchesKey(
            data,
            Key.down,
          )) {
            state.optionIndex = 1;
            requestManualReviewRender(tui,);
            return;
          }
          if (!matchesKey(
            data,
            Key.enter,
          ))
            return;
          if (state.optionIndex === 0) {
            done({ action: 'accept', },);
            return;
          }
          state.editingReason = true;
          editor.setText('',);
          requestManualReviewRender(tui,);
        },
      };
    },
    );
}

export { promptManualGoalReview, };
export type {
  ManualGoalReviewDecision,
  ManualGoalReviewPrompt,
};
