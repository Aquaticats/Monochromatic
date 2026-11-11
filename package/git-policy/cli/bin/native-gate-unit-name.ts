/**
 The name one gate carries: its label, the timestamp that keeps two gates of one label apart, and the systemd unit name
 the two make.

 A label is what the gate covers, and it names three things at once: the systemd unit, the log file, and the image tag of
 the tests it builds (`GIT_POLICY_NATIVE_IMAGE_TAG`). It is therefore short kebab case rather than a sentence, and it is
 refused here rather than by systemd, which would report a unit name it cannot read long after the label was typed.
 */

/**
 Prefix of every gate unit name, kept from the scratch launcher this command replaced, so the existing logs, journal
 entries and `gpc-gate-*` searches of this repository keep naming the same runs.
 */
export const gateUnitNamePrefix = 'gpc-gate';

/**
 Label the refusal quotes as usable, so a person sees the shape rather than a rule.
 */
const usableLabelExample = 'parallel-rounds-rebuilt-note';

/**
 Characters a label keeps: lowercase letters, digits and hyphens.
 */
const labelCharacters = 'abcdefghijklmnopqrstuvwxyz0123456789-';

/**
 Why a label cannot name a unit, one line each, empty when it can.

 @param label - what the gate covers

 @returns one line per refusal, which is one line at most

 @example
 ```ts
 gateLabelProblems({ label: 'wrapper-next-alpha-on-47eb745abb' });
 ```
 */
export function gateLabelProblems({ label }: { readonly label: string; }): readonly string[] {
  if (label === '')
    return [`the label is empty, and it names the gate's systemd unit, its log file and the image tag of its tests; pass a short kebab-case slug such as ${usableLabelExample}.`];
  for (let index = 0; index < label.length; index += 1) {
    /**
     The character at this position.
     */
    const character = label.charAt(index);
    if (!labelCharacters.includes(character))
      return [`the label "${label}" must use only lowercase letters, digits and hyphens, because it names the systemd unit, the log file and the image tag; for example ${usableLabelExample}.`];
  }
  return [];
}

/**
 Digits a two-part stamp component keeps.
 */
const twoDigits = 2;

/**
 Minutes in one hour, which turns a zone offset into a sign and two components.
 */
const minutesPerHour = 60;

/**
 Digits a year keeps.
 */
const yearDigits = 4;

/**
 A number in a fixed width, zero filled, so every part of a stamp keeps its place.

 @param value - the number to write

 @param width - how many digits it fills

 @returns the number in that many digits

 @example
 ```ts
 padded({ value: 7, width: 2 });
 ```
 */
function padded({
  value,
  width,
}: {
  readonly value: number;
  readonly width: number;
}): string {
  return String(value)
    .padStart(
    width,
    '0',
    );
}

/**
 One moment as a second-precise ISO 8601 basic timestamp with its offset, the shape `date +%Y%m%dT%H%M%S%z` prints.

 The value is computed here rather than read from `date`, so the unit name and the log name a test checks are the ones a
 run makes, with no second program to stand in for (rule `WHN`, `AGENTS.md`: never a bare date and never a round
 timestamp).

 @param moment - the moment the gate starts

 @returns the stamp, such as `20261110T180203-0500`

 @example
 ```ts
 localStamp({ moment: new Date('2026-11-10T23:02:03Z') });
 ```
 */
export function localStamp({ moment }: { readonly moment: Date; }): string {
  /**
   Minutes this zone is offset from UTC, counted as the time zone reports it: positive west of UTC.
   */
  const offsetMinutes = moment.getTimezoneOffset();
  /**
   The offset in the sign and four digits a timestamp carries, so a run in another zone names another hour.
   */
  const offset = `${offsetMinutes > 0 ? '-' : '+'}${padded({
    value: Math.floor(Math.abs(offsetMinutes) / minutesPerHour),
    width: twoDigits,
  })}${padded({
    value: Math.abs(offsetMinutes) % minutesPerHour,
    width: twoDigits,
  })}`;
  return [
    padded({
      value: moment.getFullYear(),
      width: yearDigits,
    }),
    padded({
      value: moment.getMonth() + 1,
      width: twoDigits,
    }),
    padded({
      value: moment.getDate(),
      width: twoDigits,
    }),
    'T',
    padded({
      value: moment.getHours(),
      width: twoDigits,
    }),
    padded({
      value: moment.getMinutes(),
      width: twoDigits,
    }),
    padded({
      value: moment.getSeconds(),
      width: twoDigits,
    }),
    offset,
  ].join('');
}

/**
 The name of one gate's systemd unit.

 @param label - what the gate covers, already checked by {@link gateLabelProblems}

 @param stamp - the moment the gate starts, from {@link localStamp}

 @returns the unit name, without the `.service` suffix

 @example
 ```ts
 gateUnitName({ label: 'wrapper-next-alpha', stamp: '20261110T180203-0500' });
 ```
 */
export function gateUnitName({
  label,
  stamp,
}: {
  readonly label: string;
  readonly stamp: string;
}): string {
  return `${gateUnitNamePrefix}-${label}-${stamp}`;
}

/**
 The part of a gate worktree's name a label is made of: a leading `gate-` is dropped, because every gate worktree carries
 it and the label already says what the unit is.

 @param worktree - the worktree name or path the gate runs in

 @returns the name without its leading `gate-`

 @example
 ```ts
 gateLabelStem({ worktree: 'gate-wrapper-next-alpha' });
 ```
 */
export function gateLabelStem({ worktree }: { readonly worktree: string; }): string {
  /**
   The last segment of the worktree, which is its name when a path was given.
   */
  const name = worktree.split('/')
    .filter(function hasText(segment) {
      return segment !== '';
    })
    .at(-1)
    ?? worktree;
  return name.startsWith('gate-')
    ? name.slice('gate-'.length)
    : name;
}

/**
 The label of a gate started at the tip of `main`: the worktree's own name and the commit it was moved to, so a log names
 the tree it tested even after `main` moved on.

 @param worktree - the worktree name or path the gate runs in

 @param sha - the commit, as `git rev-parse --short=<digits>` prints it

 @returns the label

 @example
 ```ts
 gateLabelOnMainTip({ worktree: 'gate-wrapper-next-alpha', sha: '47eb745abb' });
 ```
 */
export function gateLabelOnMainTip({
  worktree,
  sha,
}: {
  readonly worktree: string;
  readonly sha: string;
}): string {
  return `${gateLabelStem({ worktree })}-on-${sha}`;
}
