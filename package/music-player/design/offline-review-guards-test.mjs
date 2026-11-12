// The answers every guard in offline-review-guards.mjs must keep, recorded from the patterns the regex-necessity
// audit filed (#776 to #786) before those patterns were removed.
//
// Each table holds the accepted shape, the near miss beside it, and the boundary the removed class, `\b`, `$` or
// quote rule drew, so a scanner that starts answering a different question fails here rather than in a review page.

import { countTagOpenings } from './offline-review-guards.mjs';

//region Recorded answers, one table per guard
const guards = {
  // `/ <form\b /g` counted with `.match`, so the answer is a count and the comparison never folded case.
  formOpeningCount: {
    answer: page => countTagOpenings({
      page,
      tagName: 'form'
    }),
    cases: [
      ['', 0],
      ['<form>', 1],
      ['<form id="review-form">', 1],
      ['<form/>', 1],
      ['<form', 1],
      ['<form\n>', 1],
      ['<formx>', 0],
      ['<form_>', 0],
      ['<form1>', 0],
      ['<FORM>', 0],
      ['<forms>', 0],
      ['</form>', 0],
      ['<form><form>', 2],
      ['<div><form></form><form></form></div>', 2],
      ['<div data-x="<form>"></div>', 1],
      ['<textarea data-x="<formx>">', 0]
    ]
  }
};
//endregion

//region Every table replays, and every table reaches both answers
/**
 * Replays one table, naming the case that disagrees, and refuses a table whose cases never reach both answers:
 * silence proves nothing about a guard that could only be wrong by speaking.
 */
function expectAnswers({ name, answer, cases }) {
  const reached = {
    answered: 0,
    silent: 0
  };
  for (const [input, recorded] of cases) {
    const given = answer(input);
    if (JSON.stringify(given) !== JSON.stringify(recorded)) {
      throw new Error(`${name} answers ${JSON.stringify(input)} with ${JSON.stringify(given)}; the recorded answer is ${JSON.stringify(recorded)}.`);
    }
    const answered = Array.isArray(recorded) ? (recorded.length > 0)
      : ((typeof recorded === 'number') ? (recorded > 0) : recorded);
    if (answered) reached.answered += 1;
    else reached.silent += 1;
  }
  if ((reached.answered === 0) || (reached.silent === 0)) {
    throw new Error(`The ${name} cases reach only one answer (${reached.answered} answered, ${reached.silent} silent), so they prove nothing.`);
  }
  return cases.length;
}

let replayed = 0;
for (const [name, table] of Object.entries(guards)) replayed += expectAnswers({
  name,
  ...table
});
console.log(`Offline review guards: ${replayed} recorded answers across ${Object.keys(guards).length} guards.`);
//endregion
