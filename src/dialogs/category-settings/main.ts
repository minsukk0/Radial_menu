import { invokeCommand, listenEvent } from '../../shared/ipc';
import type { CategoryConfig, CategoryKind, MenuConfig } from '../../shared/types';

// Kind metadata
export const KIND_METADATA: Record<CategoryKind, { label: string; defaultIcon: string }> = {
  app: { label: '앱', defaultIcon: 'apps' },
  terminal: { label: '터미널', defaultIcon: 'terminal' },
  folder: { label: '폴더·주소', defaultIcon: 'folder' },
  system: { label: '시스템', defaultIcon: 'power' },
  empty: { label: '빈칸', defaultIcon: 'add' },
};

export const PRESET_ICONS = ['apps', 'terminal', 'folder', 'power', 'globe', 'window', 'paste', 'play'];
export const MAX_CATEGORIES = 8;

// Relative luminance calculation for onAccent text color
export function getOnAccentColor(accentHex: string): string {
  const hex = /^#[0-9a-f]{6}$/i.test(accentHex) ? accentHex.slice(1) : null;
  if (!hex) return '#04121C';
  const ch = [0, 2, 4].map((i) => {
    const v = parseInt(hex.slice(i, i + 2), 16) / 255;
    return v <= 0.03928 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4);
  });
  const lum = 0.2126 * ch[0] + 0.7152 * ch[1] + 0.0722 * ch[2];
  return lum < 0.2 ? '#FFFFFF' : '#04121C';
}

// Geometry helper for 200x200 radial preview
export const fmt = (v: number): string => String(Math.round(v * 10) / 10);
export const calcXY = (r: number, deg: number): [number, number] => {
  const a = (deg * Math.PI) / 180;
  return [100 + r * Math.sin(a), 100 - r * Math.cos(a)];
};
export const calcP = (r: number, deg: number): string => {
  const [x, y] = calcXY(r, deg);
  return `${fmt(x)} ${fmt(y)}`;
};

export class CategorySettingsController {
  public categories: CategoryConfig[] = [];
  public selectedIndex = 0;
  public accentColor = '#1E9BFF';
  public isAddingMode = false;
  public confirmDeleteId: string | null = null;
  public draggedIndex: number | null = null;
  public autoSaveTimer: ReturnType<typeof setTimeout> | null = null;
  public statusResetTimer: ReturnType<typeof setTimeout> | null = null;
  private eventsInitialized = false;

  constructor(private root: Document | HTMLElement = document) {}

  public getElements() {
    return {
      btnClose: this.root.querySelector('#btn-close') as HTMLButtonElement | null,
      btnCloseDialog: this.root.querySelector('#btn-close-dialog') as HTMLButtonElement | null,
      saveStatus: this.root.querySelector('#save-status') as HTMLSpanElement | null,
      pvSel: this.root.querySelector('#pv-sel') as SVGPathElement | null,
      pvLines: this.root.querySelector('#pv-lines') as SVGPathElement | null,
      pvArc: this.root.querySelector('#pv-arc') as SVGPathElement | null,
      pvIconsLayer: this.root.querySelector('#pv-icons-layer') as HTMLDivElement | null,
      pvCaption: this.root.querySelector('#pv-caption') as HTMLDivElement | null,
      inpName: this.root.querySelector('#inp-name') as HTMLInputElement | null,
      iconGrid: this.root.querySelector('#icon-grid') as HTMLDivElement | null,
      kindNote: this.root.querySelector('#kind-note') as HTMLDivElement | null,
      listTitle: this.root.querySelector('#list-title') as HTMLSpanElement | null,
      categoryList: this.root.querySelector('#category-list') as HTMLDivElement | null,
      addBtnWrap: this.root.querySelector('#add-btn-wrap') as HTMLDivElement | null,
      btnOpenAdd: this.root.querySelector('#btn-open-add') as HTMLButtonElement | null,
      addMaxNote: this.root.querySelector('#add-max-note') as HTMLSpanElement | null,
      addPickerBox: this.root.querySelector('#add-picker-box') as HTMLDivElement | null,
      btnCancelAdd: this.root.querySelector('#btn-cancel-add') as HTMLButtonElement | null,
      btnCancel: this.root.querySelector('#btn-cancel') as HTMLButtonElement | null,
      btnSave: this.root.querySelector('#btn-save') as HTMLButtonElement | null,
    };
  }

