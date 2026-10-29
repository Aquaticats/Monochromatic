#!/usr/bin/env node
/** Prove panic guards with real executable and public-loader controls in disposable source variants. */
import {
  readFile,
  rm,
  writeFile
} from 'node:fs/promises';
import { join } from 'node:path';
import {
  run,
  snapshot,
  ScannerVerificationError
} from './container-snapshot.mjs';

/** Exact replacement fails if the production source moves rather than silently testing an unmodified guard. */
function replace({
  source,
  oldText,
  newText
}) {
  if (source.split(oldText)
    .length
    !== 2) throw new ScannerVerificationError(`Fixture patch did not match once: ${oldText}`);
  return source.replace(
    oldText,
    newText
  );
}

/** Fault injection and disabled guards exist only in copied container source, never production configuration. */
async function inject({
  context,
  scanner,
  variant,
  test
}) {
  const root = join(
    context,
    scanner
  );
  const mainPath = join(
    root,
    'src/main.rs'
  );
  let main = await readFile(
    mainPath,
    'utf8'
  );
  main = replace({
    source: main,
    oldText: 'fn initialized_cli() -> anyhow::Result<i32> {',
    newText: [
      'fn initialized_cli() -> anyhow::Result<i32> {',
      '    let mode = std::env::args().nth(1).unwrap_or_default();',
      '    if mode == "fixture-startup-panic" { panic!("SYNTHETIC_PRIVATE_STARTUP"); }',
      '    if mode == "fixture-worker-panic" {',
      '        let worker = std::thread::Builder::new().name(String::from("SYNTHETIC_PRIVATE_THREAD")).spawn(fixture_worker).expect("worker");',
      '        if let Err(payload) = worker.join() { std::panic::resume_unwind(payload); }',
      '    }',
    ].join('\n')
  });
  main += '\n/// Fixture-only named worker.\nfn fixture_worker() { panic!("SYNTHETIC_PRIVATE_WORKER"); }\n';
  if (variant === 'without-output-hook') main = replace({
    source: main,
    oldText: 'std::panic::set_hook(Box::new(process_boundary::omit_panic_payload));',
    newText: ''
  });
  await writeFile(
    mainPath,
    main
  );
  const loadPath = join(
    root,
    'src/frx_load.rs'
  );
  const load = await readFile(
    loadPath,
    'utf8'
  );
  let faultedLoad = replace({
    source: load,
    oldText: '    if builtin_rules {',
    newText: '    if native_path.file_name() == Some(std::ffi::OsStr::new("fixture-load-panic")) { panic!("SYNTHETIC_PRIVATE_PARTIAL_RULES"); }\n    if builtin_rules {'
  });
  faultedLoad = replace({
    source: faultedLoad,
    oldText: 'Self::Runtime(rules) => return rules.line_matches(buf, starts),',
    newText: 'Self::Runtime(rules) => { let pairs = rules.line_matches(buf, starts); if buf.starts_with(b"FIXTURE_PRIVATE_FAULT") { panic!("SYNTHETIC_PRIVATE_MATCHER"); } return pairs; },'
  });
  await writeFile(
    loadPath,
    faultedLoad
  );
  if (variant === 'without-process-catch') {
    const path = join(
      root,
      'src/process_boundary.rs'
    );
    await writeFile(
      path,
      replace({
        source: await readFile(
          path,
          'utf8'
        ),
        oldText: 'std::panic::catch_unwind(operation)',
        newText: 'Ok::<Result<i32>, ()>(operation())'
      })
    );
  }
  if (variant === 'without-load-catch') {
    const path = join(
      root,
      'src/load_request.rs'
    );
    await writeFile(
      path,
      replace({
        source: await readFile(
          path,
          'utf8'
        ),
        oldText: 'std::panic::catch_unwind(operation)',
        newText: 'Ok::<Result<LoadedRules>, ()>(operation())'
      })
    );
  }
  if ((variant === 'without-content-catch') || (variant === 'without-name-catch')) {
    const path = join(
      root,
      'src',
      variant === 'without-content-catch' ? 'frx_scan.rs' : 'path_scan.rs'
    );
    await writeFile(
      path,
      replace({
        source: await readFile(
          path,
          'utf8'
        ),
        oldText: 'catch_unwind(matcher)',
        newText: 'Ok::<Vec<(usize, usize)>, ()>(matcher())'
      })
    );
  }
  await writeFile(
    join(
      root,
      'tests/panic_contract.rs'
    ),
    test
  );
}

/** Every disabled variant must fail the previously passing assertion, not merely fail compilation. */
async function main() {
  if (process.argv
    .length
    !== 2) throw new ScannerVerificationError('This task accepts no arguments.');
  // Freeze the committed consumer once: edits during a multi-variant campaign must not change its assertions.
  const test = await readFile(join(
    import.meta.dirname,
    'guard-fixture.rs.txt'
  ));
  for (const variant of [
    'protected',
    'without-output-hook',
    'without-process-catch',
    'without-load-catch',
    'without-content-catch',
    'without-name-catch'
  ]) {
    const fixture = await snapshot({
      name: `guard-${variant}`,
      command: [
        'cargo',
        'test',
        '--offline',
        '--locked',
        '--test',
        'panic_contract',
        '--',
        '--nocapture',
        '--test-threads=1'
      ],
      transform: async ({
        context,
        scanner
      }) =>{  await inject({
        context,
        scanner,
        variant,
        test
      }); },
    });
    try {
      const result = run({
        command: 'podman',
        args: [
          'run',
          '--rm',
          ...fixture.limits,
          '--env',
          'SCANNER_GUARD_HOME=/home/tester',
          '--env',
          'FORBIDDEN_STRINGS_CACHE_DIR=/home/tester/cache',
          fixture.image
        ],
        allowFailure: true,
        transcript: fixture.evidence
      });
      const stdout = await readFile(
        join(
          fixture.evidence,
          'stdout.log'
        ),
        'utf8'
      );
      const stderr = await readFile(
        join(
          fixture.evidence,
          'stderr.log'
        ),
        'utf8'
      );
      const output = stdout + stderr;
      const protectedPass = (result.status === 0) && stdout.includes('3 passed')
        && stderr.includes('fixture-host-hook-called');
      const disabledFailure = (result.status === 101) && stdout.includes('FAILED')
        && output.includes('test result: FAILED');
      let expected = variant === 'protected' ? protectedPass : disabledFailure;
      if (variant === 'without-output-hook') expected = expected && output.includes('SYNTHETIC_PRIVATE_STARTUP');
      if (variant === 'without-process-catch') expected = expected && output.includes('actual CLI exit must be two');
      if (variant === 'without-load-catch') expected = expected && stdout.includes('public_loader_discards_partial_rules_and_preserves_host_hook ... FAILED');
      if ((variant === 'without-content-catch') || (variant === 'without-name-catch'))
        expected = expected && stdout.includes('public_scan_catches_faults_without_retaining_partial_hits ... FAILED');
      await writeFile(
        join(
          fixture.evidence,
          'control.json'
        ),
        `${JSON.stringify(
          {
            variant,
            status: result.status,
            expected,
            protectedPass,
            disabledFailure
          },
          null,
          2
        )  }\n`
      );
      if (!expected) throw new ScannerVerificationError(`Panic control did not prove the expected boundary: ${variant}; inspect ${fixture.evidence}.`);
    } finally {
      await rm(
        fixture.context,
        {
          recursive: true,
          force: true
        }
      );
    }
  }
}

await main();
