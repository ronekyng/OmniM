/** Minimal SVG icon set — each icon is a 16×16 SVG inline component */

type IconProps = { size?: number; className?: string };

function icon(size: number | undefined, children: React.ReactNode) {
  const s = size ?? 16;
  return (
    <svg width={s} height={s} viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
      {children}
    </svg>
  );
}

export const IcoRefresh = ({ size }: IconProps) => icon(size, <><path d="M2 8a6 6 0 0 1 10.5-3.9" /><path d="M14 4v4h-4" /><path d="M14 8a6 6 0 0 1-10.5 3.9" /><path d="M2 12V8h4" /></>);

export const IcoAdd = ({ size }: IconProps) => icon(size, <><path d="M8 3v10M3 8h10" /></>);

export const IcoSmart = ({ size }: IconProps) => icon(size, <><circle cx="8" cy="8" r="2.5" /><path d="M8 1v2M8 13v2M1 8h2M13 8h2M3.1 3.1l1.4 1.4M11.5 11.5l1.4 1.4M3.1 12.9l1.4-1.4M11.5 4.5l1.4-1.4" /></>);

export const IcoAssy = ({ size }: IconProps) => icon(size, <><rect x="2" y="2" width="5" height="5" rx="1" /><rect x="9" y="2" width="5" height="5" rx="1" /><rect x="5.5" y="9" width="5" height="5" rx="1" /></>);

export const IcoDelete = ({ size }: IconProps) => icon(size, <><path d="M3 4h10M5 4V3a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v1M6 6v5M10 6v5M12 4v9a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V4" /></>);

export const IcoUndo = ({ size }: IconProps) => icon(size, <><path d="M5 3 2 6l3 3" /><path d="M2 6h8a3 3 0 0 1 3 3v1" /></>);

export const IcoRedo = ({ size }: IconProps) => icon(size, <><path d="M11 3l3 3-3 3" /><path d="M14 6H6a3 3 0 0 0-3 3v1" /></>);

export const IcoImport = ({ size }: IconProps) => icon(size, <><path d="M10 11 8 9l-2 2" /><path d="M8 2v7" /><path d="M3 10v2a1 1 0 0 0 1 1h8a1 1 0 0 0 1-1v-2" /></>);

export const IcoExport = ({ size }: IconProps) => icon(size, <><path d="M10 5 8 7 6 5" /><path d="M8 2v7" /><path d="M3 10v2a1 1 0 0 0 1 1h8a1 1 0 0 0 1-1v-2" /></>);

export const IcoMove = ({ size }: IconProps) => icon(size, <><path d="M8 1v6M8 9v6M1 8h6M9 8h6M4 4l4-3 4 3M4 12l4 3 4-3M12 4l3 4-3 4M4 4 1 8l3 4" /></>);

export const IcoRotate = ({ size }: IconProps) => icon(size, <><path d="M11 5a4 4 0 1 0 1 5" /><path d="M14 4v4h-4" /></>);

export const IcoScale = ({ size }: IconProps) => icon(size, <><path d="M3 3h10v10H3z" /><path d="M8 3v10M3 8h10" /></>);

export const IcoWireframe = ({ size }: IconProps) => icon(size, <><rect x="3" y="3" width="10" height="10" rx="1" /><path d="M3 6h10M6 3v10" /></>);

export const IcoShaded = ({ size }: IconProps) => icon(size, <><rect x="3" y="3" width="10" height="10" rx="1" fill="currentColor" fillOpacity="0.25" /><path d="M3 3h10v10H3z" /></>);

export const IcoHiddenLine = ({ size }: IconProps) => icon(size, <><rect x="3" y="3" width="10" height="10" rx="1" fill="currentColor" fillOpacity="0.15" /><path d="M3 3h10v10H3zM3 6.5h10M6.5 3v10" /></>);

export const IcoRealistic = ({ size }: IconProps) => icon(size, <><circle cx="8" cy="8" r="5" fill="currentColor" fillOpacity="0.2" /><circle cx="8" cy="8" r="5" /><path d="M5 5l6 6" strokeOpacity="0.4" /></>);

export const IcoSection = ({ size }: IconProps) => icon(size, <><path d="M2 14 14 2M3 3h10v10H3z" /></>);

export const IcoFit = ({ size }: IconProps) => icon(size, <><path d="M7 3H4a1 1 0 0 0-1 1v3M9 3h3a1 1 0 0 1 1 1v3M7 13H4a1 1 0 0 1-1-1V9M9 13h3a1 1 0 0 0 1-1V9" /></>);

