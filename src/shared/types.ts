/**
 * 휠클릭 래디얼 메뉴의 공용 타입 정의
 * docs/configuration.md, docs/interaction.md, docs/architecture.md 기반
 */

// ==========================================
// 1. 설정 모델 (menu.json)
// ==========================================

export type CategoryKind = 'app' | 'terminal' | 'folder' | 'system';

export type SystemFn =
  | 'lock'
  | 'sleep'
  | 'settings'
  | 'taskmgr'
  | 'mute'
  | 'brightness'
  | 'screenoff'
  | 'emptybin';

export type TerminalShell = 'powershell' | 'cmd' | 'wsl' | 'gitbash';
export type TerminalWindowMode = 'hidden' | 'visible' | 'keep';
export type AppWhenRunning = 'focus' | 'new';
export type FolderOpenWith = 'default' | 'explorer' | 'browser';

export interface BaseItemConfig {
  id?: string;
  label: string;
}

export interface AppItemConfig extends BaseItemConfig {
  path: string;
  args?: string;
  whenRunning?: AppWhenRunning;
  runAsAdmin?: boolean;
  icon?: string;
  mono?: string; // 앱 아이콘 없을 때 2글자 모노그램
}

export interface TerminalItemConfig extends BaseItemConfig {
  icon?: string;
  action: 'run' | 'open';
  mode?: 'command' | 'file';
  command?: string;
  file?: string;
  args?: string;
  shell?: TerminalShell;
  workDir?: string;
  startDir?: string;
  window?: TerminalWindowMode;
  runAsAdmin?: boolean;
  notifyOnDone?: boolean;
  copyOutput?: boolean;
  confirmFirst?: boolean;
  reuseTab?: boolean;
}

export interface FolderItemConfig extends BaseItemConfig {
  target: string;
  icon?: string;
  openWith?: FolderOpenWith;
}

export interface SystemItemConfig extends BaseItemConfig {
  fn: SystemFn;
  icon?: string;
}

export type MenuItemConfig =
  | AppItemConfig
  | TerminalItemConfig
  | FolderItemConfig
  | SystemItemConfig;

export interface CategoryConfig {
  id?: string;
  kind: CategoryKind;
  label: string;
  icon?: string;
  items: MenuItemConfig[];
}

export interface MenuConfig {
  $schema?: string;
  version: number;
  accent: string;
  categories: CategoryConfig[];
}

// ==========================================
// 2. 래디얼 레이아웃 기하학 모델
// ==========================================

export interface GeometryConfig {
  hubRadius: number;
  arcRadius: number;
  ringInner: number;
  ringOuter: number;
  ringLabelRadius: number;
  ringMinSectors: number;
  selectionSpill: number;
  bandInner: number;
  bandOuter: number;
  bandCorner: number;
  bandEndPadding: number;
  slotInner: number;
  slotOuter: number;
  slotSideInset: number;
  slotCorner: number;
  slotAngle: number;
  slotMaxSpan: number;
  slotMinCount: number;
  slotLabelRadius: number;
  slotDividerInner: number;
  slotDividerOuter: number;
  slotFirstDelay: number;
  slotStagger: number;
  slotStaggerBudget: number;
}

export interface LayoutOptions {
  cx?: number;
  cy?: number;
  geometry?: Partial<GeometryConfig>;
}

export interface LayoutSector {
  index: number;
  axis: number;
  path: string;
  labelX: number;
  labelY: number;
  selected: boolean;
  category: CategoryConfig;
}

export interface LayoutAddSector {
  axis: number;
  path: string;
  labelX: number;
  labelY: number;
}

export interface LayoutSelection {
  visible: boolean;
  axis: number | null;
  wedge: string;
  edge: string;
  arc: string;
}

export interface LayoutSlot {
  kind: 'item' | 'add';
  index: number;
  angle: number;
  path: string;
  labelX: number;
  labelY: number;
  delayMs: number;
  item: MenuItemConfig | null;
}

export interface LayoutSub {
  axis: number;
  slotCount: number;
  slotAngle: number;
  span: number;
  band: string;
  dividers: string;
  slots: LayoutSlot[];
}

export interface LayoutResult {
  sectorCount: number;
  step: number;
  sectors: LayoutSector[];
  addSector: LayoutAddSector | null;
  dividers: string;
  selection: LayoutSelection;
  sub: LayoutSub | null;
}

export type HitTestZone = 'hub' | 'ring' | 'band' | 'outside';

export interface HitTestResult {
  zone: HitTestZone;
  sector: number;
  angle: number;
  radius: number;
}

// ==========================================
// 3. 실행 대상 및 제스처 / IPC 모델
// ==========================================

export type CommitTarget =
  | {
      type: 'item';
      categoryIndex: number;
      itemIndex: number;
      category: CategoryConfig;
      item: MenuItemConfig;
    }
  | {
      type: 'add_item';
      categoryIndex: number;
      category: CategoryConfig;
    }
  | {
      type: 'add_category';
    }
  | {
      type: 'close';
    }
  | {
      type: 'none';
    };

export interface MenuOpenPayload {
  x: number;
  y: number;
  config?: MenuConfig;
}

export interface MenuCursorPayload {
  x: number;
  y: number;
}

export interface MenuCommitPayload {
  target: CommitTarget;
}

export interface MenuState {
  hotCategory: number;
  hotSlot: number;
  rotation: number | undefined;
  isOpen: boolean;
  config: MenuConfig;
}
