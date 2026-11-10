import { isDigit } from '@monochromatic-dev/agent-harness-shared-text-scan/ts';

/**
 Check that a text is a plain decimal number.

 @param part - one part of an image user

 @returns true for one or more digits and nothing else

 @example
 ```ts
 isDecimal('1000');
 ```
 */
function isDecimal(part: string): boolean {
  return (part !== '')
    && Array.from(part)
      .every(function checkDigit(character) {
        return isDigit(character);
      });
}

/**
 Podman user-namespace argument that makes the container's own user the host user, so the files it writes into a
 bind-mounted directory belong to the host user and can be read, replaced and deleted without `podman unshare`.

 The container still runs as the image's user: `keep-id` is given that user's numbers, so an image without a `USER`
 line keeps running as root, which the host user then stands for, and an image that runs as 1000 keeps running as 1000.

 @param imageUser - the image's configured user as `podman image inspect` reports it: empty for root, `uid`, or `uid:gid`

 @returns the argument, or none when the user is a name that cannot be mapped without looking inside the image

 @example
 ```ts
 keepIdArguments('1000:1000');
 ```
 */
export function keepIdArguments(imageUser: string): readonly string[] {
  /**
   User and group parts of the text.
   */
  const [user = '', group = '0', ...extra] = imageUser.trim()
    .split(':');
  /**
   Whether the whole text is a numeric user and, when present, a numeric group.
   */
  const numeric = ((user === '') || isDecimal(user)) && isDecimal(group)
    && (extra.length === 0);
  return numeric
    ? [`--userns=keep-id:uid=${user === '' ? '0' : user},gid=${group}`]
    : [];
}