  public ensureMinFourSlots() {
    while (this.categories.length < 4) {
      const i = this.categories.length;
      this.categories.push({
        id: `empty-${i}`,
        kind: 'empty',
        label: '빈칸',
        icon: 'add',
        items: []
      });
    }
  }

  public applyAccent(color: string) {
    this.accentColor = color;
    document.documentElement.style.setProperty('--accent', color);
    const els = this.getElements();
    if (els.btnSave) {
      els.btnSave.style.color = getOnAccentColor(color);
    }
  }

  public renderPreview() {
    const els = this.getElements();
    const count = this.categories.length;
    const n = Math.max(4, count);
    const step = 360 / n;
    const half = step / 2;
    const selIdx = Math.max(0, Math.min(this.selectedIndex, count - 1));
    const ax = selIdx * step;
    const currentCat = this.categories[selIdx];
    const isSelectedValid = currentCat && currentCat.kind !== 'empty';

    // 1. Divider lines
    let lines = '';
    for (let i = 0; i < n; i++) {
      lines += `M${calcP(20, (i + 0.5) * step)}L${calcP(92, (i + 0.5) * step)}`;
    }
    if (els.pvLines) els.pvLines.setAttribute('d', lines);

    // 2. Selected sector highlight (only for non-empty categories)
    if (count > 0 && isSelectedValid && els.pvSel) {
      const selD = `M${calcP(20, ax - half)}L${calcP(92, ax - half)}A92 92 0 0 1 ${calcP(92, ax + half)}L${calcP(20, ax + half)}A20 20 0 0 0 ${calcP(20, ax - half)}Z`;
      els.pvSel.setAttribute('d', selD);
      els.pvSel.style.display = 'block';
    } else if (els.pvSel) {
      els.pvSel.style.display = 'none';
    }

    // 3. Selected sector cyan inner arc (r=17, only for non-empty categories)
    if (count > 0 && isSelectedValid && els.pvArc) {
      const arcD = `M${calcP(17, ax - half)}A17 17 0 0 1 ${calcP(17, ax + half)}`;
      els.pvArc.setAttribute('d', arcD);
      els.pvArc.style.display = 'block';
    } else if (els.pvArc) {
      els.pvArc.style.display = 'none';
    }

    // 4. Sector icon overlays
    if (els.pvIconsLayer) {
      els.pvIconsLayer.innerHTML = '';
      for (let i = 0; i < n; i++) {
        const c = this.categories[i];
        const [x, y] = calcXY(58, i * step);
        const isSelected = i === selIdx;
        const span = document.createElement('span');

        if (!c || c.kind === 'empty') {
          span.className = 'pvi blank';
          span.style.left = `${fmt(x - 8)}px`;
          span.style.top = `${fmt(y - 8)}px`;
          span.innerHTML = `<svg width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" style="display: block"><use href="#a-add"></use></svg>`;
        } else {
          span.className = `pvi ${isSelected ? 'on' : ''}`;
          span.style.left = `${fmt(x - 8)}px`;
          span.style.top = `${fmt(y - 8)}px`;
          const iconName = c.icon || KIND_METADATA[c.kind]?.defaultIcon || 'apps';
          span.innerHTML = `<svg width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" style="display: block"><use href="#a-${iconName}"></use></svg>`;
        }
        els.pvIconsLayer?.appendChild(span);
      }
    }

    // 5. Preview Caption
    if (els.pvCaption) {
      const validCount = this.categories.filter((c) => c.kind !== 'empty').length;
      const emptyCount = n - validCount;
      els.pvCaption.textContent = `분류 ${validCount}개 · ${n}칸` + (emptyCount > 0 ? ` (빈칸 ${emptyCount})` : '');
    }
  }

