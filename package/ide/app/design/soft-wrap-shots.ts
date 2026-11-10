// Screenshots of the wrapped source view in the nested compositor, light and dark.
// What: Opens a sample file whose lines wrap in the built IDE inside the nested compositor and captures it.
// Why: The soft-wrap frames in `screenshots/2026-10-12-soft-wrap-applied/` come from this script.
// Usage: node design/soft-wrap-shots.ts <application> <output directory> [compositor]
// Needs `WAYLAND_DISPLAY` and `XDG_RUNTIME_DIR` of a running Wayland session.
import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import { captureSession, disposableProject, settleScreenshot, sessionEnvironment, wait } from './capture-shared.ts';

const [application, output, compositorArg] = process.argv.slice(2);
if (application === undefined || output === undefined) throw new Error('Usage: node design/soft-wrap-shots.ts <application> <output directory> [compositor]');
mkdirSync(output, { recursive: true });
const sample = [
  '// A long comment line that goes on and on to show how the text wraps at the right edge of the text area, with the number only on its first row.',
  'fn main() {',
  '    let message = format!("a long string literal that wraps because it does not fit the width of the pane {} {}", first_value, second_value);',
  '        let deeply_indented = compute_total(first_item, second_item, third_item, fourth_item, fifth_item, sixth_item, seventh_item); // a trailing comment',
  '    short();',
  '\tlet tabbed = values.iter().map(|value| value * 2).filter(|value| value % 3 == 0).collect::<Vec<u32>>().len(); // tab indented and long',
  '    let unbroken = "' + 'x'.repeat(190) + '";',
  '}',
  '',
].join('\n');
for (const scheme of ['light', 'dark']) {
  const directory = join(output, 'work-' + scheme);
  const project = disposableProject({ directory, sample });
  const env = sessionEnvironment({ directory });
  await captureSession({
    application,
    project,
    file: 'sample.rs',
    scheme,
    size: '1100x660',
    directory,
    env,
    logPath: join(output, 'session-' + scheme + '.log'),
    ...(compositorArg === undefined ? {} : { compositor: compositorArg }),
  }, async control => {
    await wait(4000);
    await settleScreenshot({ control, path: join(output, 'wrap-' + scheme + '.png') });
    // Drag a selection from the middle of the first long row to the middle of the third long line, across wrapped rows.
    console.log(await control('click 400 56'));
    await wait(700);
    await settleScreenshot({ control, path: join(output, 'wrap-' + scheme + '-click.png') });
    await control('key shift press');
    await wait(200);
    console.log(await control('click 520 230'));
    await wait(300);
    await control('key shift release');
    await wait(500);
    await settleScreenshot({ control, path: join(output, 'wrap-' + scheme + '-selection.png') });
  });
}
console.log('done');
