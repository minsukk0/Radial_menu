/**
 * render.ts — 래디얼 메뉴의 SVG 및 DOM 렌더링 모듈
 * UI_designs/reference/menu-standalone.html 기반
 */

import { layout, nextRotation } from './radial-layout';
import { updateHighlightMotion } from './motion';
import { generateSvgSymbols, getCategoryDefaultIcon, getSystemIcon } from '../shared/icons';
import type {
  MenuState,
  LayoutOptions,
  LayoutSlot,
  LayoutResult,
  SystemItemConfig,
} from '../shared/types';

const NS = 'http://www.w3.org/2000/svg';

export interface RenderCallbacks {
  onCategoryHover?: (index: number) => void;
  onCategoryClick?: (index: number) => void;
  onSlotHover?: (slotIndex: number) => void;
  onSlotClick?: (slot: LayoutSlot, categoryIndex: number) => void;
  onAddCategoryClick?: () => void;
  onHubClick?: () => void;
}

export class MenuRenderer {
  private container: HTMLElement;
  private opts: LayoutOptions;
  private cx: number;
  private cy: number;

  // DOM 캐시
  private baseSvg!: SVGSVGElement;
  private dividersPath!: SVGPathElement;
  private selContainer!: HTMLElement;
  private selFill!: HTMLElement;
  private selEdge!: SVGPathElement;
  private selArc!: SVGPathElement;
  private ringContainer!: HTMLElement;
  private hubButton!: HTMLButtonElement;
  private subLayer!: HTMLElement;

  private lastShownSubKey: string | null = null;
  private callbacks: RenderCallbacks = {};

  constructor(
    container: HTMLElement,
    opts: LayoutOptions = { cx: 340, cy: 340 },
    callbacks: RenderCallbacks = {}
  ) {
    this.container = container;
    this.opts = opts;
    this.cx = opts.cx === undefined ? 340 : opts.cx;
    this.cy = opts.cy === undefined ? 340 : opts.cy;
    this.callbacks = callbacks;

    this.initDOM();
  }

  private createElement<K extends keyof HTMLElementTagNameMap>(
    tag: K,
    attrs: Record<string, string>,
    parent?: HTMLElement
  ): HTMLElementTagNameMap[K] {
    const el = document.createElement(tag);
    for (const [k, v] of Object.entries(attrs)) {
      if (k === 'text') el.textContent = v;
      else if (k === 'style') el.style.cssText = v;
      else el.setAttribute(k, v);
    }
    if (parent) parent.appendChild(el);
    return el;
  }

  private createSvgElement<K extends keyof SVGElementTagNameMap>(
    tag: K,
    attrs: Record<string, string | number>,
    parent?: SVGElement | HTMLElement
  ): SVGElementTagNameMap[K] {
    const el = document.createElementNS(NS, tag);
    for (const [k, v] of Object.entries(attrs)) {
      el.setAttribute(k, String(v));
    }
    if (parent) parent.appendChild(el);
    return el;
  }