  public renderEditor() {
    const els = this.getElements();
    const current = this.categories[this.selectedIndex];
    if (!current || current.kind === 'empty') {
      if (els.inpName) {
        els.inpName.value = '';
        els.inpName.disabled = true;
      }
      if (els.kindNote) {
        els.kindNote.textContent = current ? '종류는 빈칸입니다. 만든 뒤에는 바꿀 수 없습니다.' : '';
      }
      if (els.iconGrid) {
        els.iconGrid.querySelectorAll('.ico').forEach((btn) => btn.classList.remove('on'));
      }
      return;
    }

    if (els.inpName) {
      els.inpName.disabled = false;
      els.inpName.value = current.label;
    }

    const currentIcon = current.icon || KIND_METADATA[current.kind]?.defaultIcon || 'apps';
    if (els.iconGrid) {
      els.iconGrid.querySelectorAll<HTMLButtonElement>('.ico').forEach((btn) => {
        const ico = btn.getAttribute('data-icon');
        if (ico === currentIcon) {
          btn.classList.add('on');
        } else {
          btn.classList.remove('on');
        }
      });
    }

    if (els.kindNote) {
      const kindLabel = KIND_METADATA[current.kind]?.label || '앱';
      els.kindNote.textContent = `종류는 ${kindLabel}입니다. 만든 뒤에는 바꿀 수 없습니다.`;
    }
  }

