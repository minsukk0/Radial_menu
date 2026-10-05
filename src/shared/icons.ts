/**
 * 휠클릭 래디얼 메뉴 및 다이얼로그용 24x24 SVG 선 아이콘 모음
 * 선 굵기 1.5, 모서리 및 끝 둥글게(round), 24x24 격자 기준
 */

export const ICONS = {
  apps: `<rect x="4" y="4" width="6.5" height="6.5" rx="1.5"/><rect x="13.5" y="4" width="6.5" height="6.5" rx="1.5"/><rect x="4" y="13.5" width="6.5" height="6.5" rx="1.5"/><rect x="13.5" y="13.5" width="6.5" height="6.5" rx="1.5"/>`,
  terminal: `<rect x="3" y="4.5" width="18" height="15" rx="2.5"/><path d="m7 9.5 3 2.5-3 2.5M12.5 15H17"/>`,
  folder: `<path d="M3.5 7.5a2 2 0 0 1 2-2h3.6l2 2.2h7.4a2 2 0 0 1 2 2v7.8a2 2 0 0 1-2 2h-13a2 2 0 0 1-2-2z"/>`,
  power: `<path d="M12 3v8.5"/><path d="M7.1 6.7a8 8 0 1 0 9.8 0"/>`,
  window: `<rect x="3" y="5" width="18" height="14" rx="2.5"/><path d="M3 9.5h18"/>`,
  play: `<path d="M8 5.5v13l10.5-6.5z"/>`,
  note: `<rect x="5" y="3.5" width="14" height="17" rx="2.5"/><path d="M8.5 8.5h7M8.5 12h7M8.5 15.5h4"/>`,
  paste: `<rect x="5" y="3.5" width="14" height="17" rx="2.5"/><path d="M8.5 8.5h7M8.5 12h7M8.5 15.5h4"/>`,
  globe: `<circle cx="12" cy="12" r="8.5"/><ellipse cx="12" cy="12" rx="3.6" ry="8.5"/><path d="M3.5 12h17"/>`,
  lock: `<rect x="5" y="10.5" width="14" height="10" rx="2.5"/><path d="M8 10.5V8a4 4 0 0 1 8 0v2.5"/>`,
  sleep: `<path d="M20 14.5A8.5 8.5 0 0 1 9.5 4 8.5 8.5 0 1 0 20 14.5z"/>`,
  settings: `<path d="M4 7h7M17 7h3M4 12h1.5M11 12h9M4 17h9M19 17h1"/><circle cx="14" cy="7" r="2.2"/><circle cx="8" cy="12" r="2.2"/><circle cx="16" cy="17" r="2.2"/>`,
  activity: `<path d="M3 12h4l2.5-6 4.5 12 2.5-6H21"/>`,
  timer: `<circle cx="12" cy="13.5" r="7"/><path d="M12 10v3.5l2.4 1.6M9.8 3.5h4.4"/>`,
  mute: `<path d="M4 9.5v5h3.5l4.5 4v-13l-4.5 4z"/><path d="m16 9.5 5 5M21 9.5l-5 5"/>`,
  sun: `<circle cx="12" cy="12" r="3.6"/><path d="M12 3.5v2M12 18.5v2M3.5 12h2M18.5 12h2M6 6l1.4 1.4M16.6 16.6 18 18M18 6l-1.4 1.4M7.4 16.6 6 18"/>`,
  screen: `<rect x="3" y="4" width="18" height="12.5" rx="2"/><path d="M8.5 20.5h7M12 16.5v4"/>`,
  trash: `<path d="M5 7h14M10 7V5h4v2M7 7l.8 12.5h8.4L17 7M10 10.5v5.5M14 10.5v5.5"/>`,
  plus: `<circle cx="12" cy="12" r="9.5" stroke-dasharray="2.8 3"/><path d="M12 8v8M8 12h8"/>`,
  close: `<path d="M7 7l10 10M17 7 7 17"/>`,
  check: `<path d="m5 13 4 4L19 7"/>`,
} as const;

export type IconName = keyof typeof ICONS;

/**
 * 특정 아이콘의 완전한 SVG 문자열을 생성한다.
 */
export function getIconSvg(
  name: string,
  size: number = 24,
  options: {
    strokeWidth?: number;
    className?: string;
    stroke?: string;
    ariaHidden?: boolean;
  } = {}
): string {
  const {
    strokeWidth = 1.5,
    className = '',
    stroke = 'currentColor',
    ariaHidden = true,
  } = options;

  const content = (ICONS as Record<string, string>)[name] || ICONS.window;
  const classAttr = className ? ` class="${className}"` : '';
  const ariaAttr = ariaHidden ? ' aria-hidden="true"' : '';

  return `<svg width="${size}" height="${size}" viewBox="0 0 24 24" fill="none" stroke="${stroke}" stroke-width="${strokeWidth}" stroke-linecap="round" stroke-linejoin="round"${classAttr}${ariaAttr}>${content}</svg>`;
}

/**
 * <symbol> 정의 모음을 생성하여 SVG <defs> 에 삽입할 수 있도록 한다.
 */
export function generateSvgSymbols(idPrefix: string = 'i-'): string {
  return Object.entries(ICONS)
    .map(([name, inner]) => `<symbol id="${idPrefix}${name}" viewBox="0 0 24 24">${inner}</symbol>`)
    .join('');
}

/**
 * 시스템 기능 키에 대응하는 기본 아이콘 이름 반환
 */
export function getSystemIcon(fn: string): IconName {
  switch (fn) {
    case 'lock': return 'lock';
    case 'sleep': return 'sleep';
    case 'settings': return 'settings';
    case 'taskmgr': return 'activity';
    case 'mute': return 'mute';
    case 'brightness': return 'sun';
    case 'screenoff': return 'screen';
    case 'emptybin': return 'trash';
    default: return 'power';
  }
}

/**
 * 분류 kind에 대응하는 기본 아이콘 이름 반환
 */
export function getCategoryDefaultIcon(kind: string): IconName {
  switch (kind) {
    case 'app': return 'apps';
    case 'terminal': return 'terminal';
    case 'folder': return 'folder';
    case 'system': return 'power';
    default: return 'apps';
  }
}
