import { describe, it, expect, beforeEach } from 'vitest';
import { MenuController, DEFAULT_CONFIG } from './main';
import { listenEvent } from '../shared/ipc';
import type { CommitTarget, MenuConfig } from '../shared/types';

describe('MenuController & Interaction logic', () => {
  let container: HTMLElement;
  let controller: MenuController;

  beforeEach(() => {
    document.body.innerHTML = '<div class="rm" id="menu"></div>';
    container = document.getElementById('menu')!;
    controller = new MenuController(container, DEFAULT_CONFIG);
  });

  it('initializes DOM structure properly', () => {
    expect(container.querySelector('.plate')).not.toBeNull();
    expect(container.querySelector('circle.aqua')).not.toBeNull();
    expect(container.querySelector('.sel')).not.toBeNull();
    expect(container.querySelector('.hub')).not.toBeNull();
    // 4 sectors
    expect(container.querySelectorAll('.sec').length).toBe(4);
    // category 0 is initially selected, so subLayer has slots
    expect(container.querySelector('.band')).not.toBeNull();
    expect(container.querySelectorAll('.sub').length).toBeGreaterThan(0);
  });

  it('hub zone (r < 38) resolves to no commit target', () => {
    // Center is (340, 340)
    controller.handleCursorMove(340, 340);
    const target = controller.resolveTarget();
    expect(target.type).toBe('none');
  });

  it('ring zone (38 <= r <= 186) selects category based on direction', () => {
    // 12 o'clock (0 deg) -> category 0 (앱)
    controller.handleCursorMove(340, 240); // r = 100, angle = 0
    expect(controller.state.hotCategory).toBe(0);

    // 3 o'clock (90 deg) -> category 1 (터미널)
    controller.handleCursorMove(440, 340); // r = 100, angle = 90
    expect(controller.state.hotCategory).toBe(1);

    // 6 o'clock (180 deg) -> category 2 (폴더·주소)
    controller.handleCursorMove(340, 440); // r = 100, angle = 180
    expect(controller.state.hotCategory).toBe(2);

    // 9 o'clock (270 deg) -> category 3 (시스템)
    controller.handleCursorMove(240, 340); // r = 100, angle = 270
    expect(controller.state.hotCategory).toBe(3);
  });

  it('band zone (r > 186) preserves category and selects sub-slot', () => {
    // First point to category 0 (12 o'clock)
    controller.handleCursorMove(340, 240);
    expect(controller.state.hotCategory).toBe(0);

    // Now move outwards to r = 237 (slot label radius) near 12 o'clock (angle = 0)
    // Category 0 has 4 items + 1 add = 5 slots.
    // Slots are positioned around axis = 0.
    controller.handleCursorMove(340, 340 - 237); // x = 340, y = 103 -> angle = 0
    expect(controller.state.hotCategory).toBe(0); // category unchanged!
    expect(controller.state.hotSlot).toBeGreaterThanOrEqual(0);

    const target = controller.resolveTarget();
    expect(['item', 'add_item']).toContain(target.type);
  });

  it('preserves category even when moving far outside (r > 278)', () => {
    // Select category 1 (3 o'clock)
    controller.handleCursorMove(440, 340);
    expect(controller.state.hotCategory).toBe(1);

    // Move very far outside in the opposite direction (e.g. 9 o'clock)
    // As per interaction.md: in outer area, category MUST NOT change!
    controller.handleCursorMove(10, 340);
    expect(controller.state.hotCategory).toBe(1);
  });

  it('resolves correct item on commit', async () => {
    controller.selectCategory(0);
    controller.setHotSlot(0);

    let committedResult: CommitTarget | null = null;
    await listenEvent<{ target: CommitTarget }>('menu:commit_result', (payload) => {
      committedResult = payload.target;
    });

    await controller.commit();
    expect(committedResult).not.toBeNull();
    const res = committedResult!;
    expect(res.type).toBe('item');
    if (res.type === 'item') {
      expect(res.item.label).toBe('ChatGPT');
    }
  });

  it('resolves add_item when clicking the add slot', async () => {
    // Category 1 has 3 items, so slot index 3 is 'add'
    controller.selectCategory(1);
    controller.setHotSlot(3);

    const target = controller.resolveTarget();
    expect(target.type).toBe('add_item');
    if (target.type === 'add_item') {
      expect(target.category.id).toBe('terminal');
    }
  });

  it('handles category addition slot when < 4 categories', () => {
    const twoCatsConfig: MenuConfig = {
      version: 1,
      accent: '#1E9BFF',
      categories: DEFAULT_CONFIG.categories.slice(0, 2),
    };
    controller.state.config = twoCatsConfig;
    controller.render();

    // addSector is at axis = 2 * 90 = 180 (6 o'clock)
    // Move to 6 o'clock in ring: (340, 440)
    controller.handleCursorMove(340, 440);
    expect(controller.state.hotCategory).toBe(-1);
    const addBtn = container.querySelector('button.add');
    expect(addBtn).not.toBeNull();
  });

  it('closes menu on Escape key', () => {
    expect(controller.state.isOpen).toBe(true);
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }));
    expect(controller.state.isOpen).toBe(false);
  });

  it('renders custom image icon and preset icon prioritizing slot.item.icon over mono', () => {
    const configWithIcons: MenuConfig = {
      version: 1,
      accent: '#1E9BFF',
      categories: [
        {
          id: 'custom_apps',
          kind: 'app',
          label: '앱',
          items: [
            {
              label: 'Image App',
              path: 'app.exe',
              mono: 'IA',
              icon: 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==',
            },
            {
              label: 'Preset App',
              path: 'preset.exe',
              mono: 'PA',
              icon: 'terminal',
            },
          ],
        },
      ],
    };

    const ctrl = new MenuController(container, configWithIcons);
    ctrl.selectCategory(0);

    const subSlots = container.querySelectorAll('.sub');
    expect(subSlots.length).toBeGreaterThan(0);

    // Slot 0 has data:image icon -> must render <image> element, not .mono text
    const slot0 = subSlots[0];
    const imageEl = slot0.querySelector('image');
    expect(imageEl).not.toBeNull();
    expect(imageEl?.getAttribute('href')).toContain('data:image/png');
    expect(slot0.querySelector('.mono')).toBeNull();

    // Slot 1 has preset 'terminal' icon -> must render <use href="#i-terminal">
    const slot1 = subSlots[1];
    const useEl = slot1.querySelector('use');
    expect(useEl).not.toBeNull();
    expect(useEl?.getAttribute('href')).toBe('#i-terminal');
    expect(slot1.querySelector('.mono')).toBeNull();
  });

  it('supports empty slots (1, 2, 4 layout) and commitAddCategory on click', async () => {
    const configWithEmptySlot: MenuConfig = {
      version: 1,
      accent: '#1E9BFF',
      categories: [
        {
          id: 'apps',
          kind: 'app',
          label: '앱',
          items: [{ label: '메모장', path: 'notepad.exe' }],
        },
        {
          id: 'terminal',
          kind: 'terminal',
          label: '터미널',
          items: [{ label: '새 창', action: 'open' }],
        },
        {
          id: 'slot-3',
          kind: 'empty',
          label: '빈칸',
          items: [],
        },
        {
          id: 'system',
          kind: 'system',
          label: '시스템',
          items: [{ label: '잠금', fn: 'lock' }],
        },
      ],
    };

    const ctrl = new MenuController(container, configWithEmptySlot);

    // Initial state: hotCategory is 0 (first non-empty category)
    expect(ctrl.state.hotCategory).toBe(0);

    // Empty slot (slot 2) is rendered as button with class 'add' and 'empty-slot'
    const emptyBtn = container.querySelector('button.add.empty-slot') as HTMLButtonElement;
    expect(emptyBtn).not.toBeNull();

    // Hover over slot 2 (180 deg / 6 o'clock): (340, 440)
    ctrl.handleCursorMove(340, 440);
    // hotCategory is reset to -1, hotSlot to -1
    expect(ctrl.state.hotCategory).toBe(-1);
    expect(ctrl.state.hotSlot).toBe(-1);

    // resolveTarget() should return none
    expect(ctrl.resolveTarget().type).toBe('none');

    // Clicking the empty button calls commitAddCategory
    let committedResult: CommitTarget | null = null;
    await listenEvent<{ target: CommitTarget }>('menu:commit_result', (payload) => {
      committedResult = payload.target;
    });

    emptyBtn.click();
    expect(committedResult).not.toBeNull();
    expect(committedResult!.type).toBe('add_category');
  });
});
