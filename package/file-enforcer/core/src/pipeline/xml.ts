import { parser, } from '@lezer/xml';

import { unescapeXmlAttribute, } from './xml-coding.ts';

import type {
  SyntaxNode,
  Tree,
} from '@lezer/common';

//region Shapes and sentinels: entries and absence markers

/**
 Entry block found inside a JetBrains persistent-state XML map.
 */
export type XmlEntry = {
  readonly block: string;
  readonly end: number;
  readonly key: string;
  readonly start: number;
};

/**
 Sentinel returned by attribute and option readers when the value is absent.
 */
export const ABSENT_XML_VALUE: unique symbol = Symbol('file-enforcer/pipeline/xml: requested setting has no associated text in parsed markup',);

/**
 Sentinel returned by entry lookups when no map entry matches the key.
 */
export const ABSENT_XML_ENTRY: unique symbol = Symbol('file-enforcer/pipeline/xml: no map entry matches the requested key',);

/**
 Sentinel returned when an element has no open or self-closing tag.
 */
const ABSENT_ELEMENT_HEAD: unique symbol = Symbol('file-enforcer/pipeline/xml: element has no open or self-closing tag',);

//endregion Shapes and sentinels

//region Syntax tree access: read tags and attributes regardless of formatting

/**
 Returns an element's open or self-closing tag node.
 
 @param element - XML element syntax node.
 
 @returns Tag node carrying name and attributes, or {@link ABSENT_ELEMENT_HEAD}.
 
 @example
 ```ts
 elementHead({ element });
 ```
 */
function elementHead(
  { element, }: { readonly element: SyntaxNode; },
): SyntaxNode | typeof ABSENT_ELEMENT_HEAD {
  /**
   Open or self-closing tag node, if present.
   */
  const head = element.getChild('OpenTag',) ?? element.getChild('SelfClosingTag',);
  return head ?? ABSENT_ELEMENT_HEAD;
}

/**
 Reads an element's tag name from source, locating the tag via {@link elementHead}.
 
 @param element - XML element syntax node.
 
 @param source - Document text the node indexes into.
 
 @returns Tag name, or empty string when the element has no readable tag.
 
 @example
 ```ts
 tagNameOf({ element, source });
 ```
 */
function tagNameOf(
  {
    element,
    source,
  }: {
    readonly element: SyntaxNode;
    readonly source: string
  },
): string {
  /**
   Open or self-closing tag carrying the element name.
   */
  const head = elementHead({ element, },);
  if (head === ABSENT_ELEMENT_HEAD) return '';
  /**
   Tag-name node, if present.
   */
  const tagName = head.getChild('TagName',);
  return tagName === null ? '' : source.slice(
    tagName.from,
    tagName.to,
  );
}

/**
 Strips surrounding quote characters from an attribute value token.
 
 @param raw - Attribute value source text including any quote characters.
 
 @returns Quote-free attribute value, entity-decoded via {@link unescapeXmlAttribute}.
 
 @example
 ```ts
 decodeAttributeValue({ raw: '"a &amp; b"' });
 ```
 */
function decodeAttributeValue({ raw, }: { readonly raw: string; },): string {
  /**
   Whether the token is wrapped in matching quote characters.
   */
  const quoted = (raw.length >= 2) && ((raw.startsWith('"',)) || (raw.startsWith("'",)));
  /**
   Attribute text without surrounding quotes.
   */
  const inner = quoted ? raw.slice(
    1,
    -1,
  ) : raw;
  return unescapeXmlAttribute({ value: inner, },);
}

/**
 Reads one attribute value from an element by name, ignoring attribute order,
 locating the tag via {@link elementHead}.
 
 @param element - XML element syntax node.
 
 @param source - Document text the node indexes into.
 
 @param name - Attribute name to read.
 
 @returns Attribute value decoded via {@link decodeAttributeValue}, or {@link ABSENT_XML_VALUE} when absent.
 
 @example
 ```ts
 elementAttribute({ element, source, name: 'key' });
 ```
 */
