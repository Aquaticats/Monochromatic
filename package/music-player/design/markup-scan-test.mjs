// Replays markup-scan-corpus.mjs against every scan in markup-scan.mjs.
// The corpus holds what the replaced patterns returned; this file proves the scans return the same.
// A scan without a corpus group fails here too, so a new scan cannot arrive unevidenced.
import * as scans from './markup-scan.mjs';
import { CORPUS } from './markup-scan-corpus.mjs';

//region Scan dispatch

/**
 Calls one scan for a corpus group's row input.

 @param group - corpus group naming the scan and its extra arguments

 @param input - row input text

 @returns the scan's verdict for that input
 */
function scanFor({
  group,
  input
}) {
  const extra = group.arguments;
  switch (group.scan) {
    case 'boundsRectangle':
      return scans.boundsRectangle({ text: input });
    case 'countTagStarts':
      return scans.countTagStarts({
        text: input,
        ...extra
      });
    case 'hasClipboardUse':
      return scans.hasClipboardUse({ text: input });
    case 'hasExternalSubresourceReference':
      return scans.hasExternalSubresourceReference({ text: input });
    case 'hasQuotedAttributeValue':
      return scans.hasQuotedAttributeValue({
        text: input,
        ...extra
      });
    case 'hasRequiredControlMarker':
      return scans.hasRequiredControlMarker({ text: input });
    case 'tagElements':
      return scans.tagElements({
        text: input,
        ...extra
      });
    default:
      throw new Error(`Corpus group names no known scan: ${group.scan}`);
  }
}

//endregion Scan dispatch

//region Coverage and replay

const scanned = new Set(CORPUS.map((group) => group.scan));
const unexercised = Object.keys(scans)
  .filter((name) => !scanned.has(name));
if (unexercised.length > 0) {
  throw new Error(`Scans without a recorded verdict group: ${unexercised.join(', ')}`);
}

const failures = [];
let rows = 0;
for (const group of CORPUS) {
  for (const row of group.rows) {
    rows += 1;
    const actual = scanFor({
      group,
      input: row.input
    });
    if (JSON.stringify(actual) === JSON.stringify(row.expected)) continue;
    failures.push(`${group.scan} ${JSON.stringify(group.arguments)} on ${JSON.stringify(row.input)}: recorded ${JSON.stringify(row.expected)}, scan ${JSON.stringify(actual)}`);
  }
}
if (failures.length > 0) {
  throw new Error(`${failures.length} of ${rows} recorded verdicts disagree with the scans:\n${failures.join('\n')}`);
}

//endregion Coverage and replay

console.log(`Markup scan: ${rows} recorded pattern verdicts across ${CORPUS.length} groups agree with every scan.`);