  public renderList() {
    const els = this.getElements();
    const count = this.categories.length;
    const validCount = this.categories.filter((c) => c.kind !== 'empty').length;

    if (els.listTitle) {
      els.listTitle.textContent = `분류 ${validCount}개`;
    }

    if (!els.categoryList) return;
    els.categoryList.innerHTML = '';

    this.categories.forEach((c, i) => {
      const isSelected = i === this.selectedIndex;
      const isConfirming = this.confirmDeleteId === (c.id || String(i));
      const itemCount = c.items ? c.items.length : 0;
      const isEmpty = c.kind === 'empty';
      const kindLabel = KIND_METADATA[c.kind]?.label || '앱';
      const currentIcon = c.icon || KIND_METADATA[c.kind]?.defaultIcon || (isEmpty ? 'add' : 'apps');
      const isDelDisabled = isEmpty ? count <= 4 : validCount <= 1;

      if (isConfirming) {
        // Deletion confirmation row
        const warnRow = document.createElement('div');
        warnRow.className = 'row warn';
        warnRow.innerHTML = `
          <span style="flex: 1; min-width: 0; font-size: 13px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap">
            "${c.label}" 분류와 그 안의 항목 ${itemCount}개를 함께 지울까요?
          </span>
          <button class="btn sm" id="btn-cancel-del-${i}">취소</button>
          <button class="btn sm danger" id="btn-confirm-del-${i}">지우기</button>
        `;

        warnRow.querySelector(`#btn-cancel-del-${i}`)?.addEventListener('click', (e) => {
          e.stopPropagation();
          this.confirmDeleteId = null;
          this.renderList();
        });

        warnRow.querySelector(`#btn-confirm-del-${i}`)?.addEventListener('click', (e) => {
          e.stopPropagation();
          this.deleteCategoryAtIndex(i);
        });

        els.categoryList?.appendChild(warnRow);
      } else {
        // Normal or Empty Category row
        const row = document.createElement('div');
        row.className = `row ${isEmpty ? 'empty' : ''} ${isSelected ? 'on' : ''}`.trim();
        row.setAttribute('data-index', String(i));
        row.setAttribute('data-id', c.id || String(i));

        row.innerHTML = `
          <div class="rowmain" role="button" tabindex="0" aria-label="${isEmpty ? '빈칸 슬롯 선택' : `${c.label} 분류 선택`}">
            <span class="grip" aria-hidden="true">
              <svg width="10" height="16" viewBox="0 0 10 16" fill="currentColor">
                <circle cx="2.5" cy="3" r="1.3"></circle>
                <circle cx="7.5" cy="3" r="1.3"></circle>
                <circle cx="2.5" cy="8" r="1.3"></circle>
                <circle cx="7.5" cy="8" r="1.3"></circle>
                <circle cx="2.5" cy="13" r="1.3"></circle>
                <circle cx="7.5" cy="13" r="1.3"></circle>
              </svg>
            </span>
            <span class="num">${i + 1}</span>
            <span class="cico">
              <svg width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <use href="#a-${isEmpty ? 'add' : currentIcon}"></use>
              </svg>
            </span>
            <span class="cname">${isEmpty ? '(빈칸)' : c.label}</span>
            <span class="kind">${isEmpty ? '빈칸' : kindLabel}</span>
            <span class="cnt">${isEmpty ? '항목 없음' : (itemCount > 0 ? `항목 ${itemCount}개` : '항목 없음')}</span>
          </div>
          <button class="ib" type="button" aria-label="${isEmpty ? '빈칸' : c.label} 위로 옮기기" ${i === 0 ? 'disabled' : ''} id="btn-up-${i}">
            <svg width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <use href="#a-up"></use>
            </svg>
          </button>
          <button class="ib" type="button" aria-label="${isEmpty ? '빈칸' : c.label} 아래로 옮기기" ${i === count - 1 ? 'disabled' : ''} id="btn-down-${i}">
            <svg width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <use href="#a-down"></use>
            </svg>
          </button>
          <button class="ib del" type="button" aria-label="${isEmpty ? '빈칸' : c.label} 지우기" ${isDelDisabled ? 'disabled' : ''} id="btn-del-${i}">
            <svg width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <use href="#a-trash"></use>
            </svg>
          </button>
        `;

        // Row Selection
        const rowmain = row.querySelector('.rowmain');
        let wasDragged = false;
        const handleSelect = () => {
          if (wasDragged) {
            wasDragged = false;
            return;
          }
          this.selectedIndex = i;
          this.confirmDeleteId = null;
          this.renderAll();
        };
        rowmain?.addEventListener('click', handleSelect);
        rowmain?.addEventListener('keydown', (e: Event) => {
          const ke = e as KeyboardEvent;
          if (ke.key === 'Enter' || ke.key === ' ') {
            ke.preventDefault();
            handleSelect();
          }
        });

        // Move Up
        row.querySelector(`#btn-up-${i}`)?.addEventListener('click', (e) => {
          e.stopPropagation();
          if (i > 0) this.swapCategories(i, i - 1);
        });

        // Move Down
        row.querySelector(`#btn-down-${i}`)?.addEventListener('click', (e) => {
          e.stopPropagation();
          if (i < count - 1) this.swapCategories(i, i + 1);
        });

        // Delete action
        row.querySelector(`#btn-del-${i}`)?.addEventListener('click', (e) => {
          e.stopPropagation();
          if (isDelDisabled) return;

          if (isEmpty) {
            if (count > 4) {
              this.deleteCategoryAtIndex(i);
            }
            return;
          }

          if (validCount <= 1) return;
          if (itemCount > 0) {
            this.confirmDeleteId = c.id || String(i);
            this.selectedIndex = i;
            this.renderAll();
          } else {
            this.deleteCategoryAtIndex(i);
          }
        });

        // Drag & Drop (HTML5 + Pointer Events)
        this.attachDragAndDropHandlers(row, i, () => {
          wasDragged = true;
        });

        els.categoryList?.appendChild(row);
      }
    });

    // Add Category Section State
    const emptyIndex = this.categories.findIndex((c) => c.kind === 'empty');
    const canAdd = emptyIndex !== -1 || count < MAX_CATEGORIES;

    if (els.addBtnWrap && els.btnOpenAdd && els.addMaxNote) {
      if (!canAdd) {
        els.btnOpenAdd.disabled = true;
        els.addMaxNote.textContent = '분류는 8개까지 만들 수 있습니다.';
        els.addBtnWrap.style.display = 'flex';
        if (els.addPickerBox) els.addPickerBox.style.display = 'none';
        this.isAddingMode = false;
      } else {
        els.btnOpenAdd.disabled = false;
        els.addMaxNote.textContent = '';
        if (this.isAddingMode) {
          els.addBtnWrap.style.display = 'none';
          if (els.addPickerBox) els.addPickerBox.style.display = 'block';
        } else {
          els.addBtnWrap.style.display = 'flex';
          if (els.addPickerBox) els.addPickerBox.style.display = 'none';
        }
      }
    }
  }

