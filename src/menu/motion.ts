/**
 * motion.ts — 래디얼 메뉴의 애니메이션 및 모션 제어 모듈
 * UI_designs/HANDOFF.md §4.4 기반
 */

export interface MotionConfig {
  selectionRotateDurMs: number;
  selectionFadeDurMs: number;
  bandPopDurMs: number;
  slotPopDurMs: number;
  slotFirstDelayMs: number;
  slotStaggerMs: number;
  reducedMotion: boolean;
}

export const DEFAULT_MOTION: MotionConfig = {
  selectionRotateDurMs: 220,
  selectionFadeDurMs: 150,
  bandPopDurMs: 220,
  slotPopDurMs: 240,
  slotFirstDelayMs: 50,
  slotStaggerMs: 30,
  reducedMotion: false,
};

/**
 * 사용자의 prefers-reduced-motion 설정 감지
 */
export function isReducedMotionPreferred(): boolean {
  if (typeof window === 'undefined' || !window.matchMedia) return false;
  return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
}

/**
 * 하이라이트 요소의 회전 각도 및 가시성 적용
 */
export function updateHighlightMotion(
  selElement: HTMLElement,
  rotation: number | undefined,
  visible: boolean,
  reduced: boolean = isReducedMotionPreferred()
): void {
  if (visible && rotation !== undefined) {
    selElement.classList.remove('off');
    if (reduced) {
      selElement.style.transition = 'none';
      selElement.style.transform = `rotate(${rotation}deg)`;
    } else {
      selElement.style.removeProperty('transition');
      selElement.style.transform = `rotate(${rotation}deg)`;
    }
  } else {
    selElement.classList.add('off');
  }
}

/**
 * 하위 띠 및 칸 등장 애니메이션 재시작을 위해 요소를 리셋 후 트리거
 */
export function triggerEntranceMotion(element: HTMLElement): void {
  // CSS animation 재트리거를 위한 리플로우
  element.style.animation = 'none';
  // offsetHeight 읽기로 강제 reflow
  void element.offsetHeight;
  element.style.removeProperty('animation');
}
