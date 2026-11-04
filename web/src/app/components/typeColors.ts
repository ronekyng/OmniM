/** Type-colour palette, split out of ThreeViewer so the tree (and anything
 * else light) can colour by IFC type without pulling the three.js graph into
 * the initial chunk (PRF-01/06). ThreeViewer re-imports from here. */
export function colorForType(type: string) {
  if (type.includes('Site')) return 0x4caf50;
  if (type.includes('Building')) return 0x2196f3;
  if (type.includes('Wall')) return 0xff9800;
  if (type.includes('Beam')) return 0xff5722;
  if (type.includes('Column')) return 0xf44336;
  if (type.includes('Slab')) return 0x9c27b0;
  if (type.includes('Door')) return 0x795548;
  if (type.includes('Window')) return 0x607d8b;
  if (type.includes('Roof')) return 0x8bc34a;
  if (type.includes('Tank')) return 0x00bcd4;
  if (type.includes('Pipe')) return 0x673ab7;
  if (type.includes('Part')) return 0xffeb3b;
  return 0x1565c0;
}