function elementAttribute(
  {
    element,
    source,
    name,
  }: {
    readonly element: SyntaxNode;
    readonly name: string;
    readonly source: string
  },
): string | typeof ABSENT_XML_VALUE {
  /**
   Open or self-closing tag carrying attributes.
   */
  const head = elementHead({ element, },);
  if (head === ABSENT_ELEMENT_HEAD) return ABSENT_XML_VALUE;
  /**
   Attribute node whose name matches, if any.
   */
  const match = head.getChildren('Attribute',)
    .find(function nameMatches(attribute,): boolean {
      /**
       Attribute-name node, if present.
       */
      const attributeName = attribute.getChild('AttributeName',);
      return (attributeName !== null) && (source.slice(
        attributeName.from,
        attributeName.to,
      ) === name);
    },);
  if (match === undefined) return ABSENT_XML_VALUE;
  /**
   Attribute-value node including surrounding quotes, if present.
   */
  const attributeValue = match.getChild('AttributeValue',);
  if (attributeValue === null) return ABSENT_XML_VALUE;
  return decodeAttributeValue({ raw: source.slice(
    attributeValue.from,
    attributeValue.to,
  ), },);
}

/**
 Collects every element with a given tag name, in document order, matching
 names via {@link tagNameOf}.
 
 @param tree - Parsed syntax tree.
 
 @param source - Document text the tree indexes into.
 
 @param tagName - Tag name to match.
 
 @returns Matching element nodes.
 
 @example
 ```ts
 elementsByTag({ tree, source, tagName: 'entry' });
 ```
 */
function elementsByTag(
  {
    tree,
    source,
    tagName,
  }: {
    readonly source: string;
    readonly tagName: string;
    readonly tree: Tree
  },
): readonly SyntaxNode[] {
  /**
   Elements whose tag name matches, in document order.
   */
  const matches: SyntaxNode[] = [];
  tree.iterate({
    enter(reference,): boolean {
      if ((reference.name === 'Element') && (tagNameOf({
        element: reference.node,
        source,
      },) === tagName)) {
        matches.push(reference.node,);
      }
      return true;
    },
  },);
  return matches;
}

/**
 Backs up over inline whitespace to the start of a position's line indentation.
 
 @param source - Document text.
 
 @param position - Position to back up from.
 
 @returns Index of the first whitespace character of the line.
 
 @example
 ```ts
 indentStart({ source, position });
 ```
 */
function indentStart({
  source,
  position,
}: {
  readonly position: number;
  readonly source: string
},): number {
  /**
   Cursor walked back over inline whitespace.
   */
  let index = position;
  while ((index > 0) && ((source[index - 1] === ' ') || (source[index - 1] === '\t'))) index -= 1;
  return index;
}

//endregion Syntax tree access

//region XML entry editing: locate entries, options, and insertion points

/**
 Lists JetBrains persistent-state map entries that carry a key attribute,
 found via {@link elementsByTag} and read via {@link elementAttribute}.
 
 @param xml - XML document text.
 
 @returns Entry ranges and decoded keys, in document order.
 
 @example
 ```ts
 listXmlEntries({ xml });
 ```
 */
export function listXmlEntries({ xml, }: { readonly xml: string; },): readonly XmlEntry[] {
  /**
   Parsed syntax tree for the document.
   */
  const tree = parser.parse(xml,);
  return elementsByTag({
    tree,
    source: xml,
    tagName: 'entry',
  },)
    .map(function toEntry(element,): XmlEntry | typeof ABSENT_XML_ENTRY {
      /**
       Decoded key attribute, or the absent sentinel.
       */
      const key = elementAttribute({
        element,
        source: xml,
        name: 'key',
      },);
      if (key === ABSENT_XML_VALUE) return ABSENT_XML_ENTRY;
      return {
        block: xml.slice(
          element.from,
          element.to,
        ),
        end: element.to,
        key,
        start: element.from,
      };
    },)
    .filter(function keepEntry(entry,): entry is XmlEntry {
      return entry !== ABSENT_XML_ENTRY;
    },);
}

