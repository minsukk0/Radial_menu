import { describe, it, expect } from 'vitest';
import {
  GEOMETRY,
  fmt,
  make,
  layout,
  nextRotation,
  hitTest,
} from './radial-layout';
import type { CategoryConfig } from '../shared/types';
import fs from 'fs';
import { resolve } from 'path';

// 원본 JS 파일 가져오기 (비교 검증용)
const code = fs.readFileSync(
  resolve(__dirname, '../../UI_designs/reference/radial-layout.js'),
  'utf8'
);
const refModule = { exports: {} as any };
const loader = new Function('module', 'exports', code);
loader(refModule, refModule.exports);
const ReferenceLayout = refModule.exports;

describe('GEOMETRY constants', () => {
  it('should match the design tokens', () => {
    expect(GEOMETRY.hubRadius).toBe(30);
    expect(GEOMETRY.arcRadius).toBe(36);
    expect(GEOMETRY.ringInner).toBe(38);
    expect(GEOMETRY.ringOuter).toBe(186);
    expect(GEOMETRY.ringLabelRadius).toBe(116);
    expect(GEOMETRY.ringMinSectors).toBe(4);
    expect(GEOMETRY.selectionSpill).toBe(218);
    expect(GEOMETRY.bandInner).toBe(196);
    expect(GEOMETRY.bandOuter).toBe(278);
    expect(GEOMETRY.bandCorner).toBe(14);
    expect(GEOMETRY.slotInner).toBe(200);
    expect(GEOMETRY.slotOuter).toBe(274);
    expect(GEOMETRY.slotCorner).toBe(10);
    expect(GEOMETRY.slotAngle).toBe(24);
    expect(GEOMETRY.slotMaxSpan).toBe(216);
    expect(GEOMETRY.slotMinCount).toBe(4);
    expect(GEOMETRY.slotLabelRadius).toBe(237);
  });
});

describe('Helper functions: fmt and make', () => {
  it('fmt rounds to 1 decimal place', () => {
    expect(fmt(12.34)).toBe('12.3');
    expect(fmt(12.36)).toBe('12.4');
    expect(fmt(10)).toBe('10');
  });

  it('make(cx, cy).xy calculates correct coordinates for 4 cardinal directions', () => {
    const m = make(380, 380);
    // 0도 (12시 방향)
    const [x0, y0] = m.xy(100, 0);
    expect(Math.round(x0)).toBe(380);
    expect(Math.round(y0)).toBe(280);

    // 90도 (3시 방향)
    const [x90, y90] = m.xy(100, 90);
    expect(Math.round(x90)).toBe(480);
    expect(Math.round(y90)).toBe(380);

    // 180도 (6시 방향)
    const [x180, y180] = m.xy(100, 180);
    expect(Math.round(x180)).toBe(380);
    expect(Math.round(y180)).toBe(480);

    // 270도 (9시 방향)
    const [x270, y270] = m.xy(100, 270);
    expect(Math.round(x270)).toBe(280);
    expect(Math.round(y270)).toBe(380);
  });

  it('wedge generates valid SVG path matching reference', () => {
    const mTs = make(380, 380);
    ReferenceLayout.layout([], -1); // uses make internally
    const pathTs = mTs.wedge(38, 186, -45, 45);
    expect(pathTs).toContain('M');
    expect(pathTs).toContain('L');
    expect(pathTs).toContain('A186 186');
    expect(pathTs).toContain('A38 38');
    expect(pathTs.endsWith('Z')).toBe(true);
  });
});

describe('nextRotation', () => {
  it('returns targetAxis when previous is null or undefined', () => {
    expect(nextRotation(undefined, 90)).toBe(90);
    expect(nextRotation(null, 180)).toBe(180);
  });

  it('calculates the shortest angular path', () => {
    // 0 -> 90: +90
    expect(nextRotation(0, 90)).toBe(90);
    // 0 -> 270: -90
    expect(nextRotation(0, 270)).toBe(-90);
    // 350 -> 10: +20 (370)
    expect(nextRotation(350, 10)).toBe(370);
    // 10 -> 350: -20 (-10)
    expect(nextRotation(10, 350)).toBe(-10);
    // 180 -> 0: -180
    expect(nextRotation(180, 0)).toBe(0);
  });

  it('matches ReferenceLayout.nextRotation exactly', () => {
    const angles = [0, 45, 90, 135, 180, 225, 270, 315, 360, 450, -90];
    for (const prev of angles) {
      for (const target of angles) {
        expect(nextRotation(prev, target)).toBe(
          ReferenceLayout.nextRotation(prev, target)
        );
      }
    }
  });
});