  private initDOM(): void {
    this.container.textContent = '';
    this.container.classList.add('rm');
    this.container.style.setProperty('--rm-cx', `${this.cx}px`);
    this.container.style.setProperty('--rm-cy', `${this.cy}px`);

    // 1. 원판 바탕
    this.createElement('div', { class: 'plate' }, this.container);

    // 2. 기본 SVG 레이어 (외곽선, 구분선, 그라데이션 defs, 아이콘 심볼)
    this.baseSvg = this.createSvgElement(
      'svg',
      {
        class: 'layer',
        width: 680,
        height: 680,
        viewBox: '0 0 680 680',
        'aria-hidden': 'true',
      },
      this.container
    );

    const defs = this.createSvgElement('defs', {}, this.baseSvg);
    defs.innerHTML = `
      <radialGradient id="dv" gradientUnits="userSpaceOnUse" cx="${this.cx}" cy="${this.cy}" r="186">
        <stop offset="0.2" stop-color="#A6F1EC" stop-opacity="0"/>
        <stop offset="0.62" stop-color="#A6F1EC" stop-opacity="0.26"/>
        <stop offset="1" stop-color="#A6F1EC" stop-opacity="0.05"/>
      </radialGradient>
      <radialGradient id="edge" gradientUnits="userSpaceOnUse" cx="${this.cx}" cy="${this.cy}" r="186">
        <stop offset="0.25" stop-color="#FFFFFF" stop-opacity="0"/>
        <stop offset="1" stop-color="#FFFFFF" stop-opacity="0.42"/>
      </radialGradient>
      ${generateSvgSymbols('i-')}
    `;

    this.createSvgElement(
      'circle',
      {
        class: 'aqua',
        cx: this.cx,
        cy: this.cy,
        r: 186,
        'stroke-opacity': 0.55,
      },
      this.baseSvg
    );

    this.dividersPath = this.createSvgElement(
      'path',
      {
        fill: 'none',
        stroke: 'url(#dv)',
        'stroke-width': 0.6,
      },
      this.baseSvg
    );

    // 3. 선택 하이라이트 레이어
    this.selContainer = this.createElement(
      'div',
      { class: 'layer sel off' },
      this.container
    );
    this.selFill = this.createElement('div', { class: 'fill' }, this.selContainer);

    const selSvg = this.createSvgElement(
      'svg',
      {
        width: 680,
        height: 680,
        viewBox: '0 0 680 680',
        'aria-hidden': 'true',
        style: 'position:absolute;left:0;top:0',
      },
      this.selContainer
    );

    this.selEdge = this.createSvgElement(
      'path',
      {
        fill: 'none',
        stroke: 'url(#edge)',
        'stroke-width': 1,
      },
      selSvg
    );

    this.selArc = this.createSvgElement(
      'path',
      { class: 'arc' },
      selSvg
    );

    // 4. 상위 링 버튼 레이어
    this.ringContainer = this.createElement(
      'div',
      { class: 'layer', style: 'pointer-events:auto' },
      this.container
    );

    // 5. 닫기 허브 버튼
    this.hubButton = this.createElement(
      'button',
      { class: 'hub', 'aria-label': '닫기' },
      this.container
    );
    this.hubButton.innerHTML = `
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" aria-hidden="true">
        <path d="M7 7l10 10M17 7 7 17"/>
      </svg>
    `;
    this.hubButton.addEventListener('click', (e) => {
      e.stopPropagation();
      this.callbacks.onHubClick?.();
    });

    // 6. 하위 링 레이어
    this.subLayer = this.createElement('div', { class: 'layer' }, this.container);
  }

  private renderIcon(iconName: string, size: number, parent: HTMLElement): SVGElement {
    const isImage =
      iconName.startsWith('data:image/') ||
      iconName.startsWith('http://') ||
      iconName.startsWith('https://') ||
      /\.(png|jpe?g|svg|ico|webp)$/i.test(iconName);

    const s = this.createSvgElement(
      'svg',
      {
        width: size,
        height: size,
        viewBox: '0 0 24 24',
        fill: 'none',
        stroke: 'currentColor',
        'stroke-width': 1.5,
        'stroke-linecap': 'round',
        'stroke-linejoin': 'round',
        'aria-hidden': 'true',
      },
      parent
    );

    if (isImage) {
      this.createSvgElement(
        'image',
        {
          href: iconName,
          x: 2,
          y: 2,
          width: 20,
          height: 20,
          preserveAspectRatio: 'xMidYMid meet',
        },
        s
      );
    } else {
      this.createSvgElement('use', { href: `#i-${iconName}` }, s);
    }
    return s;
  }

  private renderPlus(size: number, parent: HTMLElement): SVGElement {
    const s = this.createSvgElement(
      'svg',
      {
        width: size,
        height: size,
        viewBox: '0 0 24 24',
        fill: 'none',
        stroke: 'currentColor',
        'stroke-width': 1.3,
        'stroke-linecap': 'round',
        'aria-hidden': 'true',
      },
      parent
    );
    this.createSvgElement(
      'circle',
      { cx: 12, cy: 12, r: 9.5, 'stroke-dasharray': '2.8 3' },
      s
    );
    this.createSvgElement('path', { d: 'M12 8v8M8 12h8' }, s);
    return s;
  }

  private createLabel(x: number, y: number, parent: HTMLElement): HTMLElement {
    return this.createElement(
      'span',
      { class: 'lab', style: `left:${x}px;top:${y}px` },
      parent
    );
  }

