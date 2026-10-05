/**
 * main.ts — 래디얼 메뉴 웹뷰 진입점 및 제어 로직
 * 상태 관리, hitTest 기반 커서 추적, 키/마우스 이벤트, Tauri IPC 통신
 */

import './menu.css';
import { MenuRenderer } from './render';
import { hitTest } from './radial-layout';
import { listenEvent, emitEvent, invokeCommand } from '../shared/ipc';
import type {
  MenuConfig,
  MenuState,
  CommitTarget,
  LayoutSlot,
  LayoutResult,
  MenuOpenPayload,
  MenuCursorPayload,
} from '../shared/types';

// 기본 예시 설정 (초기화 및 개발 테스트용)
export const DEFAULT_CONFIG: MenuConfig = {
  version: 1,
  accent: '#1E9BFF',
  categories: [
    {
      id: 'apps',
      kind: 'app',
      label: '앱',
      icon: 'apps',
      items: [
        { label: 'ChatGPT', path: 'https://chatgpt.com', icon: 'gpt' },
        { label: 'Claude', path: 'https://claude.ai', icon: 'claude' },
        { label: 'Antigravity', path: 'antigravity.exe', icon: 'antigravity' },
        { label: '메모장', path: 'notepad.exe', mono: '메' },
      ],
    },
    {
      id: 'terminal',
      kind: 'terminal',
      label: '터미널',
      icon: 'terminal',
      items: [
        { label: '새 창', icon: 'window', action: 'open' },
        { label: '여기서 열기', icon: 'folder', action: 'open' },
        { label: '빌드 스크립트', icon: 'play', action: 'run' },
      ],
    },
    {
      id: 'folder',
      kind: 'folder',
      label: '폴더·주소',
      icon: 'folder',
      items: [
        { label: '다운로드', icon: 'folder', target: '%USERPROFILE%\\Downloads' },
        { label: '문서', icon: 'note', target: '%USERPROFILE%\\Documents' },
        { label: '프로젝트', icon: 'folder', target: 'D:\\Projects' },
      ],
    },
    {
      id: 'system',
      kind: 'system',
      label: '시스템',
      icon: 'power',
      items: [
        { label: '잠금', fn: 'lock' },
        { label: '절전', fn: 'sleep' },
        { label: '설정', fn: 'settings' },
        { label: '작업 관리자', fn: 'taskmgr' },
      ],
    },
  ],
};

export class MenuController {
  public state: MenuState;
  private renderer: MenuRenderer;
  private currentLayout: LayoutResult | null = null;
  private cx: number = 340;
  private cy: number = 340;

  constructor(container: HTMLElement, initialConfig: MenuConfig = DEFAULT_CONFIG) {
    this.state = {
      config: initialConfig,
      hotCategory: 0,
      hotSlot: -1,
      rotation: undefined,
      isOpen: true,
    };

    this.renderer = new MenuRenderer(
      container,
      { cx: this.cx, cy: this.cy },
      {
        onCategoryHover: (index) => this.selectCategory(index),
        onCategoryClick: (index) => this.selectCategory(index),
        onSlotHover: (slotIndex) => this.setHotSlot(slotIndex),
        onSlotClick: (slot, catIndex) => this.commitSlotClick(slot, catIndex),
        onAddCategoryClick: () => this.commitAddCategory(),
        onHubClick: () => this.closeMenu(),
      }
    );

    this.render();
    this.initEvents();
    this.initIpc();
  }

  public render(): void {
    this.currentLayout = this.renderer.render(this.state);
  }

  public selectCategory(index: number): void {
    if (this.state.hotCategory === index) return;
    this.state.hotCategory = index;
    this.state.hotSlot = -1;
    this.render();
  }

  public setHotSlot(slotIndex: number): void {
    if (this.state.hotSlot === slotIndex) return;
    this.state.hotSlot = slotIndex;
    this.render();
  }

  /**
   * 커서 좌표(x, y)에 따른 hitTest 및 상태 전이
   * interaction.md §4 판정 규칙 완벽 반영
   */
  public handleCursorMove(x: number, y: number): void {
    const cats = this.state.config.categories;
    const sectorCount = Math.max(4, cats.length);
    const hit = hitTest(x, y, sectorCount, { cx: this.cx, cy: this.cy });

    if (hit.zone === 'hub') {
      // 닫기 버튼 영역: 어떤 칸도 가리키지 않음
      if (this.state.hotSlot !== -1) {
        this.state.hotSlot = -1;
        this.render();
      }
      return;
    }

    if (hit.zone === 'ring') {
      // 상위 링: 각도로 분류를 고른다.
      if (hit.sector < cats.length) {
        if (this.state.hotCategory !== hit.sector) {
          this.state.hotCategory = hit.sector;
          this.state.hotSlot = -1;
          this.render();
        } else if (this.state.hotSlot !== -1) {
          this.state.hotSlot = -1;
          this.render();
        }
      } else if (cats.length < sectorCount && hit.sector === cats.length) {
        // "분류 추가" 칸 가리킴
        if (this.state.hotCategory !== -1) {
          this.state.hotCategory = -1;
          this.state.hotSlot = -1;
          this.render();
        }
      }
      return;
    }

    // zone === 'band' 또는 'outside' (r > 186):
    // 핵심 규칙: "바깥쪽에서는 분류를 바꾸지 않는다. 현재 띠 안에서 각도로 하위 칸을 고른다."
    if (this.state.hotCategory >= 0 && this.currentLayout?.sub) {
      const sub = this.currentLayout.sub;
      // 띠 전체 각도 범위 안인지 검사
      const diffFromAxis = Math.abs(
        ((((hit.angle - sub.axis) % 360) + 540) % 360) - 180
      );

      if (diffFromAxis > sub.span / 2) {
        // 띠의 각도 범위 밖이면 하위 칸 없음
        if (this.state.hotSlot !== -1) {
          this.state.hotSlot = -1;
          this.render();
        }
      } else {
        // 띠 각도 범위 내: 각 슬롯별 각도 비교
        let matchedSlotIndex = -1;
        for (const slot of sub.slots) {
          const diff = Math.abs(
            ((((hit.angle - slot.angle) % 360) + 540) % 360) - 180
          );
          if (diff <= sub.slotAngle / 2) {
            matchedSlotIndex = slot.index;
            break;
          }
        }

        if (this.state.hotSlot !== matchedSlotIndex) {
          this.state.hotSlot = matchedSlotIndex;
          this.render();
        }
      }
    }
  }

