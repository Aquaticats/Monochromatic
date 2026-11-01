//region Detached entry

/**
 Startup code travels in argv rather than in a file that cancellation can delete.
 Paths stay in separate arguments, never interpolated into JavaScript syntax.
 Only missing request-owned inputs mean cancellation; other load failures remain errors.
 Reading the bundle before importing also retains its bytes through workspace removal.
 The self-contained module uses a data URL so valid filesystem names need not be module URLs.
 The helper still authenticates before opening an editor.
 */
export const ANSWER_BOOTSTRAP = `
import { readFile } from 'node:fs/promises';
const helperPath = process.argv[1];
try {
  const source = await readFile(helperPath);
  await import('data:text/javascript;base64,' + source.toString('base64'));
} catch (error) {
  if (error.code === 'ENOENT'
    && (error.path === helperPath || error.path === process.argv.at(-1))) {
    console.log('This question is no longer active. Return to Pi for the current question.');
  } else {
    throw error;
  }
}
`;

//endregion Detached entry