  /**
   * 메뉴 전체 렌더링 업데이트
   */
  public render(state: MenuState): LayoutResult {
    const cats = state.config.categories;
    const hot = state.hotCategory >= cats.length ? -1 : state.hotCategory;
    const L = layout(cats, hot, this.opts);

    // 1. 구분선 업데이트
    this.dividersPath.setAttribute('d', L.dividers);

    // 2. 선택 하이라이트 업데이트
    this.selFill.style.clipPath = `path('${L.selection.wedge}')`;
    this.selEdge.setAttribute('d', L.selection.edge);
    this.selArc.setAttribute('d', L.selection.arc);

    if (L.selection.visible && L.selection.axis !== null) {
      state.rotation = nextRotation(state.rotation, L.selection.axis);
      updateHighlightMotion(this.selContainer, state.rotation, true);
    } else {
      updateHighlightMotion(this.selContainer, undefined, false);
    }

    // 3. 상위 칸 렌더링
    this.ringContainer.textContent = '';
    L.sectors.forEach((s) => {
      const isSelected = s.selected;
      const b = this.createElement(
        'button',
        {
          class: `sec${isSelected ? ' on' : ''}`,
          style: `clip-path:path('${s.path}')`,
          'aria-label': s.category.label,
        },
        this.ringContainer
      );

      const lab = this.createLabel(s.labelX, s.labelY, b);
      const iconName = s.category.icon || getCategoryDefaultIcon(s.category.kind);
      this.renderIcon(iconName, 24, lab);
      this.createElement('span', { text: s.category.label }, lab);

      b.addEventListener('mouseenter', () => this.callbacks.onCategoryHover?.(s.index));
      b.addEventListener('click', (e) => {
        e.stopPropagation();
        this.callbacks.onCategoryClick?.(s.index);
      });
    });

    if (L.addSector) {
      const a = this.createElement(
        'button',
        {
          class: 'add',
          'aria-label': '분류 추가',
          style: `clip-path:path('${L.addSector.path}')`,
        },
        this.ringContainer
      );
      const al = this.createLabel(L.addSector.labelX, L.addSector.labelY, a);
      this.renderPlus(30, al);
      this.createElement('span', { text: '분류 추가' }, al);

      a.addEventListener('click', (e) => {
        e.stopPropagation();
        this.callbacks.onAddCategoryClick?.();
      });
    }

    // 4. 하위 띠 렌더링 (가리키는 분류 또는 항목 수가 바뀔 때만 새로 그림)
    const key = L.sub
      ? `${hot}:${L.sub.slotCount}:${L.sub.slots.length}`
      : null;

    if (key !== this.lastShownSubKey) {
      this.lastShownSubKey = key;
      this.subLayer.textContent = '';

      if (L.sub) {
        this.createElement(
          'div',
          { class: 'layer band pop', style: `clip-path:path('${L.sub.band}')` },
          this.subLayer
        );

        const lines = this.createSvgElement(
          'svg',
          {
            class: 'layer pop',
            width: 680,
            height: 680,
            viewBox: '0 0 680 680',
            'aria-hidden': 'true',
          },
          this.subLayer
        );
        this.createSvgElement(
          'path',
          { class: 'aqua', d: L.sub.band, 'stroke-opacity': 0.28 },
          lines
        );
        this.createSvgElement(
          'path',
          { class: 'aqua', d: L.sub.dividers, 'stroke-opacity': 0.16 },
          lines
        );

        const fan = this.createElement('div', { class: 'fan' }, this.subLayer);
        L.sub.slots.forEach((slot) => {
          const isAdd = slot.kind === 'add';
          const b = this.createElement(
            isAdd ? 'a' : 'button',
            {
              class: `sub${isAdd ? ' plus' : ''}`,
              style: `clip-path:path('${slot.path}');animation-delay:${slot.delayMs}ms`,
              'data-slot-index': String(slot.index),
            },
            fan
          );

          const lab = this.createLabel(slot.labelX, slot.labelY, b);
          if (isAdd) {
            b.setAttribute('href', '#add');
            b.setAttribute('aria-label', '이 분류에 항목 추가');
            this.renderPlus(20, lab);
            this.createElement('span', { text: '추가' }, lab);
          } else if (slot.item) {
            b.setAttribute('aria-label', slot.item.label);
            if (slot.item.icon) {
              this.renderIcon(slot.item.icon, 20, lab);
            } else if ('mono' in slot.item && slot.item.mono) {
              this.createElement('span', { class: 'mono', text: slot.item.mono }, lab);
            } else {
              let itemIcon: string | undefined;
              if ('fn' in slot.item) {
                itemIcon = getSystemIcon((slot.item as SystemItemConfig).fn);
              }
              this.renderIcon(itemIcon || 'window', 20, lab);
            }
            this.createElement('span', { text: slot.item.label }, lab);
          }

          b.addEventListener('mouseenter', () => this.callbacks.onSlotHover?.(slot.index));
          b.addEventListener('click', (e) => {
            e.stopPropagation();
            this.callbacks.onSlotClick?.(slot, hot);
          });
        });
      }
    }

    // 5. 하위 칸 활성 상태(active/hover) 시각 반영
    if (L.sub) {
      const slotButtons = this.subLayer.querySelectorAll('.sub');
      slotButtons.forEach((btn, idx) => {
        btn.classList.toggle('active', idx === state.hotSlot);
      });
    }

    return L;
  }
}