  /**
   * 현재 가리킨 대상(CommitTarget) 계산
   * interaction.md §1 & §5
   */
  public resolveTarget(): CommitTarget {
    const cats = this.state.config.categories;
    const catIdx = this.state.hotCategory;

    if (catIdx >= 0 && catIdx < cats.length) {
      const cat = cats[catIdx];
      const slotIdx = this.state.hotSlot;

      if (slotIdx >= 0 && slotIdx < cat.items.length) {
        return {
          type: 'item',
          categoryIndex: catIdx,
          itemIndex: slotIdx,
          category: cat,
          item: cat.items[slotIdx],
        };
      }

      if (slotIdx === cat.items.length) {
        return {
          type: 'add_item',
          categoryIndex: catIdx,
          category: cat,
        };
      }
    }

    return { type: 'none' };
  }

  public async commit(): Promise<CommitTarget> {
    const target = this.resolveTarget();
    invokeCommand('frontend_log', {
      msg: `commit: target resolved = ${JSON.stringify(target)}, hotCat=${this.state.hotCategory}, hotSlot=${this.state.hotSlot}`,
    });
    await emitEvent('menu:commit_result', { target });
    this.closeMenu();
    return target;
  }

  private async commitSlotClick(slot: LayoutSlot, catIndex: number): Promise<void> {
    const cat = this.state.config.categories[catIndex];
    let target: CommitTarget;

    if (slot.kind === 'add') {
      target = {
        type: 'add_item',
        categoryIndex: catIndex,
        category: cat,
      };
    } else if (slot.item) {
      target = {
        type: 'item',
        categoryIndex: catIndex,
        itemIndex: slot.index,
        category: cat,
        item: slot.item,
      };
    } else {
      target = { type: 'none' };
    }

    await emitEvent('menu:commit_result', { target });
    this.closeMenu();
  }

  private async commitAddCategory(): Promise<void> {
    const target: CommitTarget = { type: 'add_category' };
    await emitEvent('menu:commit_result', { target });
    this.closeMenu();
  }

  public closeMenu(emit: boolean = true): void {
    if (!this.state.isOpen) return;
    this.state.isOpen = false;
    this.state.hotCategory = -1;
    this.state.hotSlot = -1;
    this.render();
    if (emit) {
      emitEvent('menu:closed', { reason: 'user_action' });
    }
  }

  private initEvents(): void {
    window.addEventListener('mousemove', (e) => {
      this.handleCursorMove(e.clientX, e.clientY);
    });

    window.addEventListener('keydown', (e) => {
      if (e.key === 'Escape') {
        this.closeMenu();
      }
    });

    window.addEventListener('contextmenu', (e) => {
      e.preventDefault();
      this.closeMenu();
    });

    window.addEventListener('mousedown', (e) => {
      if (e.button === 2) {
        e.preventDefault();
        this.closeMenu();
      }
    });
  }

  private initIpc(): void {
    // Tauri IPC 이벤트 수신 바인딩
    listenEvent<MenuOpenPayload>('menu:open', (payload) => {
      invokeCommand('frontend_log', { msg: 'IPC event menu:open received!' });
      this.cx = 340;
      this.cy = 340;
      if (payload.config) {
        this.state.config = payload.config;
      }
      this.state.isOpen = true;
      this.state.hotCategory = 0;
      this.state.hotSlot = -1;
      this.render();
      invokeCommand('frontend_log', { msg: 'menu:open render() complete' });
    });

    listenEvent('menu:commit', async () => {
      await this.commit();
    });

    listenEvent('menu:close', () => {
      this.closeMenu(false);
    });

    listenEvent<MenuCursorPayload>('menu:cursor_move', (payload) => {
      this.handleCursorMove(payload.x, payload.y);
    });

    listenEvent<{ config: MenuConfig }>('menu:config_changed', (payload) => {
      this.state.config = payload.config;
      this.render();
    });
  }
}

// 브라우저 로드 시 초기화 (지연 로딩 대비 안정적 진입)
function bootstrapMenu() {
  invokeCommand('frontend_log', { msg: `bootstrapMenu: readyState=${document.readyState}` });
  const container = document.getElementById('menu');
  if (container && !(window as any).__menuController) {
    (window as any).__menuController = new MenuController(container);
    invokeCommand('frontend_log', { msg: `MenuController initialized, children=${container.children.length}` });
  } else if (!container) {
    invokeCommand('frontend_log', { msg: 'ERROR: #menu container not found in DOM' });
  }
}

if (typeof document !== 'undefined') {
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', bootstrapMenu);
  } else {
    bootstrapMenu();
  }
}
