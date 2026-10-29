/**
 Verification script for Figma Kiwi parser fixtures.
 
 @example
 ```bash
 node package/figma/kiwi/src/verify.ts
 ```
 */

import { caughtValueText as caughtErrorMessage, } from '@monochromatic-dev/caught-value-core/ts';

import {
  FIGMA_DOCUMENT_ABSENT,
  parseFigmaFile,
} from './index.ts';

/**
 Reference directory holding hand-curated Figma exports used as integration fixtures.
 */
const SOURCE_DIR = '/home/user/Nextcloud/Text/Reference/Figma export';

/**
 Number of sample nodes printed from each decoded document.
 */
const NODE_PREVIEW_COUNT = 3;

/**
 Fixture catalogue covering every Figma file extension the parser must decode.
 */
const TEST_FILES = [
  {
    name: 'Color palette - base.fig',
    type: 'fig',
  },
  {
    name: 'MTM6162-040 participation 2 cover.deck',
    type: 'deck',
  },
  {
    name: 'Todo app - Brainstorming.jam',
    type: 'jam',
  },
] as const;

/**
 Result of verifying one fixture.
 
 @example
 ```ts
 const result: VerifyResult = { ok: true };
 ```
 */
type VerifyResult = {
  readonly ok: boolean;
};

/**
 Verifies every fixture and prints summary.
 
 @example
 ```ts
 await main();
 ```
 */
async function main(): Promise<void> {
  /**
   Per-fixture verification results.
   */
  const results = await Promise.all(TEST_FILES.map(function verifyFixture(test,): Promise<VerifyResult> {
    return verifyOne({ test, },);
  },),);
  /**
   Count of passing fixtures.
   */
  const passed = results.filter(function passedFixture(result,): boolean {
    return result.ok;
  },)
    .length;
  /**
   Count of failing fixtures.
   */
  const failed = results.length - passed;

  console.log(`\n=== Results: ${passed} passed, ${failed} failed ===`,);
  if (failed > 0)
    throw new Error(`${failed} Figma Kiwi fixture verification failed`,);
}

/**
 Verifies one fixture.
 
 @param test - Fixture descriptor.
 
 @returns Verification result.
 
 @example
 ```ts
 await verifyOne({ test: TEST_FILES[0] });
 ```
 */
async function verifyOne(
  { test, }: { readonly test: (typeof TEST_FILES)[number]; },
): Promise<VerifyResult> {
  /**
   Absolute fixture path.
   */
  const path = `${SOURCE_DIR}/${test.name}`;
  console.log(`\n=== Testing: ${test.name} ===`,);

  try {
    /**
     Parsed fixture.
     */
    const file = await parseFigmaFile(path,);
    if (file.fileType !== test.type)
      return fail({ message: `expected type "${test.type}", got "${file.fileType}"`, },);

    console.log(`  Type: ${file.fileType}`,);
    logSchemaSummary({ file, },);

    /**
     NodeType enum lookup.
     */
    const nodeType = file.schema
      .enumByName
      .get('NodeType',);
    if (nodeType === undefined)
      return fail({ message: 'NodeType enum not found', },);
    /**
     DOCUMENT enum field.
     */
    const docField = nodeType.fields
      .find(function documentField(field,): boolean {
        return field.name === 'DOCUMENT';
      },);
    if ((docField === undefined) || (docField.value !== 1))
      return fail({ message: `NodeType.DOCUMENT not found or wrong value: ${docField?.value}`, },);

    if (file.document === FIGMA_DOCUMENT_ABSENT)
      return fail({ message: 'document is absent', },);

    return verifyDocument({
      document: file.document,
      imageCount: file.images
        .size,
      fileName: file.meta
        .fileName,
    },);
  }
  catch (error) {
    console.error(`  FAIL: ${caughtErrorMessage(error,)}`,);
    return { ok: false, };
  }
}

/**
 Logs schema summary.
 
 @param file - Parsed file wrapper.
 
 @example
 ```ts
 logSchemaSummary({ file });
 ```
 */
