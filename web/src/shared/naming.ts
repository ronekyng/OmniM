/** Stable ASCII auto-naming (UI-INT-09).
 *
 *  Pattern: `<prefix>-<nnn>` (three-digit minimum, per prefix, per project).
 *  The lowest free number is reused only after the previous holder was
 *  deleted. Prefixes are never localized: they are exchange identifiers.
 */

const escapeRegex = (value: string) => value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');

export function nextAutoName(prefix: string, existingNames: Iterable<string>): string {
  const clean = (prefix || 'Object').trim().replace(/\s+/g, '-');
  const used = new Set<number>();
  const pattern = new RegExp(`^${escapeRegex(clean)}-(\\d+)$`);
  for (const name of existingNames) {
    const match = pattern.exec(name);
    if (match) used.add(Number(match[1]));
  }
  let n = 1;
  while (used.has(n)) n += 1;
  return `${clean}-${String(n).padStart(3, '0')}`;
}

/** Ensure a user-visible name is unique by appending `-2`, `-3`… */
export function uniqueName(candidate: string, existingNames: Iterable<string>): string {
  const taken = new Set(existingNames);
  if (!taken.has(candidate)) return candidate;
  let n = 2;
  while (taken.has(`${candidate}-${n}`)) n += 1;
  return `${candidate}-${n}`;
}