  public executeMove(sourceIndex: number, targetIndex: number) {
    if (sourceIndex === targetIndex) return;
    const sourceCat = this.categories[sourceIndex];
    const targetCat = this.categories[targetIndex];
    if (!sourceCat || !targetCat) return;

    // 만약 소스나 타깃 중 하나라도 빈칸(empty)이면 -> 두 슬롯의 위치를 SWAP!
    if (sourceCat.kind === 'empty' || targetCat.kind === 'empty') {
      this.categories[sourceIndex] = targetCat;
      this.categories[targetIndex] = sourceCat;
    } else {
      // 둘 다 일반 카테고리인 경우 -> splice로 순서 재배치
      const [moved] = this.categories.splice(sourceIndex, 1);
      this.categories.splice(targetIndex, 0, moved);
    }

    this.selectedIndex = targetIndex;
    this.confirmDeleteId = null;
    this.draggedIndex = null;
    this.renderAll();
    this.triggerAutoSave(true);
  }

  public attachDragAndDropHandlers(row: HTMLDivElement, index: number, onDragStartCallback?: () => void) {
    // Pointer Events 기반 직접 드래그 (WebView2 OLE DND 차단 완벽 대응)
    const rowmain = row.querySelector('.rowmain') as HTMLElement | null;
    const dragTarget = rowmain || row;

    let startX = 0;
    let startY = 0;
    let isPointerDragging = false;
    let dragClone: HTMLDivElement | null = null;

    dragTarget.addEventListener('pointerdown', (e: PointerEvent) => {
      if (e.button !== 0) return; // 좌클릭만 허용
      startX = e.clientX;
      startY = e.clientY;
      isPointerDragging = false;
      this.draggedIndex = index;

      dragTarget.setPointerCapture(e.pointerId);

      const onPointerMove = (moveEv: PointerEvent) => {
        const dist = Math.hypot(moveEv.clientX - startX, moveEv.clientY - startY);
        if (!isPointerDragging && dist > 4) {
          isPointerDragging = true;
          onDragStartCallback?.();
          row.classList.add('dragging');

          // 플로팅 미리보기 클론 생성
          dragClone = row.cloneNode(true) as HTMLDivElement;
          dragClone.classList.add('drag-floating-clone');
          dragClone.style.position = 'fixed';
          dragClone.style.left = `${moveEv.clientX - 20}px`;
          dragClone.style.top = `${moveEv.clientY - 20}px`;
          dragClone.style.width = `${row.offsetWidth}px`;
          dragClone.style.pointerEvents = 'none';
          dragClone.style.zIndex = '99999';
          dragClone.style.opacity = '0.9';
          dragClone.style.border = '1px solid var(--accent)';
          dragClone.style.boxShadow = '0 12px 30px rgba(0, 0, 0, 0.6)';
          dragClone.style.borderRadius = '10px';
          dragClone.style.background = 'rgba(11, 17, 20, 0.96)';
          document.body.appendChild(dragClone);
          document.body.style.cursor = 'grabbing';
        }

        if (isPointerDragging) {
          moveEv.preventDefault();
          if (dragClone) {
            dragClone.style.left = `${moveEv.clientX - 20}px`;
            dragClone.style.top = `${moveEv.clientY - 20}px`;
          }

          const elements = document.elementsFromPoint(moveEv.clientX, moveEv.clientY);
          const targetRow = elements.find(
            (el) => el.classList.contains('row') && el !== dragClone && el !== row
          ) as HTMLDivElement | undefined;

          this.clearDragOverIndicators();
          if (targetRow) {
            targetRow.classList.add('drag-over');
          }
        }
      };

      const onPointerUp = (upEv: PointerEvent) => {
        dragTarget.releasePointerCapture(upEv.pointerId);
        dragTarget.removeEventListener('pointermove', onPointerMove);
        dragTarget.removeEventListener('pointerup', onPointerUp);
        dragTarget.removeEventListener('pointercancel', onPointerUp);
        document.body.style.cursor = '';

        if (dragClone) {
          dragClone.remove();
          dragClone = null;
        }

        row.classList.remove('dragging');
        this.clearDragOverIndicators();

        if (isPointerDragging) {
          upEv.preventDefault();
          upEv.stopPropagation();

          const elements = document.elementsFromPoint(upEv.clientX, upEv.clientY);
          const targetRow = elements.find(
            (el) => el.classList.contains('row') && el !== row
          ) as HTMLDivElement | undefined;

          if (targetRow) {
            const targetIndexStr = targetRow.getAttribute('data-index');
            if (targetIndexStr !== null) {
              const targetIndex = parseInt(targetIndexStr, 10);
              if (!isNaN(targetIndex) && targetIndex !== index) {
                this.executeMove(index, targetIndex);
              }
            }
          }
          this.draggedIndex = null;
        }
      };

      dragTarget.addEventListener('pointermove', onPointerMove);
      dragTarget.addEventListener('pointerup', onPointerUp);
      dragTarget.addEventListener('pointercancel', onPointerUp);
    });
  }

