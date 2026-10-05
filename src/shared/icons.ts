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
  gpt: `<path d="M11.22 19.38a3.5 3.5 0 0 0 6.78-1.22v-5.17l-6-3.35"/><path d="M5.21 15.01a3.5 3.5 0 0 0 4.45 5.27l4.34-2.53v-6.95"/><path d="M6 7.63a3.5 3.5 0 0 0-2.26 6.43l4.26 2.52 6-3.35"/><path d="M12.78 4.62a3.5 3.5 0 0 0-6.78 1.22v5.07l6 3.45"/><path d="M18.79 8.99a3.5 3.5 0 0 0-4.45-5.27l-4.34 2.53v6.95"/><path d="M18 16.3a3.5 3.5 0 0 0 2.26-6.43l-4.3-2.52-5.96 3.42"/>`,
  claude: `<g transform="translate(3, 3) scale(0.18)"><path d="m19.6 66.5 19.7-11 .3-1-.3-.5h-1l-3.3-.2-11.2-.3L14 53l-9.5-.5-2.4-.5L0 49l.2-1.5 2-1.3 2.9.2 6.3.5 9.5.6 6.9.4L38 49.1h1.6l.2-.7-.5-.4-.4-.4L29 41l-10.6-7-5.6-4.1-3-2-1.5-2-.6-4.2 2.7-3 3.7.3.9.2 3.7 2.9 8 6.1L37 36l1.5 1.2.6-.4.1-.3-.7-1.1L33 25l-6-10.4-2.7-4.3-.7-2.6c-.3-1-.4-2-.4-3l3-4.2L28 0l4.2.6L33.8 2l2.6 6 4.1 9.3L47 29.9l2 3.8 1 3.4.3 1h.7v-.5l.5-7.2 1-8.7 1-11.2.3-3.2 1.6-3.8 3-2L61 2.6l2 2.9-.3 1.8-1.1 7.7L59 27.1l-1.5 8.2h.9l1-1.1 4.1-5.4 6.9-8.6 3-3.5L77 13l2.3-1.8h4.3l3.1 4.7-1.4 4.9-4.4 5.6-3.7 4.7-5.3 7.1-3.2 5.7.3.4h.7l12-2.6 6.4-1.1 7.6-1.3 3.5 1.6.4 1.6-1.4 3.4-8.2 2-9.6 2-14.3 3.3-.2.1.2.3 6.4.6 2.8.2h6.8l12.6 1 3.3 2 1.9 2.7-.3 2-5.1 2.6-6.8-1.6-16-3.8-5.4-1.3h-.8v.4l4.6 4.5 8.3 7.5L89 80.1l.5 2.4-1.3 2-1.4-.2-9.2-7-3.6-3-8-6.8h-.5v.7l1.8 2.7 9.8 14.7.5 4.5-.7 1.4-2.6 1-2.7-.6-5.8-8-6-9-4.7-8.2-.5.4-2.9 30.2-1.3 1.5-3 1.2-2.5-2-1.4-3 1.4-6.2 1.6-8 1.3-6.4 1.2-7.9.7-2.6v-.2H49L43 72l-9 12.3-7.2 7.6-1.7.7-3-1.5.3-2.8L24 86l10-12.8 6-7.9 4-4.6-.1-.5h-.3L17.2 77.4l-4.7.6-2-2 .2-3 1-1 8-5.5Z" fill="currentColor" stroke="none"/></g>`,
  chatgpt: `<path d="M11.22 19.38a3.5 3.5 0 0 0 6.78-1.22v-5.17l-6-3.35"/><path d="M5.21 15.01a3.5 3.5 0 0 0 4.45 5.27l4.34-2.53v-6.95"/><path d="M6 7.63a3.5 3.5 0 0 0-2.26 6.43l4.26 2.52 6-3.35"/><path d="M12.78 4.62a3.5 3.5 0 0 0-6.78 1.22v5.07l6 3.45"/><path d="M18.79 8.99a3.5 3.5 0 0 0-4.45-5.27l-4.34 2.53v6.95"/><path d="M18 16.3a3.5 3.5 0 0 0 2.26-6.43l-4.3-2.52-5.96 3.42"/>`,
  openai: `<path d="M11.22 19.38a3.5 3.5 0 0 0 6.78-1.22v-5.17l-6-3.35"/><path d="M5.21 15.01a3.5 3.5 0 0 0 4.45 5.27l4.34-2.53v-6.95"/><path d="M6 7.63a3.5 3.5 0 0 0-2.26 6.43l4.26 2.52 6-3.35"/><path d="M12.78 4.62a3.5 3.5 0 0 0-6.78 1.22v5.07l6 3.45"/><path d="M18.79 8.99a3.5 3.5 0 0 0-4.45-5.27l-4.34 2.53v6.95"/><path d="M18 16.3a3.5 3.5 0 0 0 2.26-6.43l-4.3-2.52-5.96 3.42"/>`,
  anthropic: `<g transform="translate(3, 3) scale(0.18)"><path d="m19.6 66.5 19.7-11 .3-1-.3-.5h-1l-3.3-.2-11.2-.3L14 53l-9.5-.5-2.4-.5L0 49l.2-1.5 2-1.3 2.9.2 6.3.5 9.5.6 6.9.4L38 49.1h1.6l.2-.7-.5-.4-.4-.4L29 41l-10.6-7-5.6-4.1-3-2-1.5-2-.6-4.2 2.7-3 3.7.3.9.2 3.7 2.9 8 6.1L37 36l1.5 1.2.6-.4.1-.3-.7-1.1L33 25l-6-10.4-2.7-4.3-.7-2.6c-.3-1-.4-2-.4-3l3-4.2L28 0l4.2.6L33.8 2l2.6 6 4.1 9.3L47 29.9l2 3.8 1 3.4.3 1h.7v-.5l.5-7.2 1-8.7 1-11.2.3-3.2 1.6-3.8 3-2L61 2.6l2 2.9-.3 1.8-1.1 7.7L59 27.1l-1.5 8.2h.9l1-1.1 4.1-5.4 6.9-8.6 3-3.5L77 13l2.3-1.8h4.3l3.1 4.7-1.4 4.9-4.4 5.6-3.7 4.7-5.3 7.1-3.2 5.7.3.4h.7l12-2.6 6.4-1.1 7.6-1.3 3.5 1.6.4 1.6-1.4 3.4-8.2 2-9.6 2-14.3 3.3-.2.1.2.3 6.4.6 2.8.2h6.8l12.6 1 3.3 2 1.9 2.7-.3 2-5.1 2.6-6.8-1.6-16-3.8-5.4-1.3h-.8v.4l4.6 4.5 8.3 7.5L89 80.1l.5 2.4-1.3 2-1.4-.2-9.2-7-3.6-3-8-6.8h-.5v.7l1.8 2.7 9.8 14.7.5 4.5-.7 1.4-2.6 1-2.7-.6-5.8-8-6-9-4.7-8.2-.5.4-2.9 30.2-1.3 1.5-3 1.2-2.5-2-1.4-3 1.4-6.2 1.6-8 1.3-6.4 1.2-7.9.7-2.6v-.2H49L43 72l-9 12.3-7.2 7.6-1.7.7-3-1.5.3-2.8L24 86l10-12.8 6-7.9 4-4.6-.1-.5h-.3L17.2 77.4l-4.7.6-2-2 .2-3 1-1 8-5.5Z" fill="currentColor" stroke="none"/></g>`,
  antigravity: `<path d="M12.00 4.50 C13.37 4.50, 14.84 5.79, 15.79 8.68 C16.63 11.36, 17.89 15.64, 19.89 18.86 C20.21 19.39, 19.47 19.61, 18.63 19.07 C17.58 18.00, 15.89 15.43, 14.11 13.61 C13.05 12.54, 12.42 12.54, 12.00 12.54 C11.58 12.54, 10.95 12.54, 9.89 13.61 C8.11 15.43, 6.42 18.00, 5.37 19.07 C4.53 19.61, 3.79 19.39, 4.11 18.86 C6.11 15.64, 7.37 11.36, 8.21 8.68 C9.16 5.79, 10.63 4.50, 12.00 4.50 Z" fill="currentColor" stroke="none"/>`,
  gemini: `<path d="M12.00 4.50 C13.37 4.50, 14.84 5.79, 15.79 8.68 C16.63 11.36, 17.89 15.64, 19.89 18.86 C20.21 19.39, 19.47 19.61, 18.63 19.07 C17.58 18.00, 15.89 15.43, 14.11 13.61 C13.05 12.54, 12.42 12.54, 12.00 12.54 C11.58 12.54, 10.95 12.54, 9.89 13.61 C8.11 15.43, 6.42 18.00, 5.37 19.07 C4.53 19.61, 3.79 19.39, 4.11 18.86 C6.11 15.64, 7.37 11.36, 8.21 8.68 C9.16 5.79, 10.63 4.50, 12.00 4.50 Z" fill="currentColor" stroke="none"/>`,
  agy: `<path d="M12.00 4.50 C13.37 4.50, 14.84 5.79, 15.79 8.68 C16.63 11.36, 17.89 15.64, 19.89 18.86 C20.21 19.39, 19.47 19.61, 18.63 19.07 C17.58 18.00, 15.89 15.43, 14.11 13.61 C13.05 12.54, 12.42 12.54, 12.00 12.54 C11.58 12.54, 10.95 12.54, 9.89 13.61 C8.11 15.43, 6.42 18.00, 5.37 19.07 C4.53 19.61, 3.79 19.39, 4.11 18.86 C6.11 15.64, 7.37 11.36, 8.21 8.68 C9.16 5.79, 10.63 4.50, 12.00 4.50 Z" fill="currentColor" stroke="none"/>`,
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

  return `<svg width="${size}" height="${size}" viewBox="0 0 24 24" fill="none" stroke="${stroke}" color="${stroke}" stroke-width="${strokeWidth}" stroke-linecap="round" stroke-linejoin="round"${classAttr}${ariaAttr}>${content}</svg>`;
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
    case 'empty': return 'plus';
    default: return 'apps';
  }
}
