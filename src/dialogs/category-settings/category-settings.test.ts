import { describe, it, expect, beforeEach, vi } from 'vitest';
import fs from 'fs';
import path from 'path';
import {
  CategorySettingsController,
  getOnAccentColor,
  PRESET_ICONS,
  MAX_CATEGORIES,
  calcXY,
  calcP,
  fmt
} from './main';

// Mock ipc
const mockInvoke = vi.fn();
vi.mock('../../shared/ipc', () => ({
  invokeCommand: (cmd: string, args: any) => mockInvoke(cmd, args),
  listenEvent: vi.fn(),
  emitEvent: vi.fn()
}));

if (typeof window.PointerEvent === 'undefined') {
  class PointerEvent extends MouseEvent {
    pointerId: number;
    constructor(type: string, params: any = {}) {
      super(type, params);
      this.pointerId = params.pointerId || 0;
    }
  }
  (window as any).PointerEvent = PointerEvent;
  (global as any).PointerEvent = PointerEvent;
}
describe('category-settings unit and DOM integration tests', () => {
  let controller: CategorySettingsController;

  beforeEach(async () => {
    vi.clearAllMocks();

    // Setup mock DOM from index.html
    const htmlPath = path.resolve(__dirname, 'index.html');
    const htmlContent = fs.readFileSync(htmlPath, 'utf8');
    document.documentElement.innerHTML = htmlContent;

    // Default mock response for get_menu_config
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'get_menu_config') {
        return {
          version: 1,
          accent: '#1E9BFF',
          categories: [
            { id: 'apps', kind: 'app', label: '앱', icon: 'apps', items: [{ label: 'App 1' }] },
            { id: 'terminal', kind: 'terminal', label: '터미널', icon: 'terminal', items: [{ label: 'Term 1' }] },
            { id: 'folders', kind: 'folder', label: '폴더·주소', icon: 'folder', items: [] },
            { id: 'system', kind: 'system', label: '시스템', icon: 'power', items: [{ label: 'Sys 1' }] }
          ]
        };
      }
      return {};
    });

    controller = new CategorySettingsController(document);
    controller.setupEvents();
    await controller.loadConfig();
  });

  describe('getOnAccentColor', () => {
    it('returns #FFFFFF for dark purple (#6E56CF) with luminance < 0.2', () => {
      expect(getOnAccentColor('#6E56CF')).toBe('#FFFFFF');
    });

    it('returns #04121C for bright blue (#1E9BFF) with luminance >= 0.2', () => {
      expect(getOnAccentColor('#1E9BFF')).toBe('#04121C');
    });

    it('returns #04121C for standard blue (#3D7BFF) with luminance >= 0.2', () => {
      expect(getOnAccentColor('#3D7BFF')).toBe('#04121C');
    });

    it('returns #04121C for pink (#D6409F) with luminance >= 0.2', () => {
      expect(getOnAccentColor('#D6409F')).toBe('#04121C');
    });

    it('handles invalid hex gracefully', () => {
      expect(getOnAccentColor('invalid')).toBe('#04121C');
      expect(getOnAccentColor('')).toBe('#04121C');
    });
  });

  describe('Geometry calculations', () => {
    it('fmt formats number to 1 decimal place', () => {
      expect(fmt(12.34)).toBe('12.3');
      expect(fmt(12.36)).toBe('12.4');
      expect(fmt(10)).toBe('10');
    });

    it('calcXY calculates 12 oclock direction (0 deg) correctly', () => {
      const [x, y] = calcXY(58, 0);
      expect(Math.round(x)).toBe(100);
      expect(Math.round(y)).toBe(42);
    });

    it('calcXY calculates 3 oclock direction (90 deg) correctly', () => {
      const [x, y] = calcXY(58, 90);
      expect(Math.round(x)).toBe(158);
      expect(Math.round(y)).toBe(100);
    });

    it('calcP returns formatted string', () => {
      expect(calcP(58, 0)).toBe('100 42');
    });
  });

  describe('PRESET_ICONS & MAX_CATEGORIES', () => {
    it('contains the 8 required preset icons', () => {
      expect(PRESET_ICONS).toEqual([
        'apps',
        'terminal',
        'folder',
        'power',
        'globe',
        'window',
        'paste',
        'play'
      ]);
      expect(PRESET_ICONS.length).toBe(8);
    });

    it('has MAX_CATEGORIES set to 8', () => {
      expect(MAX_CATEGORIES).toBe(8);
    });
  });

  describe('Initial rendering and state', () => {
    it('renders initial categories list correctly', () => {
      const list = document.getElementById('category-list');
      expect(list).not.toBeNull();
      const rows = list?.querySelectorAll('.row');
      expect(rows?.length).toBe(4);

      // First row should be selected
      expect(rows?.[0].classList.contains('on')).toBe(true);

      const title = document.getElementById('list-title');
      expect(title?.textContent).toBe('분류 4개');

      const caption = document.getElementById('pv-caption');
      expect(caption?.textContent).toBe('분류 4개 · 4칸');
    });

    it('displays selected category details in left editor', () => {
      const inpName = document.getElementById('inp-name') as HTMLInputElement;
      expect(inpName.value).toBe('앱');

      const kindNote = document.getElementById('kind-note');
      expect(kindNote?.textContent).toBe('종류는 앱입니다. 만든 뒤에는 바꿀 수 없습니다.');
    });
  });

  describe('Category Selection', () => {
    it('updates selection and left editor when clicking another row', () => {
      const list = document.getElementById('category-list');
      const rows = list?.querySelectorAll('.row');
      const secondRowMain = rows?.[1].querySelector('.rowmain') as HTMLButtonElement;

      secondRowMain.click();

      expect(controller.selectedIndex).toBe(1);

      const inpName = document.getElementById('inp-name') as HTMLInputElement;
      expect(inpName.value).toBe('터미널');

      const kindNote = document.getElementById('kind-note');
      expect(kindNote?.textContent).toBe('종류는 터미널입니다. 만든 뒤에는 바꿀 수 없습니다.');
    });
  });

  describe('Editing category name and icon', () => {
    it('updates category label on input typing', () => {
      const inpName = document.getElementById('inp-name') as HTMLInputElement;
      inpName.value = '새이름';
      inpName.dispatchEvent(new Event('input'));

      expect(controller.categories[0].label).toBe('새이름');
      const rowName = document.querySelector('[data-index="0"] .cname');
      expect(rowName?.textContent).toBe('새이름');
    });

    it('truncates name to 10 characters', () => {
      const inpName = document.getElementById('inp-name') as HTMLInputElement;
      inpName.value = '가나다라마바사아자차카타파하';
      inpName.dispatchEvent(new Event('input'));

      expect(controller.categories[0].label).toBe('가나다라마바사아자차');
      expect(controller.categories[0].label.length).toBe(10);
    });

    it('updates icon when clicking preset icon button', () => {
      const btnFolder = document.querySelector('.ico[data-icon="folder"]') as HTMLButtonElement;
      btnFolder.click();

      expect(controller.categories[0].icon).toBe('folder');
      expect(btnFolder.classList.contains('on')).toBe(true);
    });
  });

  describe('Reordering categories', () => {
    it('disables up button on first item and down button on last item', () => {
      const btnUp0 = document.getElementById('btn-up-0') as HTMLButtonElement;
      expect(btnUp0.disabled).toBe(true);

      const btnDown3 = document.getElementById('btn-down-3') as HTMLButtonElement;
      expect(btnDown3.disabled).toBe(true);
    });

    it('swaps items when clicking down button', () => {
      const btnDown0 = document.getElementById('btn-down-0') as HTMLButtonElement;
      btnDown0.click();

      expect(controller.categories[0].label).toBe('터미널');
      expect(controller.categories[1].label).toBe('앱');
      expect(controller.selectedIndex).toBe(1);
    });

    it('swaps items when clicking up button', () => {
      const btnUp1 = document.getElementById('btn-up-1') as HTMLButtonElement;
      btnUp1.click();

      expect(controller.categories[0].label).toBe('터미널');
      expect(controller.categories[1].label).toBe('앱');
      expect(controller.selectedIndex).toBe(0);
    });
  });

  describe('Deleting categories', () => {
    it('prompts confirmation when deleting category with items', () => {
      const btnDel0 = document.getElementById('btn-del-0') as HTMLButtonElement;
      btnDel0.click();

      // Row should change to warn confirmation row
      const list = document.getElementById('category-list');
      const warnRow = list?.querySelector('.row.warn');
      expect(warnRow).not.toBeNull();
      expect(warnRow?.textContent).toContain('"앱" 분류와 그 안의 항목 1개를 함께 지울까요?');

      // Cancel button should revert confirmation
      const btnCancel = warnRow?.querySelector('button:not(.danger)') as HTMLButtonElement;
      btnCancel.click();

      expect(list?.querySelector('.row.warn')).toBeNull();
      expect(controller.categories.length).toBe(4);
    });

    it('converts to empty slot when category has no items in 4-slot mode', () => {
      // Third category ('폴더·주소') has 0 items
      const btnDel2 = document.getElementById('btn-del-2') as HTMLButtonElement;
      btnDel2.click();

      expect(controller.categories.length).toBe(4);
      expect(controller.categories[2].kind).toBe('empty');
      expect(controller.categories[2].label).toBe('빈칸');
      expect(controller.categories.map(c => c.label)).not.toContain('폴더·주소');
    });

    it('confirms and converts to empty slot when non-empty category is deleted in 4-slot mode', () => {
      const btnDel0 = document.getElementById('btn-del-0') as HTMLButtonElement;
      btnDel0.click();

      const warnRow = document.querySelector('.row.warn');
      const btnConfirm = warnRow?.querySelector('button.danger') as HTMLButtonElement;
      btnConfirm.click();

      expect(controller.categories.length).toBe(4);
      expect(controller.categories[0].kind).toBe('empty');
      expect(controller.categories[0].label).toBe('빈칸');
      expect(controller.categories[1].label).toBe('터미널');
    });

    it('shrinks slot count when deleting category in 5+ slots mode', () => {
      // Add a 5th category
      controller.categories.push({
        id: 'cat-5',
        kind: 'app',
        label: '앱 2',
        icon: 'apps',
        items: []
      });
      controller.renderAll();
      expect(controller.categories.length).toBe(5);

      const btnDel4 = document.getElementById('btn-del-4') as HTMLButtonElement;
      btnDel4.click();

      expect(controller.categories.length).toBe(4);
      expect(controller.categories.map(c => c.label)).not.toContain('앱 2');
    });

    it('disables delete button when only 1 valid category remains', () => {
      controller.categories = [
        { id: 'app1', kind: 'app', label: '앱', icon: 'apps', items: [] },
        { id: 'empty-1', kind: 'empty', label: '빈칸', icon: 'add', items: [] },
        { id: 'empty-2', kind: 'empty', label: '빈칸', icon: 'add', items: [] },
        { id: 'empty-3', kind: 'empty', label: '빈칸', icon: 'add', items: [] }
      ];
      controller.renderAll();

      const btnDel0 = document.getElementById('btn-del-0') as HTMLButtonElement;
      expect(btnDel0.disabled).toBe(true);

      const btnDel1 = document.getElementById('btn-del-1') as HTMLButtonElement;
      expect(btnDel1.disabled).toBe(true);
    });
  });

  describe('Minimum 4 slots and 1, 2, 4 layout', () => {
    it('ensures minimum 4 slots when loading config with fewer categories', async () => {
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === 'get_menu_config') {
          return {
            version: 1,
            accent: '#1E9BFF',
            categories: [
              { id: 'apps', kind: 'app', label: '앱', icon: 'apps', items: [] },
              { id: 'terminal', kind: 'terminal', label: '터미널', icon: 'terminal', items: [] }
            ]
          };
        }
        return {};
      });

      const ctrl = new CategorySettingsController(document);
      ctrl.setupEvents();
      await ctrl.loadConfig();

      expect(ctrl.categories.length).toBe(4);
      expect(ctrl.categories[0].kind).toBe('app');
      expect(ctrl.categories[1].kind).toBe('terminal');
      expect(ctrl.categories[2].kind).toBe('empty');
      expect(ctrl.categories[3].kind).toBe('empty');
    });

    it('renders 1, 2, 4 layout correctly with slot 3 being empty', () => {
      controller.categories = [
        { id: 'apps', kind: 'app', label: '앱', icon: 'apps', items: [] },
        { id: 'terminal', kind: 'terminal', label: '터미널', icon: 'terminal', items: [] },
        { id: 'empty-2', kind: 'empty', label: '빈칸', icon: 'add', items: [] },
        { id: 'folders', kind: 'folder', label: '폴더·주소', icon: 'folder', items: [] }
      ];
      controller.renderAll();

      // Check rows in category list
      const rows = document.querySelectorAll('.row');
      expect(rows.length).toBe(4);
      expect(rows[2].classList.contains('empty')).toBe(true);
      expect(rows[2].querySelector('.cname')?.textContent).toBe('(빈칸)');
      expect(rows[2].querySelector('.kind')?.textContent).toBe('빈칸');
      expect(rows[2].querySelector('.cnt')?.textContent).toBe('항목 없음');

      // Check preview caption
      const caption = document.getElementById('pv-caption');
      expect(caption?.textContent).toBe('분류 3개 · 4칸 (빈칸 1)');

      // Check title
      const title = document.getElementById('list-title');
      expect(title?.textContent).toBe('분류 3개');

      // Check preview icons
      const iconsLayer = document.getElementById('pv-icons-layer');
      const icons = iconsLayer?.querySelectorAll('.pvi');
      expect(icons?.length).toBe(4);
      expect(icons?.[2].classList.contains('blank')).toBe(true);
      expect(icons?.[2].innerHTML).toContain('href="#a-add"');
    });

    it('disables name input and clears highlight when empty slot is selected', () => {
      controller.categories = [
        { id: 'apps', kind: 'app', label: '앱', icon: 'apps', items: [] },
        { id: 'terminal', kind: 'terminal', label: '터미널', icon: 'terminal', items: [] },
        { id: 'empty-2', kind: 'empty', label: '빈칸', icon: 'add', items: [] },
        { id: 'folders', kind: 'folder', label: '폴더·주소', icon: 'folder', items: [] }
      ];
      controller.selectedIndex = 2; // select empty slot
      controller.renderAll();

      const inpName = document.getElementById('inp-name') as HTMLInputElement;
      expect(inpName.disabled).toBe(true);
      expect(inpName.value).toBe('');

      const pvSel = document.querySelector('#pv-sel') as SVGPathElement;
      expect(pvSel.style.display).toBe('none');

      const pvArc = document.querySelector('#pv-arc') as SVGPathElement;
      expect(pvArc.style.display).toBe('none');
    });

    it('replaces the first empty slot when adding category', () => {
      controller.categories = [
        { id: 'apps', kind: 'app', label: '앱', icon: 'apps', items: [] },
        { id: 'terminal', kind: 'terminal', label: '터미널', icon: 'terminal', items: [] },
        { id: 'empty-2', kind: 'empty', label: '빈칸', icon: 'add', items: [] },
        { id: 'folders', kind: 'folder', label: '폴더·주소', icon: 'folder', items: [] }
      ];
      controller.renderAll();

      const btnOpenAdd = document.getElementById('btn-open-add') as HTMLButtonElement;
      btnOpenAdd.click();

      // Click '시스템' tile
      const sysTile = document.querySelector('.tile[data-kind="system"]') as HTMLButtonElement;
      sysTile.click();

      expect(controller.categories.length).toBe(4);
      expect(controller.categories[2].kind).toBe('system');
      expect(controller.categories[2].label).toBe('시스템');
      expect(controller.selectedIndex).toBe(2);

      const caption = document.getElementById('pv-caption');
      expect(caption?.textContent).toBe('분류 4개 · 4칸');
    });
  });

  describe('Drag and drop reordering', () => {
    it('sets up draggable rows with grip handles and div.rowmain', () => {
      const rows = document.querySelectorAll('.row');
      expect(rows.length).toBe(4);
      rows.forEach((row) => {
        expect(row.getAttribute('draggable')).toBeNull();
        const rowmain = row.querySelector('.rowmain');
        expect(rowmain?.tagName.toLowerCase()).toBe('div');
        expect(rowmain?.getAttribute('role')).toBe('button');
        const grip = row.querySelector('.grip');
        expect(grip).not.toBeNull();
      });
    });

    it('reorders categories on Pointer drag and drop', () => {
      const rows = document.querySelectorAll<HTMLDivElement>('.row');
      const row0 = rows[0];
      const row2 = rows[2];
      const target0 = row0.querySelector('.rowmain') || row0;
      
      target0.setPointerCapture = vi.fn();
      target0.releasePointerCapture = vi.fn();
      document.elementsFromPoint = vi.fn().mockReturnValue([row2]);

      const pd = new PointerEvent('pointerdown', { button: 0, clientX: 0, clientY: 0, pointerId: 1 });
      target0.dispatchEvent(pd);
      expect(controller.draggedIndex).toBe(0);

      const pm = new PointerEvent('pointermove', { clientX: 10, clientY: 10, pointerId: 1 });
      target0.dispatchEvent(pm);
      expect(row0.classList.contains('dragging')).toBe(true);

      const pu = new PointerEvent('pointerup', { clientX: 10, clientY: 10, pointerId: 1 });
      target0.dispatchEvent(pu);

      expect(controller.categories[0].label).toBe('터미널');
      expect(controller.categories[2].label).toBe('앱');
      expect(controller.selectedIndex).toBe(2);
      expect(controller.draggedIndex).toBeNull();
      
      delete (document as any).elementsFromPoint;
    });

    it('can reorder an empty slot by drag and drop (SWAP with non-empty)', () => {
      controller.categories = [
        { id: 'apps', kind: 'app', label: '앱', icon: 'apps', items: [] },
        { id: 'terminal', kind: 'terminal', label: '터미널', icon: 'terminal', items: [] },
        { id: 'empty-2', kind: 'empty', label: '빈칸', icon: 'add', items: [] },
        { id: 'folders', kind: 'folder', label: '폴더', icon: 'folder', items: [] }
      ];
      controller.renderAll();

      const rows = document.querySelectorAll<HTMLDivElement>('.row');
      const emptyRow = rows[2];
      const targetRow = rows[0];
      const targetEmpty = emptyRow.querySelector('.rowmain') || emptyRow;

      targetEmpty.setPointerCapture = vi.fn();
      targetEmpty.releasePointerCapture = vi.fn();
      document.elementsFromPoint = vi.fn().mockReturnValue([targetRow]);

      targetEmpty.dispatchEvent(new PointerEvent('pointerdown', { button: 0, clientX: 0, clientY: 0, pointerId: 1 }));
      targetEmpty.dispatchEvent(new PointerEvent('pointermove', { clientX: 10, clientY: 10, pointerId: 1 }));
      targetEmpty.dispatchEvent(new PointerEvent('pointerup', { clientX: 10, clientY: 10, pointerId: 1 }));

      expect(controller.categories[0].kind).toBe('empty');
      expect(controller.categories[2].label).toBe('앱');
      
      delete (document as any).elementsFromPoint;
    });

    it('swaps item 3 with empty slot 4 when dragged, resulting in 1, 2, [empty], 4 layout', () => {
      controller.categories = [
        { id: 'apps', kind: 'app', label: '앱', icon: 'apps', items: [] },
        { id: 'terminal', kind: 'terminal', label: '터미널', icon: 'terminal', items: [] },
        { id: 'folders', kind: 'folder', label: '폴더', icon: 'folder', items: [] },
        { id: 'empty-3', kind: 'empty', label: '빈칸', icon: 'add', items: [] }
      ];
      controller.renderAll();

      const rows = document.querySelectorAll<HTMLDivElement>('.row');
      const row2 = rows[2];
      const row3 = rows[3];
      const target2 = row2.querySelector('.rowmain') || row2;

      target2.setPointerCapture = vi.fn();
      target2.releasePointerCapture = vi.fn();
      document.elementsFromPoint = vi.fn().mockReturnValue([row3]);

      target2.dispatchEvent(new PointerEvent('pointerdown', { button: 0, clientX: 0, clientY: 0, pointerId: 1 }));
      target2.dispatchEvent(new PointerEvent('pointermove', { clientX: 10, clientY: 10, pointerId: 1 }));
      target2.dispatchEvent(new PointerEvent('pointerup', { clientX: 10, clientY: 10, pointerId: 1 }));

      expect(controller.categories[2].kind).toBe('empty');
      expect(controller.categories[3].label).toBe('폴더');
      
      delete (document as any).elementsFromPoint;
    });    it('directly invokes executeMove to swap an item into an empty slot', () => {
      // 0: 앱, 1: 터미널, 2: 폴더·주소, 3: empty
      controller.categories = [
        { id: 'apps', kind: 'app', label: '앱', icon: 'apps', items: [] },
        { id: 'terminal', kind: 'terminal', label: '터미널', icon: 'terminal', items: [] },
        { id: 'folders', kind: 'folder', label: '폴더·주소', icon: 'folder', items: [] },
        { id: 'empty-3', kind: 'empty', label: '빈칸', icon: 'add', items: [] }
      ];
      controller.renderAll();

      controller.executeMove(2, 3);

      expect(controller.categories[2].kind).toBe('empty');
      expect(controller.categories[3].label).toBe('폴더·주소');
      expect(controller.selectedIndex).toBe(3);
      expect(mockInvoke).toHaveBeenCalledWith('save_categories', expect.any(Object));
    });
  });

  describe('Adding categories', () => {
    it('opens kind picker on "+ 분류 추가" click', () => {
      const btnOpenAdd = document.getElementById('btn-open-add') as HTMLButtonElement;
      const addPicker = document.getElementById('add-picker-box') as HTMLDivElement;

      expect(addPicker.style.display).toBe('none');

      btnOpenAdd.click();
      expect(addPicker.style.display).toBe('block');
    });

    it('adds new category with auto-incremented name if kind already exists', () => {
      const btnOpenAdd = document.getElementById('btn-open-add') as HTMLButtonElement;
      btnOpenAdd.click();

      // Click '앱' tile
      const appTile = document.querySelector('.tile[data-kind="app"]') as HTMLButtonElement;
      appTile.click();

      expect(controller.categories.length).toBe(5);
      // Since '앱' already exists, new category should be '앱 2'
      expect(controller.categories[4].label).toBe('앱 2');
      expect(controller.categories[4].kind).toBe('app');
      expect(controller.categories[4].icon).toBe('apps');
      expect(controller.categories[4].items).toEqual([]);

      // Newly added category should be selected
      expect(controller.selectedIndex).toBe(4);
    });

    it('disables "+ 분류 추가" when reaching 8 categories', async () => {
      const btnOpenAdd = document.getElementById('btn-open-add') as HTMLButtonElement;
      for (let i = 0; i < 4; i++) {
        btnOpenAdd.click();
        const tile = document.querySelector('.tile[data-kind="app"]') as HTMLButtonElement;
        tile.click();
      }

      expect(controller.categories.length).toBe(8);

      expect(btnOpenAdd.disabled).toBe(true);
      const note = document.getElementById('add-max-note');
      expect(note?.textContent).toBe('분류는 8개까지 만들 수 있습니다.');
    });
  });

  describe('Keyboard ESC handling', () => {
    it('closes adding mode on ESC', () => {
      const btnOpenAdd = document.getElementById('btn-open-add') as HTMLButtonElement;
      btnOpenAdd.click();
      expect(controller.isAddingMode).toBe(true);

      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }));
      expect(controller.isAddingMode).toBe(false);
    });

    it('cancels delete confirmation on ESC', () => {
      const btnDel0 = document.getElementById('btn-del-0') as HTMLButtonElement;
      btnDel0.click();
      expect(controller.confirmDeleteId).not.toBeNull();

      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }));
      expect(controller.confirmDeleteId).toBeNull();
    });

    it('closes dialog on ESC when normal state', () => {
      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }));
      expect(mockInvoke).toHaveBeenCalledWith('close_dialog_window', {
        dialogLabel: 'category-settings'
      });
    });
  });

  describe('Auto-saving configuration', () => {
    it('debounces auto-save on name input typing and shows save status', async () => {
      vi.useFakeTimers();
      const inpName = document.getElementById('inp-name') as HTMLInputElement;
      inpName.value = '변경된이름';
      inpName.dispatchEvent(new Event('input'));

      // Before timer expires, save_categories should not be called yet
      expect(mockInvoke).not.toHaveBeenCalledWith('save_categories', expect.any(Object));

      // Fast-forward debounce timer (250ms)
      vi.advanceTimersByTime(250);
      await Promise.resolve();

      expect(mockInvoke).toHaveBeenCalledWith('save_categories', {
        categories: expect.arrayContaining([
          expect.objectContaining({ label: '변경된이름' })
        ])
      });

      // Save status indicator opacity
      const saveStatus = document.getElementById('save-status');
      expect(saveStatus?.style.opacity).toBe('1');

      // Fast-forward 1000ms for status fade out
      vi.advanceTimersByTime(1000);
      expect(saveStatus?.style.opacity).toBe('0');

      vi.useRealTimers();
    });

    it('flushes pending auto-save when closing dialog via close button', async () => {
      vi.useFakeTimers();
      const inpName = document.getElementById('inp-name') as HTMLInputElement;
      inpName.value = '즉시저장';
      inpName.dispatchEvent(new Event('input'));

      expect(controller.autoSaveTimer).not.toBeNull();

      // Click close button in footer (#btn-close-dialog)
      const btnCloseDialog = document.getElementById('btn-close-dialog') as HTMLButtonElement;
      btnCloseDialog.click();
      
      // Wait for async handleClose to complete microtasks
      await vi.waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('close_dialog_window', {
          dialogLabel: 'category-settings'
        });
      });

      expect(controller.autoSaveTimer).toBeNull();
      expect(mockInvoke).toHaveBeenCalledWith('save_categories', {
        categories: expect.arrayContaining([
          expect.objectContaining({ label: '즉시저장' })
        ])
      });

      vi.useRealTimers();
    });

    it('immediately auto-saves when reordering via up/down buttons', () => {
      const btnDown0 = document.getElementById('btn-down-0') as HTMLButtonElement;
      btnDown0.click();

      expect(mockInvoke).toHaveBeenCalledWith('save_categories', {
        categories: expect.any(Array)
      });
    });

    it('immediately auto-saves when deleting a category', () => {
      const btnDel2 = document.getElementById('btn-del-2') as HTMLButtonElement;
      btnDel2.click();

      expect(mockInvoke).toHaveBeenCalledWith('save_categories', {
        categories: expect.any(Array)
      });
    });

    it('immediately auto-saves when picking an icon', () => {
      const btnGlobe = document.querySelector('.ico[data-icon="globe"]') as HTMLButtonElement;
      btnGlobe.click();

      expect(mockInvoke).toHaveBeenCalledWith('save_categories', {
        categories: expect.any(Array)
      });
    });
  });
});