export const IcoClose = ({ size }: IconProps) => icon(size, <><path d="M4 4l8 8M12 4l-8 8" /></>);

export const IcoSearch = ({ size }: IconProps) => icon(size, <><circle cx="6.5" cy="6.5" r="3.5" /><path d="M9 9l4 4" /></>);

export const IcoCopy = ({ size }: IconProps) => icon(size, <><rect x="5" y="5" width="7" height="7" rx="1" /><path d="M2 9V3a1 1 0 0 1 1-1h6" /></>);

export const IcoFocus = ({ size }: IconProps) => icon(size, <><circle cx="8" cy="8" r="2" /><path d="M8 1v2M8 13v2M1 8h2M13 8h2" /></>);

export const IcoFolderOpen = ({ size }: IconProps) => icon(size, <><path d="M2 4v8a1 1 0 0 0 1 1h10a1 1 0 0 0 1-1V7a1 1 0 0 0-1-1H8L6 4H3a1 1 0 0 0-1 1z" /></>);

export const IcoInfo = ({ size }: IconProps) => icon(size, <><circle cx="8" cy="8" r="6" /><path d="M8 7v4M8 5v.5" /></>);

export const IcoHome = ({ size }: IconProps) => icon(size, <><path d="M8 1 1 8h2v6h4v-4h2v4h4V8h2L8 1z" /></>);

export const IcoGrid = ({ size }: IconProps) => icon(size, <><path d="M3 3h10v10H3zM3 8h10M8 3v10" /></>);

export const IcoLocal = ({ size }: IconProps) => icon(size, <><circle cx="8" cy="8" r="3" /><path d="M8 2v2M8 12v2M2 8h2M12 8h2" /></>);

export const IcoOrtho = ({ size }: IconProps) => icon(size, <><rect x="3" y="3" width="10" height="10" /><path d="M3 8h10M8 3v10" /></>);

export const IcoSketch = ({ size }: IconProps) => icon(size, <><path d="M3 3l4 10 6-8 2 6" /><circle cx="3" cy="3" r="1" /><circle cx="7" cy="13" r="1" /><circle cx="13" cy="5" r="1" /><circle cx="15" cy="11" r="1" /></>);

export const IcoCatalog = ({ size }: IconProps) => icon(size, <><rect x="2" y="2" width="12" height="3" rx="1" /><rect x="2" y="6.5" width="12" height="3" rx="1" /><rect x="2" y="11" width="12" height="3" rx="1" /></>);

export const IcoSeries = ({ size }: IconProps) => icon(size, <><circle cx="4" cy="4" r="2" /><circle cx="12" cy="4" r="2" /><circle cx="4" cy="12" r="2" /><circle cx="12" cy="12" r="2" /><path d="M6 4h6M4 6v6M12 6v6M6 12h6" /></>);

export const IcoMeasureDist = ({ size }: IconProps) => icon(size, <><path d="M3 12 13 4" /><circle cx="3" cy="12" r="2" /><circle cx="13" cy="4" r="2" /><text x="8" y="10" fontSize="3" fill="currentColor" stroke="none" textAnchor="middle">d</text></>);

export const IcoMeasureAngle = ({ size }: IconProps) => icon(size, <><path d="M3 13 8 3 13 13" /><circle cx="8" cy="3" r="2" /><path d="M8 3A5 5 0 0 1 12 9" fill="none" /><text x="8" y="11" fontSize="3" fill="currentColor" stroke="none" textAnchor="middle">∠</text></>);

export const IcoMeasureArea = ({ size }: IconProps) => icon(size, <><path d="M3 12 6 3 13 5 10 14Z" fill="currentColor" fillOpacity="0.15" /><text x="8" y="10" fontSize="3" fill="currentColor" stroke="none" textAnchor="middle">A</text></>);

export const IcoXray = ({ size }: IconProps) => icon(size, <><circle cx="8" cy="8" r="5" fill="currentColor" fillOpacity="0.15" /><circle cx="8" cy="8" r="5" /><path d="M5 8h6M8 5v6" strokeOpacity="0.5" /></>);

export const IcoSnap = ({ size }: IconProps) => icon(size, <><circle cx="8" cy="8" r="5" fill="currentColor" fillOpacity="0.15" /><circle cx="8" cy="8" r="5" stroke="currentColor" fill="none" /><path d="M8 5v6M5 8h6" /></>);