function logSchemaSummary(
  { file, }: {
    readonly file: {
      readonly schema: {
        readonly definitions: readonly { readonly kind: string; }[];
      };
    };
  },
): void {
  /**
   Total definition count.
   */
  const totalDefs = file.schema
    .definitions
    .length;
  /**
   Enum definition count.
   */
  const enums = file.schema
    .definitions
    .filter(function isEnum(definition,): boolean {
      return definition.kind === 'ENUM';
    },)
    .length;
  /**
   Struct definition count.
   */
  const structs = file.schema
    .definitions
    .filter(function isStruct(definition,): boolean {
      return definition.kind === 'STRUCT';
    },)
    .length;
  /**
   Message definition count.
   */
  const messages = file.schema
    .definitions
    .filter(function isMessage(definition,): boolean {
      return definition.kind === 'MESSAGE';
    },)
    .length;
  console.log(`  Schema: ${totalDefs} definitions (${enums} enums, ${structs} structs, ${messages} messages)`,);
}

/**
 Verifies decoded document shape.
 
 @param document - Decoded document record.
 
 @param imageCount - Image count.
 
 @param fileName - File name from metadata.
 
 @returns Verification result.
 
 @example
 ```ts
 verifyDocument({ document: { type: 'MessageType.NODE_CHANGES', nodeChanges: [] }, imageCount: 0, fileName: 'demo' });
 ```
 */
function verifyDocument(
  {
    document,
    imageCount,
    fileName,
  }: {
    readonly document: Readonly<Record<string, unknown>>;
    readonly fileName: string;
    readonly imageCount: number;
  },
): VerifyResult {
  if (document.type !== 'MessageType.NODE_CHANGES')
    return fail({ message: `expected MessageType.NODE_CHANGES, got ${caughtErrorMessage(document.type,)}`, },);
  /**
   Candidate node changes value.
   */
  const { nodeChanges, } = document;
  if (!Array.isArray(nodeChanges,))
    return fail({ message: 'nodeChanges not found or not an array', },);
  console.log(`  Document: ${nodeChanges.length} node changes`,);
  logNodePreview({ nodeChanges, },);
  console.log(`  Images: ${imageCount}`,);
  console.log(`  Meta: fileName="${fileName}"`,);
  return { ok: true, };
}

/**
 Logs first decoded node records.
 
 @param nodeChanges - Node changes array.
 
 @example
 ```ts
 logNodePreview({ nodeChanges: [] });
 ```
 */
function logNodePreview({ nodeChanges, }: { readonly nodeChanges: readonly unknown[]; },): void {
  for (let loopIndex = 0; loopIndex < Math.min(
    NODE_PREVIEW_COUNT,
    nodeChanges.length,
  ); loopIndex++) {
    /**
     Current node-change value.
     */
    const nodeChange = nodeChanges[loopIndex];
    if (!isRecord(nodeChange,))
      continue;
    /**
     Display name.
     */
    const name = stringField({
      record: nodeChange,
      key: 'name',
    },);
    /**
     Display type.
     */
    const type = stringField({
      record: nodeChange,
      key: 'type',
    },);
    console.log(`    Node ${loopIndex + 1}: type=${type} name="${name}"`,);
  }
}

/**
 Logs failure and returns failed result.
 
 @param message - Failure message.
 
 @returns Failed verification result.
 
 @example
 ```ts
 fail({ message: 'bad' });
 // { ok: false }
 ```
 */
function fail({ message, }: { readonly message: string; },): VerifyResult {
  console.error(`  FAIL: ${message}`,);
  return { ok: false, };
}

/**
 Returns whether value is a non-array record.
 
 @param value - Candidate value.
 
 @returns Whether value is a record.
 
 @example
 ```ts
 isRecord({});
 // true
 ```
 */
function isRecord(value: unknown,): value is Readonly<Record<string, unknown>> {
  return ((typeof value) === 'object')
    && (value !== null)
    && (!Array.isArray(value,));
}

/**
 Reads string field from record.
 
 @param record - Source record.
 
 @param key - Field key.
 
 @returns String field or question mark fallback.
 
 @example
 ```ts
 stringField({ record: { name: 'demo' }, key: 'name' });
 // 'demo'
 ```
 */
function stringField(
  {
    record,
    key,
  }: {
    readonly key: string;
    readonly record: Readonly<Record<string, unknown>>;
  },
): string {
  /**
   Field value.
   */
  const value = record[key];
  return (typeof value) === 'string' ? value : '?';
}

try {
  await main();
}
catch (error) {
  console.error(caughtErrorMessage(error,),);
  throw error;
}
