/**
 Whether a capture handed to an external package's retained callback is charged.
 
 Its own test, over a dependency this file writes, because no other route exists. The external channel
 needs an installed package with a locked version whose shipped implementation the analyzer can load,
 and this workspace has none it can reach: a `workspace:*` dependency appears in `pnpm-lock.yaml` only
 as `link:package/...`, never as a `name@version` key, so `packageVersionIsLocked` answers false for
 every package the repo owns.
 
 Authoring one satisfies every gate, because `packageVersionIsLocked` walks ancestors of
 `consumerProject.configFileName` rather than the repository root. A disposable consumer directory
 therefore owns its own lockfile, its own configured project, and its own `node_modules`.
 
 What the authored lockfile does **not** prove: that pnpm would emit that key for a package installed
 this way. It would not. A `file:` tarball is keyed `name@file:/absolute/path`, and a workspace link is
 keyed `link:`, so neither reaches the gate. The key shape here is copied from this repository's own
 `pnpm-lock.yaml`, and the gate under test reads a lockfile rather than an installer, so a copied shape
 is the sound input. The boundary is worth stating rather than leaving implied.
 
 Driven through the effect summaries rather than through oxlint, which keeps this test about one thing.
 The end-to-end falsification was run separately, and it produced both halves at once:
 
 ```text
 consumer.ts:15: ... is exposed to these unresolved calls: capture-retainer-probe@1.0.0 . retainCallback
 consumer.ts:25: Parameter "config" can be deeply readonly: property row is writable.
 ```
 
 That run needed `--threads=1` at the time, because the whole external channel was then failing with
 `spawn ENOMEM` under oxlint's default worker count. That was a separate defect, since fixed by starting
 the external implementation child before oxlint reserves its per-worker buffers, and
 `external-channel-workers.unit.test.ts` guards it at the default count. This test cannot see that
 failure at all: plain `node` spawns the second child without trouble, which is exactly why the other
 test exists.
 
 @module
 */

