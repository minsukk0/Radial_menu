/**
 * radial-layout.ts — 휠클릭 래디얼 메뉴의 배치 계산 모듈
 * UI_designs/reference/radial-layout.js 완벽 이식
 *
 * 화면 기술과 무관한 순수 함수 모음이다. 분류 목록과 "지금 가리키는 분류"를 넣으면
 * 각 칸의 모양(SVG path 문자열), 이름 위치, 하위 칸 배치, 등장 지연 시간을 돌려준다.
 *
 * 좌표: 메뉴 중심이 (cx, cy). 각도는 12시 방향이 0도이고 시계 방향으로 커진다.
 * 길이 단위는 100% 배율 기준 px. 값의 출처는 tokens/tokens.json의 "menu" 묶음이다.
 */

import type {
  GeometryConfig,
  LayoutOptions,
  LayoutSector,
  LayoutAddSector,
  LayoutSelection,
  LayoutSlot,
  LayoutSub,
  LayoutResult,
  HitTestResult,
  HitTestZone,
  CategoryConfig,
} from '../shared/types';

export const GEOMETRY: GeometryConfig = {
  hubRadius: 30,
  arcRadius: 36,
  ringInner: 38,
  ringOuter: 186,
  ringLabelRadius: 116,
  ringMinSectors: 4,
  selectionSpill: 218,
  bandInner: 196,
  bandOuter: 278,
  bandCorner: 14,
  bandEndPadding: 2,
  slotInner: 200,
  slotOuter: 274,
  slotSideInset: 2,
  slotCorner: 10,
  slotAngle: 24,
  slotMaxSpan: 216,
  slotMinCount: 4,
  slotLabelRadius: 237,
  slotDividerInner: 210,
  slotDividerOuter: 264,
  slotFirstDelay: 50,
  slotStagger: 30,
  slotStaggerBudget: 150,
};

export function fmt(v: number): string {
  return String(Math.round(v * 10) / 10);
}

export interface GeometryHelpers {
  xy: (r: number, deg: number) => [number, number];
  P: (r: number, deg: number) => string;
  wedge: (r0: number, r1: number, a0: number, a1: number) => string;
  pad: (
    r0: number,
    r1: number,
    a0: number,
    a1: number,
    inset: number,
    corner: number
  ) => string;
}

export function make(cx: number, cy: number): GeometryHelpers {
  // 반지름 방향 단위 벡터와 접선 방향 단위 벡터
  function U(deg: number): [number, number] {
    const a = (deg * Math.PI) / 180;
    return [Math.sin(a), -Math.cos(a)];
  }

  function T(deg: number): [number, number] {
    const a = (deg * Math.PI) / 180;
    return [Math.cos(a), Math.sin(a)];
  }

  function xy(r: number, deg: number): [number, number] {
    const u = U(deg);
    return [cx + r * u[0], cy + r * u[1]];
  }

  function P(r: number, deg: number): string {
    const p = xy(r, deg);
    return fmt(p[0]) + ' ' + fmt(p[1]);
  }

  // 각진 부채꼴: 상위 칸, 선택 하이라이트
  function wedge(r0: number, r1: number, a0: number, a1: number): string {
    const big = a1 - a0 > 180 ? 1 : 0;
    return (
      'M' +
      P(r0, a0) +
      'L' +
      P(r1, a0) +
      'A' +
      r1 +
      ' ' +
      r1 +
      ' 0 ' +
      big +
      ' 1 ' +
      P(r1, a1) +
      'L' +
      P(r0, a1) +
      'A' +
      r0 +
      ' ' +
      r0 +
      ' 0 ' +
      big +
      ' 0 ' +
      P(r0, a0) +
      'Z'
    );
  }

  // 네 모서리가 둥근 부채꼴: 하위 띠, 하위 칸
  // inset: 옆 변을 각도 경계선에서 안쪽으로 평행 이동하는 거리(px), corner: 모서리 반지름
  function pad(
    r0: number,
    r1: number,
    a0: number,
    a1: number,
    inset: number,
    corner: number
  ): string {
    function fillet(
      a: number,
      sgn: number,
      R: number,
      outer: boolean
    ): [string, string] {
      const Rc = outer ? R - corner : R + corner;
      const d = inset + corner;
      const s = Math.sqrt(Rc * Rc - d * d);
      const u = U(a);
      const t = T(a);
      const k = R / Rc;
      return [
        fmt(cx + sgn * inset * t[0] + s * u[0]) +
          ' ' +
          fmt(cy + sgn * inset * t[1] + s * u[1]),
        fmt(cx + (sgn * d * t[0] + s * u[0]) * k) +
          ' ' +
          fmt(cy + (sgn * d * t[1] + s * u[1]) * k),
      ];
    }

    const so = fillet(a0, 1, r1, true);
    const eo = fillet(a1, -1, r1, true);
    const si = fillet(a0, 1, r0, false);
    const ei = fillet(a1, -1, r0, false);
    const q = 'A' + corner + ' ' + corner + ' 0 0 1 ';
    const big = a1 - a0 > 180 ? 1 : 0;

    return (
      'M' +
      si[0] +
      'L' +
      so[0] +
      q +
      so[1] +
      'A' +
      r1 +
      ' ' +
      r1 +
      ' 0 ' +
      big +
      ' 1 ' +
      eo[1] +
      q +
      eo[0] +
      'L' +
      ei[0] +
      q +
      ei[1] +
      'A' +
      r0 +
      ' ' +
      r0 +
      ' 0 ' +
      big +
      ' 0 ' +
      si[1] +
      q +
      si[0] +
      'Z'
    );
  }

  return { xy, P, wedge, pad };
}

