// Exercises need-rule-scan.mjs on the real Settings-pane builder and on synthetic builder sources.
// The real builder proves the shape the formatter produces is read whole; the synthetic sources pin
// the shapes that must fail loudly instead of leaving a rule unexercised.
import {
  needCalls,
  needRuleNames,
  withoutNeedRule
} from './need-rule-scan.mjs';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

//region Harness

const failures = [];

/**
 Number of checks that ran, so the completion message reports a measured count.
 */
let checksRun = 0;

function check({
  name,
  expected,
  actual
}) {
  checksRun += 1;
  if (JSON.stringify(expected) === JSON.stringify(actual)) return;
  failures.push(`${name}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
}

function checkFails({
  name,
  message,
  run
}) {
  checksRun += 1;
  try {
    run();
  } catch (error) {
    if (error.message.includes(message)) return;
    failures.push(`${name}: failed with ${JSON.stringify(error.message)} instead of holding ${JSON.stringify(message)}`);
    return;
  }
  failures.push(`${name}: returned instead of throwing ${JSON.stringify(message)}`);
}

//endregion Harness

//region Real builder source

/**
 Settings-pane builder source text, read the way the consumer test reads it.
 */
const builderSource = readFileSync(
  join(
    import.meta.dirname,
    'settings-pane.mjs'
  ),
  'utf8'
);

const builderRules = needRuleNames({ source: builderSource });
const builderCalls = needCalls({ source: builderSource });

if (builderRules.length === 0) {
  throw new Error('Settings-pane builder declares no rule statements; the parse found no shape it knows.');
}
check({
  name: 'each real rule name appears once',
  expected: builderRules.length,
  actual: new Set(builderRules).size
});
check({
  name: 'one statement per real rule',
  expected: builderRules.length,
  actual: builderCalls.length
});
check({
  name: 'every real statement names its own rule',
  expected: [],
  actual: builderCalls.filter((call, index) => call.rule !== builderRules[index])
    .map((call) => call.rule)
});
check({
  name: 'every real statement span slices back to itself',
  expected: [],
  actual: builderCalls.filter((call) => builderSource.slice(call.start, call.end) !== call.text)
    .map((call) => call.rule)
});
check({
  name: 'every real statement is a complete need call',
  expected: [],
  actual: builderCalls.filter((call) => !(call.text.trimStart()
    .startsWith('need({'))
    || !(call.text.trimEnd()
      .endsWith('});'))
    || !call.text.includes('detail:'))
    .map((call) => call.rule)
});

// The audit's no-regex site for this shape was the rule list, so a targeted rule must be present.
if (!builderRules.includes('no-input-control')) {
  throw new Error('Settings-pane builder lost its no-input-control rule.');
}

/**
 A builder copy with one rule's statement removed.
 */
const withoutFirst = withoutNeedRule({
  source: builderSource,
  rule: builderRules[0]
});
check({
  name: 'removing one real rule leaves the rest',
  expected: builderRules.length - 1,
  actual: needRuleNames({ source: withoutFirst }).length
});
check({
  name: 'removing one real rule drops only that name',
  expected: false,
  actual: needRuleNames({ source: withoutFirst }).includes(builderRules[0])
});
check({
  name: 'removing one real rule drops exactly that span',
  expected: builderSource.length - builderCalls[0].text.length,
  actual: withoutFirst.length
});

//endregion Real builder source

//region Synthetic builder sources

/**
 Source text holding one statement in the multi-line shape the formatter produces.
 */
const MULTI_LINE = [
  'function need({ rule, holds, detail }) {',
  '  if (!holds) throw new Error(`${rule} ${detail}`);',
  '}',
  'need({',
  "  rule: 'first-rule',",
  '  holds: true,',
  "  detail: 'a detail'",
  '});',
  'need({',
  "  rule: 'second-rule',",
  '  holds: false,',
  "  detail: 'another detail'",
  '});',
  '',
].join('\n');

check({
  name: 'declaration is not a statement',
  expected: ['first-rule', 'second-rule'],
  actual: needRuleNames({ source: MULTI_LINE })
});
check({
  name: 'a duplicated statement collapses to one name',
  expected: ['first-rule'],
  actual: needRuleNames({ source: MULTI_LINE.replace('second-rule', 'first-rule') })
});
checkFails({
  name: 'a duplicate name is not silently removed once',
  message: 'Expected one Settings-pane rule named first-rule; found 2.',
  run: () => withoutNeedRule({
    source: MULTI_LINE.replace('second-rule', 'first-rule'),
    rule: 'first-rule'
  })
});
checkFails({
  name: 'an absent rule is reported',
  message: 'Expected one Settings-pane rule named third-rule; found 0.',
  run: () => withoutNeedRule({
    source: MULTI_LINE,
    rule: 'third-rule'
  })
});
checkFails({
  name: 'an unterminated statement is reported',
  message: 'is never closed',
  run: () => needRuleNames({ source: MULTI_LINE.slice(0, MULTI_LINE.indexOf("  rule: 'second-rule'")) })
});
checkFails({
  name: 'a statement without a rule name is reported',
  message: 'has no rule name',
  run: () => needRuleNames({
    source: MULTI_LINE.replace("rule: 'first-rule',", 'rule,')
  })
});
checkFails({
  name: 'a one-line statement is reported rather than skipped',
  message: 'is not a multi-line statement',
  run: () => needRuleNames({ source: "need({ rule: 'first-rule', holds: true, detail: 'x' });" })
});
check({
  name: 'a statement with a double-quoted rule name is read',
  expected: ['quoted-rule'],
  actual: needRuleNames({
    source: 'need({\n  rule: "quoted-rule",\n  holds: true\n});\n'
  })
});
checkFails({
  name: 'a rule name with a capital is reported rather than read',
  message: 'has no rule name',
  run: () => needRuleNames({
    source: 'need({\n  rule: "Quoted-Rule",\n  holds: true\n});\n'
  })
});

//endregion Synthetic builder sources

if (failures.length > 0) {
  throw new Error(`${failures.length} rule scan ${failures.length === 1 ? 'check disagrees' : 'checks disagree'}:\n${failures.join('\n')}`);
}
console.log(`Rule scan: ${builderRules.length} Settings-pane rules read whole, and ${checksRun} checks held.`);
