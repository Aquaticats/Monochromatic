# `findmnt` with two operands prints nothing unless one mount has both as source and target

## Symptom

The guided runner's `f1-images` stage mounted two loop images and then failed its own check.
The check ran `findmnt --noheadings --output SOURCE,TARGET <folder>/source <folder>/dest` and read no output,
while `/proc/self/mountinfo` listed both folders on loop devices.
The first reading blamed a link in the fixture folder's path.
That reading was wrong.

## Measured cause

util-linux 2.42.4, measured 2026-11-07 at user level on two mounted folders:

- `findmnt --noheadings --output SOURCE,TARGET /var/mnt/encrypted` printed its line.
- The same call through a symbolic link to that folder printed the same line,
  so `findmnt` canonicalizes a path operand before comparing.
- `findmnt --noheadings --output SOURCE,TARGET /var/mnt/encrypted /var` printed nothing and exited 0.
  Two operands mean one source and one target of the same mount (`findmnt [options] device mountpoint`),
  so two mount points never match.
- `findmnt --mountpoint <path>` once per folder printed each line,
  and exited 1 for a folder that was not a mount point.

## Remedy

Ask once per folder with `--mountpoint`.
`package/repository-move/cli/src/guided-loop-image-stages.data.ts` does,
and `package/repository-move/cli/src/guided-runner.unit.test.ts` fails any stage text that gives `findmnt` two operands.