  public clearDragOverIndicators() {
    const els = this.getElements();
    els.categoryList?.querySelectorAll('.row').forEach((r) => {
      r.classList.remove('drag-over', 'drag-over-top', 'drag-over-bottom');
    });
  }

  public swapCategories(a: number, b: number) {
    const temp = this.categories[a];
    this.categories[a] = this.categories[b];
    this.categories[b] = temp;
    this.selectedIndex = b;
    this.confirmDeleteId = null;
    this.renderAll();
    this.triggerAutoSave(true);
  }

  public deleteCategoryAtIndex(idx: number) {
    const cat = this.categories[idx];
    if (!cat) return;
    const validCount = this.categories.filter((c) => c.kind !== 'empty').length;
    if (cat.kind !== 'empty' && validCount <= 1) return;

    if (this.categories.length <= 4) {
      if (cat.kind !== 'empty') {
        this.categories[idx] = {
          id: `empty-${idx}`,
          kind: 'empty',
          label: '빈칸',
          icon: 'add',
          items: []
        };
      }
    } else {
      this.categories.splice(idx, 1);
      this.ensureMinFourSlots();
    }

    this.confirmDeleteId = null;
    if (this.selectedIndex >= this.categories.length) {
      this.selectedIndex = this.categories.length - 1;
    }
    this.renderAll();
    this.triggerAutoSave(true);
  }

  public renderAll() {
    if (this.categories.length === 0) {
      this.selectedIndex = 0;
    } else {
      this.selectedIndex = Math.max(0, Math.min(this.selectedIndex, this.categories.length - 1));
    }
    this.renderPreview();
    this.renderEditor();
    this.renderList();
  }