describe('hitTest', () => {
  it('correctly classifies zones (default cx=380, cy=380)', () => {
    // 중심 (r < 38)
    expect(hitTest(380, 380, 4).zone).toBe('hub');
    expect(hitTest(380, 350, 4).zone).toBe('hub'); // r = 30 < 38

    // 상위 링 (38 <= r <= 186)
    expect(hitTest(380, 340, 4).zone).toBe('ring'); // r = 40
    expect(hitTest(380, 200, 4).zone).toBe('ring'); // r = 180
    expect(hitTest(380, 194, 4).zone).toBe('ring'); // r = 186

    // 하위 띠 (186 < r <= 278)
    expect(hitTest(380, 180, 4).zone).toBe('band'); // r = 200
    expect(hitTest(380, 105, 4).zone).toBe('band'); // r = 275

    // 바깥 (r > 278)
    expect(hitTest(380, 50, 4).zone).toBe('outside'); // r = 330
  });

  it('correctly calculates sector based on angle (4 sectors)', () => {
    // 12시 방향 (0도) -> sector 0 (-45 ~ +45)
    expect(hitTest(380, 280, 4).sector).toBe(0);
    expect(hitTest(380, 280, 4).angle).toBeCloseTo(0, 1);

    // 3시 방향 (90도) -> sector 1 (45 ~ 135)
    expect(hitTest(480, 380, 4).sector).toBe(1);
    expect(hitTest(480, 380, 4).angle).toBeCloseTo(90, 1);

    // 6시 방향 (180도) -> sector 2 (135 ~ 225)
    expect(hitTest(380, 480, 4).sector).toBe(2);
    expect(hitTest(380, 480, 4).angle).toBeCloseTo(180, 1);

    // 9시 방향 (270도) -> sector 3 (225 ~ 315)
    expect(hitTest(280, 380, 4).sector).toBe(3);
    expect(hitTest(280, 380, 4).angle).toBeCloseTo(270, 1);
  });

  it('works with custom center (cx=340, cy=340 for 680x680 window)', () => {
    const opts = { cx: 340, cy: 340 };
    expect(hitTest(340, 340, 4, opts).zone).toBe('hub');
    expect(hitTest(340, 340 - 100, 4, opts).zone).toBe('ring');
    expect(hitTest(340, 340 - 100, 4, opts).sector).toBe(0);
    expect(hitTest(340 + 100, 340, 4, opts).sector).toBe(1);
    expect(hitTest(340, 340 - 220, 4, opts).zone).toBe('band');
    expect(hitTest(340, 340 - 300, 4, opts).zone).toBe('outside');
  });

  it('matches ReferenceLayout.hitTest exactly', () => {
    const points = [
      [380, 380],
      [380, 350],
      [380, 300],
      [450, 450],
      [200, 200],
      [100, 500],
    ];
    for (const [x, y] of points) {
      const tsRes = hitTest(x, y, 4);
      const jsRes = ReferenceLayout.hitTest(x, y, 4);
      expect(tsRes.zone).toBe(jsRes.zone);
      expect(tsRes.sector).toBe(jsRes.sector);
      expect(tsRes.angle).toBeCloseTo(jsRes.angle, 3);
      expect(tsRes.radius).toBeCloseTo(jsRes.radius, 3);
    }
  });
});