/* ── UI-DS-09: glyph replacements for emoji / pictographic characters ── */

export const IcoChevronUp = ({ size }: IconProps) => icon(size, <path d="M4 10l4-4 4 4" />);
export const IcoChevronDown = ({ size }: IconProps) => icon(size, <path d="M4 6l4 4 4-4" />);
export const IcoChevronLeft = ({ size }: IconProps) => icon(size, <path d="M10 4 6 8l4 4" />);
export const IcoChevronRight = ({ size }: IconProps) => icon(size, <path d="M6 4l4 4-4 4" />);
export const IcoCheck = ({ size }: IconProps) => icon(size, <path d="M3 8.5 6.5 12 13 4" />);
export const IcoWarning = ({ size }: IconProps) => icon(size, <><path d="M8 2 1.5 13.5h13L8 2z" /><path d="M8 6v4M8 12v.5" /></>);
export const IcoEdit = ({ size }: IconProps) => icon(size, <><path d="M10.5 2.5 13.5 5.5 6 13H3v-3l7.5-7.5z" /><path d="M9 4l3 3" /></>);
export const IcoHelp = ({ size }: IconProps) => icon(size, <><circle cx="8" cy="8" r="6" /><path d="M6 6a2 2 0 1 1 3 1.7c-.6.4-1 .9-1 1.8M8 12v.5" /></>);
export const IcoBox = ({ size }: IconProps) => icon(size, <><path d="M2 5l6-3 6 3v6l-6 3-6-3V5z" /><path d="M2 5l6 3 6-3M8 8v6" /></>);
export const IcoVolumeOn = ({ size }: IconProps) => icon(size, <><path d="M3 6h3l3-3v10l-3-3H3V6z" /><path d="M11 6a3 3 0 0 1 0 4M12.5 4.5a5 5 0 0 1 0 7" /></>);
export const IcoVolumeOff = ({ size }: IconProps) => icon(size, <><path d="M3 6h3l3-3v10l-3-3H3V6z" /><path d="M11 6l4 4M15 6l-4 4" /></>);
export const IcoMagnet = ({ size }: IconProps) => icon(size, <><path d="M4 3v5a4 4 0 0 0 8 0V3" /><path d="M4 3h3v5a1 1 0 0 0 2 0V3h3" /><path d="M4 6h3M9 6h3" /></>);
export const IcoAnchor = ({ size }: IconProps) => icon(size, <><circle cx="8" cy="3.5" r="1.5" /><path d="M8 5v8M4 8H2a6 6 0 0 0 12 0h-2M5 10.5c1 1.5 2 2 3 2s2-.5 3-2" /></>);
export const IcoSun = ({ size }: IconProps) => icon(size, <><circle cx="8" cy="8" r="3" /><path d="M8 1v2M8 13v2M1 8h2M13 8h2M3.1 3.1l1.4 1.4M11.5 11.5l1.4 1.4M3.1 12.9l1.4-1.4M11.5 4.5l1.4-1.4" /></>);
export const IcoMoon = ({ size }: IconProps) => icon(size, <path d="M13 9.5A5.5 5.5 0 0 1 6.5 3a5.5 5.5 0 1 0 6.5 6.5z" />);
export const IcoTarget = ({ size }: IconProps) => icon(size, <><path d="M8 14s5-4.5 5-8A5 5 0 0 0 3 6c0 3.5 5 8 5 8z" /><circle cx="8" cy="6" r="1.5" /></>);
export const IcoDrag = ({ size }: IconProps) => icon(size, <><path d="M8 1v14M5 4l3-3 3 3M5 12l3 3 3-3" /></>);
export const IcoStar = ({ size, filled }: IconProps & { filled?: boolean }) =>
  icon(size, <path d="M8 1.5l1.9 4 4.4.6-3.2 3.1.8 4.3L8 11.5l-3.9 2 .8-4.3L1.7 6l4.4-.6L8 1.5z" fill={filled ? 'currentColor' : 'none'} />);

export type GlyphName =
  | 'wall' | 'slab' | 'column' | 'beam' | 'door' | 'window' | 'rebar'
  | 'duct' | 'tray' | 'road' | 'pipe' | 'manhole' | 'tank' | 'pump'
  | 'valve' | 'hx' | 'compressor' | 'angle' | 'gear'
  | 'extrude' | 'revolve' | 'sweep' | 'loft' | 'fusion' | 'lofted_fusion'
  | 'boolean' | 'chamfer' | 'fillet' | 'pattern' | 'shell';