/**
 * 메뉴 한 상태의 배치를 계산한다.
 *
 * @param categories 등록된 분류 (순서대로 12시부터 시계 방향)
 * @param hot 지금 가리키는 분류의 번호. 없으면 -1.
 * @param opts 좌표 중심 및 기하 파라미터 오버라이드
 */
export function layout(
  categories: CategoryConfig[],
  hot: number,
  opts?: LayoutOptions
): LayoutResult {
  const options = opts || {};
  const g: GeometryConfig = { ...GEOMETRY, ...(options.geometry || {}) };
  const cx = options.cx === undefined ? 380 : options.cx;
  const cy = options.cy === undefined ? 380 : options.cy;
  const m = make(cx, cy);

  // 상위 링: 분류 수만큼 균등 분배, 최소 4칸. 모자라는 칸은 빈칸.
  const count = categories.length;
  const n = Math.max(g.ringMinSectors, count);
  const step = 360 / n;
  const half = step / 2;
  let activeHot = hot;
  if (activeHot >= count) activeHot = -1;

  const sectors: LayoutSector[] = categories.map((cat, i) => {
    const axis = i * step;
    const p = m.xy(g.ringLabelRadius, axis);
    return {
      index: i,
      axis: axis,
      path: m.wedge(g.ringInner, g.ringOuter, axis - half, axis + half),
      labelX: Math.round(p[0] * 10) / 10,
      labelY: Math.round(p[1] * 10) / 10,
      selected: i === activeHot,
      category: cat,
    };
  });

  // 빈칸 중 첫 칸에만 "분류 추가"를 둔다.
  let addSector: LayoutAddSector | null = null;
  if (count < n) {
    const aa = count * step;
    const ap = m.xy(g.ringLabelRadius, aa);
    addSector = {
      axis: aa,
      path: m.wedge(g.ringInner, g.ringOuter, aa - half, aa + half),
      labelX: Math.round(ap[0] * 10) / 10,
      labelY: Math.round(ap[1] * 10) / 10,
    };
  }

  let dividers = '';
  for (let i = 0; i < n; i++) {
    dividers +=
      'M' +
      m.P(g.ringInner, (i + 0.5) * step) +
      'L' +
      m.P(g.ringOuter, (i + 0.5) * step);
  }

  // 선택 하이라이트: 12시 방향(축 0도) 기준 모양. 화면에서는 메뉴 중심을 축으로 axis 만큼 돌린다.
  const selection: LayoutSelection = {
    visible: activeHot >= 0,
    axis: activeHot >= 0 ? activeHot * step : null,
    wedge: m.wedge(g.ringInner, g.selectionSpill, -half, half),
    edge:
      'M' +
      m.P(g.ringInner, -half) +
      'L' +
      m.P(g.ringOuter, -half) +
      'A' +
      g.ringOuter +
      ' ' +
      g.ringOuter +
      ' 0 0 1 ' +
      m.P(g.ringOuter, half) +
      'L' +
      m.P(g.ringInner, half),
    arc:
      'M' +
      m.P(g.arcRadius, -half) +
      'A' +
      g.arcRadius +
      ' ' +
      g.arcRadius +
      ' 0 0 1 ' +
      m.P(g.arcRadius, half),
  };

  // 하위 링: 항목 수 + "추가" 칸 하나, 최소 4칸.
  // 칸 각도는 24도이고, 칸 각도의 합이 216도를 넘으면 216 / 칸 수로 줄인다.
  // 축이 아래쪽 반(180도 이상)이면 순서를 뒤집어 읽는 방향을 맞춘다.
  let sub: LayoutSub | null = null;
  if (activeHot >= 0) {
    const items = categories[activeHot].items || [];
    const axis = activeHot * step;
    const slots = Math.max(g.slotMinCount, items.length + 1);
    const slotAngle = Math.min(g.slotAngle, g.slotMaxSpan / slots);
    const span = slots * slotAngle + g.bandEndPadding * 2;
    const reversed = axis >= 180;
    const stagger = Math.min(g.slotStagger, g.slotStaggerBudget / slots);

    let lines = '';
    for (let j = 1; j < slots; j++) {
      const la = axis + (j - slots / 2) * slotAngle;
      lines += 'M' + m.P(g.slotDividerInner, la) + 'L' + m.P(g.slotDividerOuter, la);
    }

    const list: LayoutSlot[] = [];
    for (let k = 0; k <= items.length; k++) {
      const pos = reversed ? slots - 1 - k : k;
      const ang = axis + (pos - (slots - 1) / 2) * slotAngle;
      const sp = m.xy(g.slotLabelRadius, ang);
      list.push({
        kind: k < items.length ? 'item' : 'add',
        index: k,
        angle: ang,
        path: m.pad(
          g.slotInner,
          g.slotOuter,
          ang - slotAngle / 2,
          ang + slotAngle / 2,
          g.slotSideInset,
          g.slotCorner
        ),
        labelX: Math.round(sp[0] * 10) / 10,
        labelY: Math.round(sp[1] * 10) / 10,
        delayMs: Math.round(g.slotFirstDelay + k * stagger),
        item: k < items.length ? items[k] : null,
      });
    }

    sub = {
      axis: axis,
      slotCount: slots,
      slotAngle: slotAngle,
      span: span,
      band: m.pad(
        g.bandInner,
        g.bandOuter,
        axis - span / 2,
        axis + span / 2,
        0,
        g.bandCorner
      ),
      dividers: lines,
      slots: list,
    };
  }

  return {
    sectorCount: n,
    step: step,
    sectors: sectors,
    addSector: addSector,
    dividers: dividers,
    selection: selection,
    sub: sub,
  };
}