/**
 Finds one XML map entry by key, searching entries from {@link listXmlEntries}.
 
 @param xml - XML document text.
 
 @param key - Entry key to find.
 
 @returns Matching entry, or {@link ABSENT_XML_ENTRY}.
 
 @example
 ```ts
 findXmlEntryByKey({ xml, key: 'server' });
 ```
 */
export function findXmlEntryByKey(
  {
    xml,
    key,
  }: {
    readonly key: string;
    readonly xml: string
  },
): XmlEntry | typeof ABSENT_XML_ENTRY {
  /**
   First entry whose key matches, if any.
   */
  const found = listXmlEntries({ xml, },)
    .find(function keyMatches(entry,): boolean {
      return entry.key === key;
    },);
  return found ?? ABSENT_XML_ENTRY;
}

/**
 Reads an option element's value by name from an entry block, ignoring order,
 found via {@link elementsByTag} and read via {@link elementAttribute}.
 
 @param block - XML block containing option elements.
 
 @param optionName - Option name attribute to find.
 
 @returns Decoded option value, or {@link ABSENT_XML_VALUE}.
 
 @example
 ```ts
 getXmlOptionValue({ block, optionName: 'commandLine' });
 ```
 */
export function getXmlOptionValue(
  {
    block,
    optionName,
  }: {
    readonly block: string;
    readonly optionName: string
  },
): string | typeof ABSENT_XML_VALUE {
  /**
   Parsed syntax tree for the block.
   */
  const tree = parser.parse(block,);
  /**
   First option element whose name attribute matches, if any.
   */
  const option = elementsByTag({
    tree,
    source: block,
    tagName: 'option',
  },)
    .find(function optionNameMatches(element,): boolean {
      return elementAttribute({
        element,
        source: block,
        name: 'name',
      },) === optionName;
    },);
  if (option === undefined) return ABSENT_XML_VALUE;
  return elementAttribute({
    element: option,
    source: block,
    name: 'value',
  },);
}

/**
 Replaces an XML map entry (located via {@link findXmlEntryByKey}) if present,
 otherwise inserts before the last map element's (found via {@link elementsByTag})
 close tag. Locating splice points by parsed structure (not literal indentation,
 trimmed via {@link indentStart}) keeps edits stable across IntelliJ reformatting;
 replacement drops the existing line indentation so repeated runs do not accrete
 leading whitespace.
 
 @param xml - XML document text.
 
 @param key - Entry key to replace.
 
 @param block - Full entry block to insert.
 
 @returns Updated XML document text.
 
 @throws Error when the document has no map element to insert into.
 
 @example
 ```ts
 replaceOrInsertXmlEntry({ xml, key, block });
 ```
 */
export function replaceOrInsertXmlEntry(
  {
    xml,
    key,
    block,
  }: {
    readonly block: string;
    readonly key: string;
    readonly xml: string
  },
): string {
  /**
   Existing entry with this key, or the absent sentinel.
   */
  const existing = findXmlEntryByKey({
    xml,
    key,
  },);
  if (existing !== ABSENT_XML_ENTRY) {
    /**
     Start of the existing entry's line, dropping its indentation.
     */
    const replaceAt = indentStart({
      source: xml,
      position: existing.start,
    },);
    return `${xml.slice(
      0,
      replaceAt,
    )}${block}${xml.slice(existing.end,)}`;
  }
  /**
   Parsed syntax tree for the document.
   */
  const tree = parser.parse(xml,);
  /**
   Last map element in the document, if any.
   */
  const lastMap = elementsByTag({
    tree,
    source: xml,
    tagName: 'map',
  },)
    .at(-1,);
  if (lastMap === undefined) throw new Error('Could not find a JetBrains XML map element',);
  /**
   Close tag node of the last map, if present.
   */
  const mapClose = lastMap.getChild('CloseTag',);
  if (mapClose === null) throw new Error('Could not find the JetBrains XML map close tag',);
  /**
   Insertion point at the start of the map close tag's indentation.
   */
  const insertAt = indentStart({
    source: xml,
    position: mapClose.from,
  },);
  return `${xml.slice(
    0,
    insertAt,
  )}${block}\n${xml.slice(insertAt,)}`;
}

//endregion XML entry editing