/** Single glyph index for catalog quick-actions and feature types. */
export function Glyph({ name, size }: { name: GlyphName; size?: number }) {
  const s = size ?? 16;
  const shapes: Record<GlyphName, React.ReactNode> = {
    wall: <><rect x="2" y="4" width="12" height="8" /><path d="M5 4v8M8 4v8M11 4v8" /></>,
    slab: <rect x="1" y="5" width="14" height="6" rx="1" />,
    column: <rect x="5" y="2" width="6" height="12" rx="1" />,
    beam: <rect x="1" y="6" width="14" height="4" rx="1" />,
    door: <><rect x="4" y="2" width="7" height="12" /><path d="M11 4a4 4 0 0 1 0 8" /></>,
    window: <><rect x="2" y="4" width="12" height="8" /><path d="M8 4v8M2 8h12" /></>,
    rebar: <><circle cx="8" cy="8" r="5" /><circle cx="8" cy="8" r="1" /><path d="M8 1v3M8 12v3M1 8h3M12 8h3" /></>,
    duct: <><rect x="2" y="4" width="12" height="8" /><path d="M5 4v8" /></>,
    tray: <path d="M2 5v6h12V5M2 5h12M5 8h6" />,
    road: <><path d="M3 13 5 3M11 13 13 3" /><path d="M8 4v2M8 8v2M8 12v1" /></>,
    pipe: <><circle cx="8" cy="8" r="5" /><circle cx="8" cy="8" r="1.5" /></>,
    manhole: <><circle cx="8" cy="8" r="6" /><circle cx="8" cy="8" r="3" /></>,
    tank: <><rect x="4" y="2" width="8" height="12" rx="4" /><path d="M4 6h8M4 10h8" /></>,
    pump: <><circle cx="7" cy="8" r="4" /><path d="M11 8h3M3 8H1" /></>,
    valve: <><path d="M2 4l6 4-6 4V4zM14 4l-6 4 6 4V4z" /><path d="M8 8v4" /></>,
    hx: <><rect x="2" y="4" width="12" height="8" rx="2" /><path d="M4 8c1-1 2-1 3 0s2 1 3 0 2-1 3 0" /></>,
    compressor: <><rect x="2" y="5" width="9" height="7" rx="1" /><circle cx="12" cy="8.5" r="2" /></>,
    angle: <path d="M4 2v11h10" />,
    gear: <><circle cx="8" cy="8" r="3" /><path d="M8 1v3M8 12v3M1 8h3M12 8h3M3 3l2 2M11 11l2 2M3 13l2-2M11 5l2-2" /></>,
    extrude: <><rect x="3" y="8" width="8" height="5" /><path d="M7 2v6M4.5 5 7 2l2.5 3" /></>,
    revolve: <><path d="M12 5a4 4 0 1 0 1 5" /><path d="M15 4v4h-4" /></>,
    sweep: <><path d="M2 12c4-8 8 4 12-4" /><path d="M11 5h3v3" /></>,
    loft: <><ellipse cx="8" cy="4" rx="4" ry="1.5" /><ellipse cx="8" cy="12" rx="6" ry="1.5" /><path d="M4 4v8M12 4v8" /></>,
    fusion: <><circle cx="6" cy="8" r="3.5" /><circle cx="10" cy="8" r="3.5" /></>,
    lofted_fusion: <><ellipse cx="8" cy="3" rx="3" ry="1.2" /><ellipse cx="8" cy="13" rx="3" ry="1.2" /><path d="M5 4c0 4 6 4 6 8M11 4c0 4-6 4-6 8" /></>,
    boolean: <><circle cx="6" cy="8" r="3.5" /><circle cx="10" cy="8" r="3.5" /><path d="M8 5v6" /></>,
    chamfer: <path d="M3 3h7l3 3v7H3V3z" />,
    fillet: <path d="M3 3h6a4 4 0 0 1 4 4v6" />,
    pattern: <><rect x="2" y="2" width="5" height="5" /><rect x="9" y="2" width="5" height="5" /><rect x="2" y="9" width="5" height="5" /><rect x="9" y="9" width="5" height="5" /></>,
    shell: <><rect x="2" y="2" width="12" height="12" /><rect x="5" y="5" width="6" height="6" /></>,
  };
  return icon(s, shapes[name] ?? shapes.extrude);
}
