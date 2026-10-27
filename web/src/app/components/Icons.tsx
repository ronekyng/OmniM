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
