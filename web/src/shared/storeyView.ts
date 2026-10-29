import type { MeshObject } from './types';

/** Storey filter for the viewport (P8-S4).
 *
 *  Objects explicitly associated with a storey (`level_id`, then
 *  `bottom_level_id`) are shown only on that storey; unassigned objects stay
 *  visible everywhere so loose geometry never disappears silently. */
export function meshVisibleOnStorey(
  object: Pick<MeshObject, 'level_id' | 'bottom_level_id'>,
  storeyId: string | null,
): boolean {
  if (!storeyId) return true;
  if (object.level_id) return object.level_id === storeyId;
  if (object.bottom_level_id) return object.bottom_level_id === storeyId;
  return true;
}
