/**
 Oxlint plugin that exists to misbehave on demand, so the fixture runner can be shown a plugin that throws.

 Five rules, each enabled alone or together by the config a test writes: one reports every program, one
 reports nothing, one throws on every program the way a crashed plugin does, one reports every program
 with a fix that puts a marker line at the top of the file, and one throws only when the process that runs
 it carries a named environment variable.

 @module
 */

/**
 Subset of the fixer these rules read.
 */
type ProbeFixer = {
  /**
   Inserts text at one offset of the source.
   */
  readonly insertTextBeforeRange: (
    range: readonly [
      number,
      number,
    ],
    text: string,
  ) => unknown;
};

/**
 Diagnostic these rules report.
 */
type ProbeDiagnostic = {
  /**
   Node the diagnostic points at.
   */
  readonly node: unknown;
  /**
   Text shown to the reader.
   */
  readonly message: string;
  /**
   Edit applied when the run asks for fixes.
   */
  readonly fix?: (fixer: ProbeFixer,) => unknown;
};

/**
 Subset of the rule context these rules read.
 */
type ProbeContext = {
  /**
   Reports one diagnostic at a node.
   */
  readonly report: (diagnostic: ProbeDiagnostic,) => void;
};

/**
 Message of the error the throwing rule raises, searched for by tests.
 */
export const PROBE_FAILURE_TEXT = 'harness probe plugin failed on purpose';

/**
 Name of the environment variable that makes the conditional rule throw.
 */
export const PROBE_THROW_VARIABLE = 'HARNESS_PROBE_THROW';

/**
 Line the fixing rule puts at the top of a source, searched for by tests.
 */
export const PROBE_FIX_MARKER = '// fixed by the harness probe plugin\n';

/**
 Plugin whose rules report, stay quiet, throw, or fix.
 */
const plugin = {
  meta: { name: 'harness-probe', },
  rules: {
    'reports-program': {
      create(context: ProbeContext,): { readonly Program: (node: unknown,) => void; } {
        return {
          Program(node,): void {
            context.report({
              node,
              message: 'harness probe report',
            },);
          },
        };
      },
    },
    'fixes-program': {
      meta: { fixable: 'code', },
      create(context: ProbeContext,): { readonly Program: (node: unknown,) => void; } {
        return {
          Program(node,): void {
            context.report({
              node,
              message: 'harness probe fix',
              fix(fixer,): unknown {
                return fixer.insertTextBeforeRange(
                  [
                    0,
                    0,
                  ],
                  PROBE_FIX_MARKER,
                );
              },
            },);
          },
        };
      },
    },
    'throws-when-asked': {
      create(): { readonly Program: () => void; } {
        return {
          Program(): void {
            if (process.env[PROBE_THROW_VARIABLE] !== undefined)
              throw new Error(PROBE_FAILURE_TEXT,);
          },
        };
      },
    },
    'reports-nothing': {
      create(): object {
        return {};
      },
    },
    'always-throws': {
      create(): { readonly Program: () => void; } {
        return {
          Program(): void {
            throw new Error(PROBE_FAILURE_TEXT,);
          },
        };
      },
    },
  },
};

export default plugin;