/**
 * 하이라이트를 돌릴 누적 각도를 구한다. 항상 가까운 방향으로 돌도록
 * 이전 누적 각도에 -180~180도 범위의 차이만 더한다.
 */
export function nextRotation(
  previous: number | undefined | null,
  targetAxis: number
): number {
  if (previous === undefined || previous === null) return targetAxis;
  return previous + ((((targetAxis - previous) % 360) + 540) % 360 - 180);
}

/**
 * 커서 위치로 가리키는 대상을 정한다.
 * 반환: { zone: 'hub' | 'ring' | 'band' | 'outside', sector: number, angle: number, radius: number }
 */
export function hitTest(
  x: number,
  y: number,
  sectorCount: number,
  opts?: LayoutOptions
): HitTestResult {
  const options = opts || {};
  const g: GeometryConfig = { ...GEOMETRY, ...(options.geometry || {}) };
  const cx = options.cx === undefined ? 380 : options.cx;
  const cy = options.cy === undefined ? 380 : options.cy;

  const dx = x - cx;
  const dy = y - cy;
  const radius = Math.sqrt(dx * dx + dy * dy);
  const angle = ((Math.atan2(dx, -dy) * 180) / Math.PI + 360) % 360;
  const step = 360 / sectorCount;
  const sector = Math.floor(((angle + step / 2) % 360) / step);

  const zone: HitTestZone =
    radius < g.ringInner
      ? 'hub'
      : radius <= g.ringOuter
      ? 'ring'
      : radius <= g.bandOuter
      ? 'band'
      : 'outside';

  return { zone, sector, angle, radius };
}
