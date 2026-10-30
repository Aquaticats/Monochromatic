import { join } from 'node:path';

import {
  alphanumericCharacters,
  hyphenateOutside,
  startsThenUses,
  wordStartCharacters,
} from '@monochromatic-dev/shell-argument-core/ts/character-set.ts';
import {
  imageTagCharacters,
  logNameCharacters,
  maximumImageTagLength,
  maximumUnitNameLength,
  tagIndexPlaceholder,
  tagLabelPlaceholder,
  unitNameCharacters,
} from './constants.ts';
import { CargoMutantsToolingError } from './error.ts';

/**
 Rust source suffix removed from a scope to get its stem.
 */
const rustSuffix = '.rs';

/**
 Remove the Rust suffix from a scope.

 @param file - source file, relative to the scope prefix
 
 @returns the file without a trailing `.rs`
 
 @example
 ```ts
 stemOf('lock.rs');
 ```
 */
export function stemOf(file: string): string {
  return file.endsWith(rustSuffix) ? file.slice(
    0,
    -rustSuffix.length
  ) : file;
}

/**
 Build the name of the systemd unit that runs one file's campaign.

 Characters outside letters, digits and hyphens become hyphens, so a file name with a path separator, space, quote or
 newline still gives a valid unit name; the caller rejects two files that collapse to one name.

 @param prefix - unit name prefix
 
 @param label - campaign label
 
 @param file - source file, relative to the scope prefix
 
 @returns the unit name without the `.service` suffix
 
 @throws CargoMutantsToolingError when the name would exceed the systemd length limit
 
 @example
 ```ts
 buildUnitName({ prefix: 'mutpar', label: 'w5', file: 'lock.rs' });
 ```
 */
export function buildUnitName({
  prefix,
  label,
  file,
}: {
  readonly prefix: string;
  readonly label: string;
  readonly file: string;
}): string {
  /**
   Unit name before the length check.
   */
  const unit = hyphenateOutside({
    text: `${prefix}-${label}-${stemOf(file)}`,
    allowed: unitNameCharacters,
  });
  if (unit.length > maximumUnitNameLength)
    throw new CargoMutantsToolingError(
      `The unit name for ${JSON.stringify(file)} is ${unit.length} characters, over the limit of ${maximumUnitNameLength}; use a shorter label or prefix.`,
    );
  return unit;
}

/**
 Build the path prefix of one file's attempt logs; attempt `n` appends `.n`.

 @param directory - log directory
 
 @param prefix - log file name prefix
 
 @param label - campaign label
 
 @param file - source file, relative to the scope prefix
 
 @returns `<directory>/<prefix>-<label>-<stem>.log`
 
 @example
 ```ts
 buildLogBase({ directory: '/logs', prefix: 'mutpar', label: 'w5', file: 'lock.rs' });
 ```
 */
export function buildLogBase({
  directory,
  prefix,
  label,
  file,
}: {
  readonly directory: string;
  readonly prefix: string;
  readonly label: string;
  readonly file: string;
}): string {
  return join(
    directory,
    `${prefix}-${label}-${hyphenateOutside({
      text: stemOf(file),
      allowed: logNameCharacters
    })}.log`,
  );
}

/**
 Build the image tag of one file's campaign from the template.

 @param template - tag template holding the index placeholder and, usually, the label placeholder
 
 @param label - campaign label
 
 @param index - zero-based position of the file in the argument list
 
 @returns the tag
 
 @throws CargoMutantsToolingError when the template lacks the index placeholder or the tag is not a valid image tag
 
 @example
 ```ts
 buildImageTag({ template: '{label}{index}', label: 'w5', index: 2 });
 ```
 */
export function buildImageTag({
  template,
  label,
  index,
}: {
  readonly template: string;
  readonly label: string;
  readonly index: number;
}): string {
  if (!template.includes(tagIndexPlaceholder))
    throw new CargoMutantsToolingError(
      `The tag template ${JSON.stringify(template)} must contain ${tagIndexPlaceholder}, or concurrent campaigns would share one image tag.`,
    );
  /**
   Template with both placeholders replaced.
   */
  const tag = template
    .replaceAll(
    tagLabelPlaceholder,
    label
    )
    .replaceAll(
    tagIndexPlaceholder,
    String(index)
    );
  if ((tag.length > maximumImageTagLength) || (!startsThenUses({
    text: tag,
    first: alphanumericCharacters + wordStartCharacters,
    rest: imageTagCharacters,
  })))
    throw new CargoMutantsToolingError(
      `The image tag ${JSON.stringify(tag)} is not valid: use up to ${maximumImageTagLength} letters, digits, underscores, dots and hyphens, not starting with a dot or hyphen.`,
    );
  return tag;
}