import {
  mkdirSync,
  mkdtempSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { isFunctionLikeDeclaration, } from 'typescript/ast/is';

import {
  buildEffectSummaryIndex,
  closeSemanticBridge,
  NO_EFFECT_SUMMARY,
  openSemanticFile,
} from '../dist/final/node/index.mjs';

/**
 Authored dependency name, which must not name anything installed.
 */
const PACKAGE_NAME = 'capture-retainer-probe';

/**
 Authored dependency version, matched exactly by the fixture lockfile key.
 */
const PACKAGE_VERSION = '1.0.0';

/**
 Shipped implementation, whose effects the analyzer proves by reading this source.
 
 `retainCallback` keeps both arguments past its own return, which is what makes its summary non-empty
 and therefore what makes the external gate open at all. `stampRow` writes its formal, so a consumer
 handing it caller state is charged a proven mutation rather than opacity, and that difference is the
 fixture's own proof that the external path ran.
 
 `runCallback` invokes rather than keeps, which reaches a different branch of the formal selection:
 retention arrives as an opaque formal and invocation as an invoked one. Without it, deleting
 invocation from that selection would leave this suite green while a measured false offer returned.
 */
const IMPLEMENTATION_SOURCE = `const heldCallbacks = [];
const heldRows = [];

export function retainCallback(callback, row) {
  heldCallbacks.push(callback);
  heldRows.push(row);
  return heldCallbacks.length;
}

export function stampRow(row) {
  row.label = 'external';
}

export function retainRow(row) {
  heldRows.push(row);
  return heldRows.length;
}

export function runCallback(callback) {
  return callback();
}

export function stampAndIgnoreCallback(callback, row) {
  row.label = 'external';
  return 0;
}
`;

/**
 Shipped declarations, which are what the consumer's checker resolves each call against.
 
 `retainCallback` declares `() => void` while the consumer hands it `() => Row`. That substitution is
 legal, which is the whole reason a reading closure reaches caller state here.
 */
const DECLARATION_SOURCE = `export declare function retainCallback(callback: () => void, row: { label: string; }): number;
export declare function stampRow(row: { label: string; }): void;
export declare function retainRow(row: { label: string; }): number;
export declare function runCallback(callback: () => void): void;
export declare function stampAndIgnoreCallback(callback: () => void, row: { label: string; }): number;
`;

/**
 Second authored dependency, declaring no runtime entry and named so its root has a code suffix.
 
 Both properties are deliberate and each pins one half of the resolution it exercises. Declaring no
 `exports`, `main` or `module` is the shape `ignore@7.0.6` ships, which Node resolves by falling back to
 an index file. Ending the name in `.js` means the package root itself carries a supported suffix, so a
 resolver that accepts any existing path would hand back the directory.
 */
const INDEX_FALLBACK_NAME = 'entryless-probe.js';

/**
 Shipped implementation of the package that declares no entry.
 
 Writes its formal rather than keeping a callback, because a kept callback cannot discriminate. Opacity
 from a resolved external retention and opacity from the unresolved boundary are the same set, so a
 capture reads the same whether resolution worked or not. A proven mutation is written by the external
 path alone.
 */
const INDEX_FALLBACK_IMPLEMENTATION = `export function stampEntry(row) {
  row.label = 'entryless';
}
`;

/**
 Declarations of the package that declares no entry, reached through its `types` field.
 */
const INDEX_FALLBACK_DECLARATION = `export declare function stampEntry(row: { label: string; }): void;
`;

/**
 Consumer source, one callable per claim.
 */
const CONSUMER_SOURCE = `import { retainCallback, retainRow, runCallback, stampAndIgnoreCallback, stampRow, } from '${PACKAGE_NAME}';
import { stampEntry, } from '${INDEX_FALLBACK_NAME}';

export type Row = { label: string; };

export type Config = { row: Row; };

export function stampThroughExternal(config: Config,): void {
  stampRow(config.row,);
}

export function retainRowThroughExternal(config: Config,): number {
  return retainRow(config.row,);
}

export function proveRetainCallbackResolves(config: Config,): number {
  return retainCallback(
    function fresh(): void {
      const own: Row = { label: 'own', };
      own.label = 'changed';
    },
    config.row,
  );
}

export function handRowProducerToExternalRetainer(config: Config,): number {
  return retainCallback(
    function produce(): Row {
      return config.row;
    },
    { label: 'own', },
  );
}

export function handFreshProducerToExternalRetainer(): number {
  return retainCallback(
    function produceFresh(): Row {
      return { label: 'fresh', };
    },
    { label: 'own', },
  );
}

export function handCaptureWriterToExternalRetainer(config: Config,): number {
  return retainCallback(
    function writeRow(): void {
      config.row.label = 'external';
    },
    { label: 'own', },
  );
}

export function handRowProducerToIgnoredPosition(config: Config,): number {
  return stampAndIgnoreCallback(
    function produceIgnored(): Row {
      return config.row;
    },
    { label: 'own', },
  );
}

export function stampThroughIgnoringExport(config: Config,): number {
  return stampAndIgnoreCallback(
    function produceOwn(): Row {
      return { label: 'fresh', };
    },
    config.row,
  );
}

export function stampThroughEntrylessPackage(config: Config,): void {
  stampEntry(config.row,);
}

export function handRowProducerToExternalRunner(config: Config,): void {
  runCallback(function produceForRunner(): Row {
    return config.row;
  },);
}
`;

/**
 Disposable consumer workspace owning its own lockfile, configured project and dependency.
 */
type ExternalCaptureFixture = {
  readonly consumerPath: string;
  readonly [Symbol.dispose]: () => void;
};

/**
 Materializes the consumer workspace and the dependency it resolves against.
 
 @returns fixture removed when the enclosing scope ends.
 
 @example
 ```ts
 using fixture = externalCaptureFixture();
 ```
 */
function externalCaptureFixture(): ExternalCaptureFixture {
  /**
   Consumer root, outside every repository lockfile's reach.
   */
  const root = mkdtempSync(join(
    scratchRoot(),
    'readonly-external-capture-',
  ),);
  /**
   Installed dependency root, whose `node_modules` segment is what package identity reads.
   */
  const packageRoot = join(
    root,
    'node_modules',
    PACKAGE_NAME,
  );
  mkdirSync(
    packageRoot,
    { recursive: true, },
  );
  writeFileSync(
    join(
      packageRoot,
      'package.json',
    ),
    `${JSON.stringify(
      {
        name: PACKAGE_NAME,
        version: PACKAGE_VERSION,
        type: 'module',
        exports: {
          types: './index.d.ts',
          default: './index.js',
        },
      },
      null,
      2,
    )}\n`,
  );
  writeFileSync(
    join(
      packageRoot,
      'index.js',
    ),
    IMPLEMENTATION_SOURCE,
  );
  writeFileSync(
    join(
      packageRoot,
      'index.d.ts',
    ),
    DECLARATION_SOURCE,
  );
  /**
   Root of the dependency that declares no runtime entry, mirroring `ignore@7.0.6`.
   */
  const fallbackRoot = join(
    root,
    'node_modules',
    INDEX_FALLBACK_NAME,
  );
  mkdirSync(
    fallbackRoot,
    { recursive: true, },
  );
  /* No `exports`, no `main`, no `module`, and no `type`, which is exactly what `ignore@7.0.6` ships. Node
   * resolves such a package by the legacy rule that falls back to an index file, and `types` is what the
   * consumer's checker reads for declarations. */
  writeFileSync(
    join(
      fallbackRoot,
      'package.json',
    ),
    `${JSON.stringify(
      {
        name: INDEX_FALLBACK_NAME,
        version: PACKAGE_VERSION,
        types: './index.d.ts',
      },
      null,
      2,
    )}\n`,
  );
  writeFileSync(
    join(
      fallbackRoot,
      'index.js',
    ),
    INDEX_FALLBACK_IMPLEMENTATION,
  );
  writeFileSync(
    join(
      fallbackRoot,
      'index.d.ts',
    ),
    INDEX_FALLBACK_DECLARATION,
  );
  /* Key shape copied from this repository's `pnpm-lock.yaml`. `lockfileLinePackageKey` scans for keys at
   * two-space indentation without regard to section, so the surrounding structure is present for
   * readers rather than for the parser. */
  writeFileSync(
    join(
      root,
      'pnpm-lock.yaml',
    ),
    `lockfileVersion: '9.0'

packages:

  ${PACKAGE_NAME}@${PACKAGE_VERSION}:
    resolution: {integrity: sha512-${'A'.repeat(86,)}==}

  ${INDEX_FALLBACK_NAME}@${PACKAGE_VERSION}:
    resolution: {integrity: sha512-${'B'.repeat(86,)}==}

snapshots:

  ${PACKAGE_NAME}@${PACKAGE_VERSION}: {}

  ${INDEX_FALLBACK_NAME}@${PACKAGE_VERSION}: {}
`,
  );
  writeFileSync(
    join(
      root,
      'tsconfig.json',
    ),
    `${JSON.stringify(
      {
        compilerOptions: {
          strict: true,
          module: 'nodenext',
          moduleResolution: 'nodenext',
          target: 'esnext',
          noEmit: true,
        },
        include: ['consumer.ts',],
      },
      null,
      2,
    )}\n`,
  );
  /**
   Consumer source path, whose ancestors supply the configured project and the lockfile.
   */
  const consumerPath = join(
    root,
    'consumer.ts',
  );
  writeFileSync(
    consumerPath,
    CONSUMER_SOURCE,
  );
  return {
    consumerPath,
    [Symbol.dispose](): void {
      rmSync(
        root,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

await describe({
  name: 'external capture channel',
  concurrency: 1,
  children: [
    it({
      name: 'charges a capture handed to an external formal the implementation keeps',
      fn: async () => {
        /* The callable handing a closure to a package that keeps it was offered `readonly` before
         * `recordExternalCaptureOpacity`. That was a false offer rather than an imprecision: the rule
         * wrote `Parameter "config" can be deeply readonly: property row is writable.` under
         * `oxlint --threads=1`, and the retained closure hands back the caller's own row.
         *
         * The proof callables carry the other half, and they are why the subject means anything. If any
         * gate had rejected the authored dependency, `stampThroughExternal` would report ordinary
         * unresolved opacity instead of a proven mutation, and `proveRetainCallbackResolves` would carry
         * no package provenance. Those two shapes separate "the hole is closed" from "my fixture never
         * reached the code", and a fixture that stopped reaching it would otherwise look like a pass. */
        using fixture = externalCaptureFixture();
        /**
         Consumer session, whose configured project is the fixture's own.
         */
        const session = openSemanticFile({
          fileName: fixture.consumerPath,
          sourceText: CONSUMER_SOURCE,
          hasBOM: false,
        },);
        /**
         Effects over the consumer, resolving the authored dependency through the external channel.
         */
        const index = buildEffectSummaryIndex({
          project: session.project,
          activeSourceFile: session.sourceFile,
        },);
        /**
         Reads one index set of one consumer callable.
         
         @param functionName - Consumer callable to inspect.
         
         @param read - Which index set to take off the summary.
         
         @returns those parameter indexes in ascending order.
         
         @throws when the name does not resolve to a callable carrying a summary.
         
         @example
         ```ts
         consumerIndexes({ functionName: 'stampThroughExternal', read: opaqueOf });
         ```
         */
        function consumerIndexes({
          functionName,
          read,
        }: {
          readonly functionName: string;
          readonly read: (summary: {
            readonly referentMutatedParameterIndexes: Iterable<number>;
            readonly opaqueParameterIndexes: Iterable<number>;
          },) => Iterable<number>;
        },): readonly number[] {
          /**
           Name node of the requested consumer declaration.
           */
          const nameNode = session.nodeAtOffset(
            CONSUMER_SOURCE.indexOf(`function ${functionName}`,)
              + 'function '.length,
          );
          /**
           Declaration owning that name.
           */
          const declaration = nameNode.parent;
          if (!isFunctionLikeDeclaration(declaration,))
            throw new Error(`Expected a declaration for ${functionName}.`,);
          /**
           Effect summary for that declaration.
           */
          const summary = index.get(declaration,);
          if (summary === NO_EFFECT_SUMMARY)
            throw new Error(`Expected an effect summary for ${functionName}.`,);
          return [...read(summary,),]
            .toSorted(function byIndex(left: number, right: number,): number {
              return left - right;
            },);
        }
        /**
         Reads the opaque parameter indexes of one consumer callable.
         
         @param functionName - Consumer callable to inspect.
         
         @returns opaque parameter indexes in ascending order.
         
         @example
         ```ts
         consumerOpaque('handRowProducerToExternalRetainer');
         ```
         */
        function consumerOpaque(functionName: string,): readonly number[] {
          return consumerIndexes({
            functionName,
            read: function opaqueOf(summary,): Iterable<number> {
              return summary.opaqueParameterIndexes;
            },
          },);
        }
        /**
         Reads the written parameter indexes of one consumer callable.
         
         @param functionName - Consumer callable to inspect.
         
         @returns written parameter indexes in ascending order.
         
         @example
         ```ts
         consumerWritten('stampThroughExternal');
         ```
         */
        function consumerWritten(functionName: string,): readonly number[] {
          return consumerIndexes({
            functionName,
            read: function writtenOf(summary,): Iterable<number> {
              return summary.referentMutatedParameterIndexes;
            },
          },);
        }
        /** Proven external mutation of a formal the consumer fed caller state. */
        const stamped = consumerWritten('stampThroughExternal',);
        /** Opacity from the same call, which must be absent for the mutation to be the proven kind. */
        const stampedOpaque = consumerOpaque('stampThroughExternal',);
        /** Proven external retention of a row passed directly. */
        const retainedRow = consumerOpaque('retainRowThroughExternal',);
        /** Proven external retention through the same export the capture cases use. */
        const retainerReached = consumerOpaque('proveRetainCallbackResolves',);
        /** Capture handed to the retaining export inside a closure that reads it. */
        const producedCapture = consumerOpaque('handRowProducerToExternalRetainer',);
        /** Closure allocating its own row, which must never be charged. */
        const freshCapture = consumerOpaque('handFreshProducerToExternalRetainer',);
        /** Capture inside a closure that writes it rather than handing it back. */
        const writtenCapture = consumerWritten('handCaptureWriterToExternalRetainer',);
        /** Capture handed to an export that invokes rather than keeps. */
        const invokedCapture = consumerOpaque('handRowProducerToExternalRunner',);
        /** Capture handed to a formal the implementation neither invokes, keeps, nor writes through. */
        const ignoredCapture = consumerOpaque('handRowProducerToIgnoredPosition',);
        /** Proven mutation by that same export, of the formal it does use. */
        const ignoringExportStamped = consumerWritten('stampThroughIgnoringExport',);
        /** Proven mutation by a package whose manifest declares no runtime entry at all. */
        const entrylessStamped = consumerWritten('stampThroughEntrylessPackage',);
        /** Opacity from that same call, absent only when the implementation really was inspected. */
        const entrylessOpaque = consumerOpaque('stampThroughEntrylessPackage',);
        closeSemanticBridge();
        /* The gates opened. A proven mutation arrives through the external summary, so this parameter is
         * written rather than merely unknown, and no unresolved boundary was recorded beside it. Both
         * halves matter: the write alone would also appear if the fixture's own direct-write scan had
         * produced it, and the empty opacity is what rules that out. */
        expect(stamped,).toEqual([0,],);
        expect(stampedOpaque,).toEqual([],);
        /* Proven external retention, charged from the shipped implementation's own store into a module
         * binding. Under oxlint these two carry `capture-retainer-probe@1.0.0` in the message, which is
         * provenance only `applyExternalEffect` writes. */
        expect(retainedRow,).toEqual([0,],);
        /* The per-export proof. This is the same `retainCallback` the two capture cases call, so it
         * settles that the gate opens for that exact export rather than merely for the package. */
        expect(retainerReached,).toEqual([0,],);
        /* The subject. `applyExternalEffect` maps external formals onto caller origins taken from
         * argument positions, and a closure argument carries no origin of its own, so a capture inside
         * one was charged by nothing: `recordOpaqueBoundary` is what charges captures, and the external
         * branch returns before reaching it. Measured `[]` before the channel existed. */
        expect(producedCapture,).toEqual([0,],);
        /* The control, and the one that decides this is an attribution rather than a rule against
         * handing any closure to an external callee. The same export, the same position, a closure
         * allocating its own row: nothing captured travels, so the offer stands. */
        expect(freshCapture,).toEqual([],);
        /* The shape that is already covered, and the reason the falsification needed a reading closure
         * rather than a writing one. A write inside the closure is attributed by the direct-write scan
         * whatever the callee is, so it never reaches the capture channel and cannot expose its
         * absence. */
        expect(writtenCapture,).toEqual([0,],);
        /* The second branch of the formal selection, and it needs its own case rather than inheriting
         * the subject's. Retention reaches an opaque formal and invocation reaches an invoked one, so
         * deleting invocation from `exposingFormals` leaves every other line here passing while this
         * false offer returns. Measured as a false offer in its own right: this reported
         * `Parameter "config" can be deeply readonly` under `oxlint --threads=1` before the channel
         * existed. */
        expect(invokedCapture,).toEqual([0,],);
        /* The precision the per-formal charge exists for, and the pair that would fail if captures were
         * charged for every argument instead. `stampAndIgnoreCallback` writes its second formal and does
         * nothing whatever with its first, so a closure handing back caller state in that first position
         * exposes nothing and the parameter keeps its offer.
         *
         * The second line is what makes the first mean something. An empty result is otherwise
         * indistinguishable from a gate that rejected the dependency, and the proven mutation says the
         * same export resolved and applied. */
        expect(ignoredCapture,).toEqual([],);
        expect(ignoringExportStamped,).toEqual([0,],);
        /* The package that declares no `exports`, `main` or `module`, which is the shape `ignore@7.0.6`
         * ships and the one non-builtin package that ever reached implementation resolution and failed.
         * `manifestRuntimeTarget` declined before the directory-index fallback could run, so nothing about
         * this package was ever inspected.
         *
         * A proven mutation is what says the fallback worked, and a capture would not have said it. The
         * first shape written here handed this package a closure and asserted the capture was charged; that
         * assertion passed with the fix reverted, because the unresolved boundary charges captures too and
         * both paths write the same set. Two mutants survived it. This pair cannot be satisfied that way:
         * an unresolved call records opacity and no mutation, and a resolved one records the mutation its
         * shipped implementation performs.
         *
         * The package name ends in `.js` on purpose, so its root directory carries a supported suffix. A
         * resolver accepting any existing path returns that directory, nothing loads from it, and this pair
         * fails. Both halves of the resolution are pinned by these two lines. */
        expect(entrylessStamped,).toEqual([0,],);
        expect(entrylessOpaque,).toEqual([],);
      },
    },),
  ],
},);