  public setupEvents() {
    if (this.eventsInitialized) return;
    this.eventsInitialized = true;

    const els = this.getElements();

    // Close buttons
    const handleClose = async () => {
      if (this.autoSaveTimer) {
        clearTimeout(this.autoSaveTimer);
        this.autoSaveTimer = null;
        await this.saveNow();
      }
      await this.closeDialog();
    };
    els.btnClose?.addEventListener('click', handleClose);
    els.btnCloseDialog?.addEventListener('click', handleClose);
    els.btnCancel?.addEventListener('click', handleClose);

    // Keyboard ESC
    window.addEventListener('keydown', async (e) => {
      if (e.key === 'Escape') {
        if (this.isAddingMode) {
          this.isAddingMode = false;
          this.renderList();
        } else if (this.confirmDeleteId !== null) {
          this.confirmDeleteId = null;
          this.renderList();
        } else {
          await handleClose();
        }
      }
    });

    // Name Input real-time sync
    els.inpName?.addEventListener('input', () => {
      const current = this.categories[this.selectedIndex];
      if (!current || current.kind === 'empty' || !els.inpName) return;
      const newName = (els.inpName.value || '').slice(0, 10);
      current.label = newName;

      // Fast-sync list row without rebuilding DOM to retain input focus
      const rowEl = els.categoryList?.querySelector<HTMLElement>(`[data-index="${this.selectedIndex}"] .cname`);
      if (rowEl) {
        rowEl.textContent = newName;
      }
      this.triggerAutoSave(false);
    });

    // Preset Icon click
    els.iconGrid?.querySelectorAll<HTMLButtonElement>('.ico').forEach((btn) => {
      btn.addEventListener('click', () => {
        const current = this.categories[this.selectedIndex];
        if (!current || current.kind === 'empty') return;
        const ico = btn.getAttribute('data-icon');
        if (ico) {
          current.icon = ico;
          this.renderPreview();
          this.renderEditor();
          this.renderList();
          this.triggerAutoSave(true);
        }
      });
    });

    // "+ 분류 추가" open
    els.btnOpenAdd?.addEventListener('click', () => {
      const emptyIndex = this.categories.findIndex((c) => c.kind === 'empty');
      if (emptyIndex === -1 && this.categories.length >= MAX_CATEGORIES) return;
      this.isAddingMode = true;
      this.confirmDeleteId = null;
      this.renderList();
    });

    // Cancel adding category
    els.btnCancelAdd?.addEventListener('click', () => {
      this.isAddingMode = false;
      this.renderList();
    });

    // Kind tiles in add picker
    els.addPickerBox?.querySelectorAll<HTMLButtonElement>('.tile').forEach((btn) => {
      btn.addEventListener('click', () => {
        const kind = btn.getAttribute('data-kind') as CategoryKind;
        if (!kind) return;

        const emptyIndex = this.categories.findIndex((c) => c.kind === 'empty');
        if (emptyIndex === -1 && this.categories.length >= MAX_CATEGORIES) return;

        const baseLabel = KIND_METADATA[kind]?.label || '앱';
        const existingNames = this.categories
          .filter((c) => c.kind !== 'empty')
          .map((c) => c.label);
        let newName = baseLabel;
        for (let n = 2; existingNames.includes(newName); n++) {
          newName = `${baseLabel} ${n}`;
        }

        const defaultIcon = KIND_METADATA[kind]?.defaultIcon || 'apps';
        const newId = `${kind}-${Date.now().toString(36)}-${Math.floor(Math.random() * 1000)}`;

        const newCategory: CategoryConfig = {
          id: newId,
          kind,
          label: newName,
          icon: defaultIcon,
          items: []
        };

        if (emptyIndex !== -1) {
          // 첫 번째 빈칸 슬롯을 새 분류로 교체!
          this.categories[emptyIndex] = newCategory;
          this.selectedIndex = emptyIndex;
        } else {
          // 빈칸이 없고 4칸 이상인 경우 목록 끝에 추가
          this.categories.push(newCategory);
          this.selectedIndex = this.categories.length - 1;
        }

        this.isAddingMode = false;
        this.confirmDeleteId = null;
        this.renderAll();
        this.triggerAutoSave(true);

        const currentEls = this.getElements();
        currentEls.inpName?.focus();
        currentEls.inpName?.select();
      });
    });

    // Save Categories Button (legacy if present)
    els.btnSave?.addEventListener('click', async () => {
      await this.saveCategories();
    });
  }

