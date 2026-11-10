// Screenshots of hints and diagnostic messages on wrapped lines in the nested compositor, light and dark.
// What: Opens a sample file whose lines wrap in a debug build of the IDE, with hints and diagnostics injected through
//       the debug-only inspection file (`IDE_INSPECT_ANNOTATIONS`), at a wide and a narrow window.
// Why: The frames in `screenshots/2026-10-12-wrap-annotations-applied/` come from this script.
// Usage: node design/wrap-annotation-shots.ts <debug application> <output directory> <runtime directory> [compositor]
// The application must be a debug build (`mise run //package/ide/app:build:debug`): release builds have no
// inspection path. `runtime directory` is `target/debug/runtime` of that build.
// Needs `WAYLAND_DISPLAY` and `XDG_RUNTIME_DIR` of a running Wayland session.
import { mkdirSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { captureSession, disposableProject, settleScreenshot, sessionEnvironment, wait } from './capture-shared.ts';

const [application, output, runtime, compositorArg] = process.argv.slice(2);
if (application === undefined || output === undefined || runtime === undefined) {
  throw new Error('Usage: node design/wrap-annotation-shots.ts <debug application> <output directory> <runtime directory> [compositor]');
}
mkdirSync(output, { recursive: true });
const lines = [
  '// A comment line with a hint and a message that both wrap, to show where blocks stand over a wrapped line.',
  'fn main() {',
  '    let message = format!("a long string literal that wraps because it does not fit the width of the pane {} {}", first_value, second_value);',
  '    short();',
  '        let deeply_indented = compute_total(first_item, second_item, third_item, fourth_item, fifth_item, sixth_item); // a trailing comment',
  '    let total = area(2, 3);',
  '}',
  '',
];
const sample = lines.join('\n');
// Character offsets of the injected places, from the sample itself.
const at = (needle: string): number => {
  const index = sample.indexOf(needle);
  if (index < 0) throw new Error('missing ' + needle);
  return [...sample.slice(0, index)].length;
};
const longMessage = 'expected `String`, found `&str`: the argument named second_value is a string slice here, so it has to be converted with `to_string` before it can be formatted into the message of the macro';
const injected = {
  hints: [
    { position: at('message ='), label: ': String', kind: 'type' },
    { position: at('first_value,'), label: 'first:', kind: 'parameter' },
    { position: at('second_value)'), label: 'second:', kind: 'parameter' },
    { position: at('compute_total(') + 'compute_total('.length, label: 'first_item:', kind: 'parameter' },
    { position: at('sixth_item)'), label: 'sixth_item:', kind: 'parameter' },
    { position: at('area(2') + 'area('.length, label: 'width:', kind: 'parameter' },
  ],
  diagnostics: [
    { source: 'rustc', start: at('second_value)'), end: at('second_value)') + 'second_value'.length, severity: 'error', code: 'E0308', message: longMessage },
    { source: 'rustc', start: at('short();'), end: at('short();') + 5, severity: 'warning', code: 'W0001', message: 'unused result' },
    { source: 'clippy', start: at('deeply_indented'), end: at('deeply_indented') + 'deeply_indented'.length, severity: 'warning', code: 'clippy::needless_late_init', message: 'this binding is only used in the next call, so it could be written inline' },
  ],
};
const sizes = [['wide', '1100x660'], ['narrow', '760x660']] as const;
for (const scheme of ['light', 'dark']) {
  for (const [name, size] of sizes) {
    const directory = join(output, 'work-' + scheme + '-' + name);
    const project = disposableProject({ directory, sample });
    const inspection = join(directory, 'annotations.json');
    writeFileSync(inspection, JSON.stringify(injected, null, 2));
    const env = { ...sessionEnvironment({ directory, helixRuntime: resolve(runtime) }), IDE_INSPECT_ANNOTATIONS: inspection };
    await captureSession({
      application,
      project,
      file: 'sample.rs',
      scheme,
      size,
      directory,
      env,
      logPath: join(output, 'session-' + scheme + '-' + name + '.log'),
      ...(compositorArg === undefined ? {} : { compositor: compositorArg }),
    }, async control => {
      await wait(5000);
      await settleScreenshot({ control, path: join(output, 'wrap-annotations-' + scheme + '-' + name + '.png') });
    });
  }
}
console.log('done');
