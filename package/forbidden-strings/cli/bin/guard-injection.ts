/**
 Fault injection and disabled guards for the panic-guard controls, applied to copied container source only.
 */
import {
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import { ScannerVerificationError } from './container-snapshot.ts';

/**
 The control variants: the protected scanner, then one with each panic guard disabled.
 */
export const guardVariants: readonly string[] = [
  'protected',
  'without-output-hook',
  'without-process-catch',
  'without-load-catch',
  'without-content-catch',
  'without-name-catch',
];

/**
 Exact replacement fails if the production source moves rather than silently testing an unmodified guard.

 @param source - the text to edit

 @param oldText - the text that must occur exactly once

 @param newText - what replaces it

 @returns the edited text

 @throws ScannerVerificationError when the text to replace does not occur exactly once

 @example
 ```ts
 replace({ source: 'a b', oldText: 'a', newText: 'c' });
 ```
 */
export function replace({
  source,
  oldText,
  newText,
}: {
  readonly source: string;
  readonly oldText: string;
  readonly newText: string;
}): string {
  if (source.split(oldText)
    .length
    !== 2) throw new ScannerVerificationError(`Fixture patch did not match once: ${oldText}`);
  return source.replace(
    oldText,
    newText,
  );
}

/**
 Edit one copied source file in place.

 @param path - the file to edit

 @param oldText - the text that must occur exactly once

 @param newText - what replaces it

 @throws ScannerVerificationError when the text to replace does not occur exactly once

 @example
 ```ts
 await patchFile({ path, oldText: 'a', newText: 'b' });
 ```
 */
async function patchFile({
  path,
  oldText,
  newText,
}: {
  readonly path: string;
  readonly oldText: string;
  readonly newText: string;
}): Promise<void> {
  await writeFile(
    path,
    replace({
      source: await readFile(
        path,
        'utf8',
      ),
      oldText,
      newText,
    }),
  );
}

/**
 Add the fixture-only startup and worker panics to the scanner's main source, and drop the output hook when asked.

 @param path - the copied `src/main.rs`

 @param variant - the control variant

 @throws ScannerVerificationError when a patched text does not occur exactly once

 @example
 ```ts
 await injectMain({ path, variant: 'protected' });
 ```
 */
async function injectMain({
  path,
  variant,
}: {
  readonly path: string;
  readonly variant: string;
}): Promise<void> {
  /**
   The main source with the fixture-only panics.
   */
  const faulted = `${replace({
    source: await readFile(
      path,
      'utf8',
    ),
    oldText: 'fn initialized_cli() -> anyhow::Result<i32> {',
    newText: [
      'fn initialized_cli() -> anyhow::Result<i32> {',
      '    let mode = std::env::args().nth(1).unwrap_or_default();',
      '    if mode == "fixture-startup-panic" { panic!("SYNTHETIC_PRIVATE_STARTUP"); }',
      '    if mode == "fixture-worker-panic" {',
      '        let worker = std::thread::Builder::new().name(String::from("SYNTHETIC_PRIVATE_THREAD")).spawn(fixture_worker).expect("worker");',
      '        if let Err(payload) = worker.join() { std::panic::resume_unwind(payload); }',
      '    }',
    ].join('\n'),
  })}\n/// Fixture-only named worker.\nfn fixture_worker() { panic!("SYNTHETIC_PRIVATE_WORKER"); }\n`;
  await writeFile(
    path,
    variant === 'without-output-hook' ? replace({
      source: faulted,
      oldText: 'std::panic::set_hook(Box::new(process_boundary::omit_panic_payload));',
      newText: '',
    }) : faulted,
  );
}

/**
 Add the fixture-only load and matcher panics to the rule loader's source.

 @param path - the copied `src/frx_load.rs`

 @throws ScannerVerificationError when a patched text does not occur exactly once

 @example
 ```ts
 await injectLoad(path);
 ```
 */
async function injectLoad(path: string): Promise<void> {
  /**
   The loader source with the partial-rules panic.
   */
  const withLoadPanic = replace({
    source: await readFile(
      path,
      'utf8',
    ),
    oldText: '    if builtin_rules {',
    newText: '    if native_path.file_name() == Some(std::ffi::OsStr::new("fixture-load-panic")) { panic!("SYNTHETIC_PRIVATE_PARTIAL_RULES"); }\n    if builtin_rules {',
  });
  await writeFile(
    path,
    replace({
      source: withLoadPanic,
      oldText: 'Self::Runtime(rules) => return rules.line_matches(buf, starts),',
      newText: 'Self::Runtime(rules) => { let pairs = rules.line_matches(buf, starts); if buf.starts_with(b"FIXTURE_PRIVATE_FAULT") { panic!("SYNTHETIC_PRIVATE_MATCHER"); } return pairs; },',
    }),
  );
}

/**
 Remove the one panic catch that a variant disables.

 @param root - the copied scanner directory

 @param variant - the control variant

 @throws ScannerVerificationError when the catch is not found exactly once

 @example
 ```ts
 await disableCatch({ root, variant: 'without-load-catch' });
 ```
 */
async function disableCatch({
  root,
  variant,
}: {
  readonly root: string;
  readonly variant: string;
}): Promise<void> {
  if (variant === 'without-process-catch') await patchFile({
    path: join(
      root,
      'src/process_boundary.rs',
    ),
    oldText: 'std::panic::catch_unwind(operation)',
    newText: 'Ok::<Result<i32>, ()>(operation())',
  });
  if (variant === 'without-load-catch') await patchFile({
    path: join(
      root,
      'src/load_request.rs',
    ),
    oldText: 'std::panic::catch_unwind(operation)',
    newText: 'Ok::<Result<LoadedRules>, ()>(operation())',
  });
  if ((variant === 'without-content-catch') || (variant === 'without-name-catch')) await patchFile({
    path: join(
      root,
      'src',
      variant === 'without-content-catch' ? 'frx_scan.rs' : 'path_scan.rs',
    ),
    oldText: 'catch_unwind(matcher)',
    newText: 'Ok::<Vec<(usize, usize)>, ()>(matcher())',
  });
}

/**
 Fault injection and disabled guards exist only in copied container source, never production configuration.

 @param context - the build context

 @param scanner - the scanner directory inside the context

 @param variant - the control variant

 @param test - the frozen consumer test, written as `tests/panic_contract.rs`

 @throws ScannerVerificationError when a patched text does not occur exactly once

 @example
 ```ts
 await inject({ context, scanner, variant: 'protected', test });
 ```
 */
export async function inject({
  context,
  scanner,
  variant,
  test,
}: {
  readonly context: string;
  readonly scanner: string;
  readonly variant: string;
  readonly test: string;
}): Promise<void> {
  /**
   The copied scanner directory.
   */
  const root = join(
    context,
    scanner,
  );
  await injectMain({
    path: join(
      root,
      'src/main.rs',
    ),
    variant,
  });
  await injectLoad(join(
    root,
    'src/frx_load.rs',
  ));
  await disableCatch({
    root,
    variant,
  });
  await writeFile(
    join(
      root,
      'tests/panic_contract.rs',
    ),
    test,
  );
}