  public triggerAutoSave(immediate: boolean = false) {
    if (this.autoSaveTimer) {
      clearTimeout(this.autoSaveTimer);
      this.autoSaveTimer = null;
    }

    if (immediate) {
      this.saveNow();
    } else {
      this.autoSaveTimer = setTimeout(() => {
        this.saveNow();
      }, 250);
    }
  }

  public async saveNow() {
    if (this.autoSaveTimer) {
      clearTimeout(this.autoSaveTimer);
      this.autoSaveTimer = null;
    }

    const validCount = this.categories.filter((c) => c.kind !== 'empty').length;
    if (validCount === 0) {
      return;
    }

    // Ensure non-empty labels and icons
    this.categories.forEach((c) => {
      if (!c.label.trim()) {
        c.label = KIND_METADATA[c.kind]?.label || '분류';
      }
      if (!c.icon) {
        c.icon = KIND_METADATA[c.kind]?.defaultIcon || (c.kind === 'empty' ? 'add' : 'apps');
      }
    });

    try {
      await invokeCommand('save_categories', { categories: this.categories });

      const els = this.getElements();
      if (els.saveStatus) {
        els.saveStatus.style.opacity = '1';
        if (this.statusResetTimer) {
          clearTimeout(this.statusResetTimer);
        }
        this.statusResetTimer = setTimeout(() => {
          if (els.saveStatus) {
            els.saveStatus.style.opacity = '0';
          }
          this.statusResetTimer = null;
        }, 1000);
      }
    } catch (err) {
      console.error('Failed to auto-save categories:', err);
    }
  }

  public async closeDialog() {
    try {
      await invokeCommand('close_dialog_window', { dialogLabel: 'category-settings' });
    } catch {
      window.close();
    }
  }

  public async saveCategories() {
    await this.saveNow();
    await this.closeDialog();
  }

  public async loadConfig() {
    try {
      const config = await invokeCommand<MenuConfig>('get_menu_config');
      if (config) {
        if (config.accent) {
          this.applyAccent(config.accent);
        }
        if (Array.isArray(config.categories) && config.categories.length > 0) {
          this.categories = JSON.parse(JSON.stringify(config.categories));
        }
      }
    } catch (err) {
      console.error('Failed to get menu config:', err);
    }

    // Fallback if empty config
    if (this.categories.length === 0) {
      this.categories = [
        { id: 'apps', kind: 'app', label: '앱', icon: 'apps', items: [] },
        { id: 'terminal', kind: 'terminal', label: '터미널', icon: 'terminal', items: [] },
        { id: 'folders', kind: 'folder', label: '폴더·주소', icon: 'folder', items: [] },
        { id: 'system', kind: 'system', label: '시스템', icon: 'power', items: [] }
      ];
    }

    this.ensureMinFourSlots();

    this.selectedIndex = 0;
    this.isAddingMode = false;
    this.confirmDeleteId = null;
    this.renderAll();
  }

  public init() {
    this.setupEvents();
    this.loadConfig();

    listenEvent('dialog:init', () => {
      this.loadConfig();
    });
  }
}

// Global active controller instance
let globalController: CategorySettingsController | null = null;

export function getGlobalController(): CategorySettingsController {
  if (!globalController) {
    globalController = new CategorySettingsController(document);
  }
  return globalController;
}

// Auto-run when in browser/WebView2 environment
if (typeof window !== 'undefined') {
  const runInit = () => {
    if (document.getElementById('category-list')) {
      const controller = getGlobalController();
      controller.init();
    }
  };

  if (document.readyState === 'loading') {
    window.addEventListener('DOMContentLoaded', runInit);
  } else {
    runInit();
  }
}