describe('layout calculation', () => {
  const sampleCategories: CategoryConfig[] = [
    {
      id: 'app',
      kind: 'app',
      label: '앱',
      icon: 'apps',
      items: [
        { label: '메모장', path: 'notepad.exe' },
        { label: '계산기', path: 'calc.exe' },
        { label: '그림판', path: 'mspaint.exe' },
      ],
    },
    {
      id: 'term',
      kind: 'terminal',
      label: '터미널',
      icon: 'terminal',
      items: [{ label: '새 창', action: 'open' }],
    },
    {
      id: 'folder',
      kind: 'folder',
      label: '폴더',
      icon: 'folder',
      items: [{ label: '사진', target: 'Pictures' }],
    },
    {
      id: 'system',
      kind: 'system',
      label: '시스템',
      icon: 'power',
      items: [{ label: '잠금', fn: 'lock' }],
    },
  ];

  it('produces 4 sectors with step 90 when 4 categories provided', () => {
    const res = layout(sampleCategories, 0);
    expect(res.sectorCount).toBe(4);
    expect(res.step).toBe(90);
    expect(res.sectors.length).toBe(4);
    expect(res.addSector).toBeNull();
    expect(res.selection.visible).toBe(true);
    expect(res.selection.axis).toBe(0);
    expect(res.sub).not.toBeNull();
  });

  it('adds addSector when categories < 4', () => {
    const twoCats = sampleCategories.slice(0, 2);
    const res = layout(twoCats, 0);
    expect(res.sectorCount).toBe(4); // min 4
    expect(res.sectors.length).toBe(2);
    expect(res.addSector).not.toBeNull();
    expect(res.addSector?.axis).toBe(180); // 2 * 90 = 180
  });

  it('handles hot = -1 (no category selected)', () => {
    const res = layout(sampleCategories, -1);
    expect(res.selection.visible).toBe(false);
    expect(res.selection.axis).toBeNull();
    expect(res.sub).toBeNull();
  });

  it('reverses slot order when axis >= 180', () => {
    // 6시 방향 (axis = 180)
    const res = layout(sampleCategories, 2);
    expect(res.sub).not.toBeNull();
    expect(res.sub?.axis).toBe(180);
    // reversed: pos = slots - 1 - k
    const slots = res.sub!.slots;
    expect(slots.length).toBe(2); // 1 item + 1 add
    expect(slots[0].kind).toBe('item');
    expect(slots[1].kind).toBe('add');
  });

  it('matches ReferenceLayout.layout 100% identically across different configs', () => {
    for (let catCount = 1; catCount <= 4; catCount++) {
      const cats = sampleCategories.slice(0, catCount);
      for (let hot = -1; hot < catCount; hot++) {
        const tsRes = layout(cats, hot);
        const jsRes = ReferenceLayout.layout(cats, hot);

        expect(tsRes.sectorCount).toBe(jsRes.sectorCount);
        expect(tsRes.step).toBe(jsRes.step);
        expect(tsRes.dividers).toBe(jsRes.dividers);
        expect(tsRes.selection.visible).toBe(jsRes.selection.visible);
        expect(tsRes.selection.axis).toBe(jsRes.selection.axis);
        expect(tsRes.selection.wedge).toBe(jsRes.selection.wedge);
        expect(tsRes.selection.edge).toBe(jsRes.selection.edge);
        expect(tsRes.selection.arc).toBe(jsRes.selection.arc);

        if (jsRes.addSector) {
          expect(tsRes.addSector).not.toBeNull();
          expect(tsRes.addSector!.path).toBe(jsRes.addSector.path);
          expect(tsRes.addSector!.labelX).toBe(jsRes.addSector.labelX);
          expect(tsRes.addSector!.labelY).toBe(jsRes.addSector.labelY);
        } else {
          expect(tsRes.addSector).toBeNull();
        }

        if (jsRes.sub) {
          expect(tsRes.sub).not.toBeNull();
          expect(tsRes.sub!.axis).toBe(jsRes.sub.axis);
          expect(tsRes.sub!.band).toBe(jsRes.sub.band);
          expect(tsRes.sub!.dividers).toBe(jsRes.sub.dividers);
          expect(tsRes.sub!.slots.length).toBe(jsRes.sub.slots.length);
          for (let s = 0; s < jsRes.sub.slots.length; s++) {
            expect(tsRes.sub!.slots[s].path).toBe(jsRes.sub.slots[s].path);
            expect(tsRes.sub!.slots[s].labelX).toBe(jsRes.sub.slots[s].labelX);
            expect(tsRes.sub!.slots[s].labelY).toBe(jsRes.sub.slots[s].labelY);
            expect(tsRes.sub!.slots[s].delayMs).toBe(
              jsRes.sub.slots[s].delayMs
            );
          }
        } else {
          expect(tsRes.sub).toBeNull();
        }
      }
    }
  });

  describe('Empty category (empty slot) handling', () => {
    const configWithEmpty: CategoryConfig[] = [
      {
        id: 'cat-1',
        kind: 'app',
        label: '앱',
        items: [{ label: '메모장', path: 'notepad.exe' }],
      },
      {
        id: 'cat-2',
        kind: 'terminal',
        label: '터미널',
        items: [{ label: 'PowerShell', action: 'open' }],
      },
      {
        id: 'slot-3',
        kind: 'empty',
        label: '빈칸',
        items: [],
      },
      {
        id: 'cat-4',
        kind: 'system',
        label: '시스템',
        items: [{ label: '잠금', fn: 'lock' }],
      },
    ];

    it('creates 4 sectors with step 90 and places empty category in slot 2', () => {
      const res = layout(configWithEmpty, 0);
      expect(res.sectorCount).toBe(4);
      expect(res.step).toBe(90);
      expect(res.sectors.length).toBe(4);
      expect(res.sectors[2].category.kind).toBe('empty');
      expect(res.sectors[2].axis).toBe(180);
      expect(res.sectors[2].selected).toBe(false);
      expect(res.addSector).toBeNull();
    });

    it('does not generate sub-band or selection when hot points to empty slot', () => {
      const res = layout(configWithEmpty, 2);
      expect(res.selection.visible).toBe(false);
      expect(res.selection.axis).toBeNull();
      expect(res.sub).toBeNull();
      expect(res.sectors[2].selected).toBe(false);
    });

    it('generates sub-band normally when hot points to valid slot 3 (system)', () => {
      const res = layout(configWithEmpty, 3);
      expect(res.selection.visible).toBe(true);
      expect(res.selection.axis).toBe(270);
      expect(res.sub).not.toBeNull();
      expect(res.sub?.axis).toBe(270);
    });
  });
});
