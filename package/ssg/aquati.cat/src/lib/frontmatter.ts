/**
 YAML frontmatter fence scanner.
 
 Splits a raw MDX file into its YAML frontmatter block and the remaining body.
 The fences are strict: three dashes must end their line, on both the opening
 and the closing side, so a longer thematic break never opens a block.
 
 Uses a hand-written scanner instead of `gray-matter` because that dependency
 tree is large and one of its YAML loaders uses `eval`.
 */

import { parse as parseYaml, } from 'yaml';

//region Types

/**
 Raw frontmatter parse result: YAML data plus the remaining body.
 */
export type ParsedFrontmatter = {
  readonly data: Record<string, unknown>;
  readonly content: string;
};

//endregion Types

//region Fence scanning

/**
 Opening delimiter for YAML frontmatter blocks.
 */
const FRONTMATTER_OPEN = '---';

/**
 Unicode Byte Order Mark code point, stripped from file content before parsing.
 */
const BOM = 0xFE_FF;

/**
 Parses YAML frontmatter delimited by `---` from a raw string.
 
 Splits at the first two `---` lines to extract the YAML block and body.
 Returns empty data when no valid frontmatter is found.
 
 Exporting this parser lets tests drive the fence scanner directly, including
 the inputs no MDX file in the tree happens to carry.
 
 @param raw - full file content including frontmatter
 
 @returns parsed YAML data and remaining body content
 
 @example
 ```ts
 const { data, content } = parseFrontmatter('---\ntitle: Hello\n---\nbody');
 // data = { title: 'Hello' }, content = 'body'
 ```
 */
export function parseFrontmatter(raw: string,): ParsedFrontmatter {
  /* Strip optional leading BOM. */
  /**
   BOM-trimmed input used for every subsequent index computation.
   */
  const str = raw.codePointAt(0,)
    === BOM ? raw.slice(1,) : raw;

  if (!str.startsWith(FRONTMATTER_OPEN,)) {
    return {
      data: {},
      content: str,
    };
  }

  /**
   Length of the opening fence, used as the offset of the character after it.
   */
  const fenceLength = FRONTMATTER_OPEN.length;
  /**
   Character right after the opening fence; the fence is valid only when the
   line ends there, matching the closing-fence rule below. Without this guard a
   document starting with a longer thematic break such as `----` opened a
   frontmatter block and had its body swallowed as YAML.
   */
  const charAfterOpen = str.charAt(fenceLength,);
  /**
   Whether the opening fence ended its line, which the closing-fence rule below mirrors.
   */
  const openingFenceEndsLine = (charAfterOpen === '\n')
    || (charAfterOpen === '\r')
    || (charAfterOpen === '');
  if (!openingFenceEndsLine) {
    return {
      data: {},
      content: str,
    };
  }

  /* Skip past the opening `---` and its trailing newline. */
  /**
   Index of the first newline after the opening fence, or `-1` when malformed.
   */
  const afterOpen = str.indexOf(
    '\n',
    FRONTMATTER_OPEN.length,
  );
  if (afterOpen === (-1)) {
    return {
      data: {},
      content: str,
    };
  }

  /**
   Scan for the closing `---` that sits at the start of a line.
   Start searching from the character right after the first newline.
   */
  /**
   Starting offset for the closing-fence scan; first character after the opening newline.
   */
  const searchFrom = afterOpen + 1;
  /**
   Cursor advanced through the loop while hunting for a column-zero closing fence.
   */
  // oxlint-disable-next-line no-restricted-syntax/no-function-root-let -- parser cursor advanced across multiple loop iterations
  let closeStart = searchFrom;

  for (;;) {
    const idx = str.indexOf(
      FRONTMATTER_OPEN,
      closeStart,
    );
    if (idx === (-1)) {
      return {
        data: {},
        content: str,
      };
    }

    /* The delimiter must be at column 0 or immediately after a newline. */
    if ((idx === 0) || (str[idx - 1]
      === '\n')) {
      /**
       Offset just past the closing fence; the next char must be newline or EOF for a valid close.
       */
      const afterDelim = idx + FRONTMATTER_OPEN
        .length;

      /* Next char must be a newline or EOF for a valid closing fence. */
      if (
        (afterDelim === str
          .length)
          || (str[afterDelim]
            === '\n')
          || (str[afterDelim]
            === '\r')
      ) {
        /**
         YAML body between the opening and closing fences fed to the `yaml` package's `parse`.
         */
        const yamlBlock = str.slice(
          searchFrom,
          idx,
        );
        /**
         Body start cursor advanced past CR/LF so the post body excludes the closing fence.
         */
        let bodyStart = afterDelim;
        if (str[bodyStart]
          === '\r')
          bodyStart += 1;
        if (str[bodyStart]
          === '\n')
          bodyStart += 1;

        return {
          // oxlint-disable-next-line no-unsafe-type-assertion -- `yaml`'s `parse` returns `any` for untyped frontmatter; the assertion narrows to the documented record contract, re-validated downstream by `postFileFrontmatterSchema` in `loadContent`.
          data: (parseYaml(yamlBlock,)
            ?? {}) as Record<string, unknown>,
          content: str.slice(bodyStart,),
        };
      }
    }

    closeStart = idx + 1;
  }
}

//endregion Fence scanning
