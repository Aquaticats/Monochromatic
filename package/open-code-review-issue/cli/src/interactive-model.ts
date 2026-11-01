/**
 Interactive prompt and selection contracts.
 
 @module
 */

import type { InputPosition, } from './model.ts';
import type { RenderedIssue, } from './issue-model.ts';

/**
 TTY streams passed explicitly to Inquirer.
 */
export type PromptStreams = {
  readonly input: NodeJS.ReadableStream;
  readonly output: NodeJS.WritableStream;
};

/**
 Square checkbox theme subset used by Inquirer checkbox.
 */
export type SquareCheckboxTheme = {
  readonly icon: {
    readonly checked: string;
    readonly unchecked: string;
    readonly disabledChecked: string;
    readonly disabledUnchecked: string;
  };
  readonly style: {
    readonly message: (value: string) => string;
    readonly highlight: (value: string) => string;
  };
};

/**
 Interactive selection approved after per-security confirmation.
 */
export type InteractiveSelection = {
  readonly issues: readonly RenderedIssue[];
  readonly withheldPositions: readonly InputPosition[];
};
