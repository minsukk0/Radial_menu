/**
 * ipc.ts — Tauri IPC 래퍼 모듈
 * Tauri 2 웹뷰 환경과 브라우저 단독 개발 환경 모두에서 안전하게 동작하도록 추상화
 */

type UnlistenFn = () => void;
type EventCallback<T = any> = (event: { event: string; payload: T }) => void;

interface TauriWindow {
  __TAURI__?: {
    core?: {
      invoke?: <T = any>(cmd: string, args?: Record<string, unknown>) => Promise<T>;
    };
    event?: {
      listen?: <T = any>(event: string, handler: EventCallback<T>) => Promise<UnlistenFn>;
      emit?: (event: string, payload?: unknown) => Promise<void>;
    };
  };
}

const tauriWindow = (typeof window !== 'undefined' ? window : {}) as unknown as TauriWindow;

export const isTauriEnvironment = (): boolean => {
  return typeof window !== 'undefined' && Boolean(tauriWindow.__TAURI__);
};

/**
 * Tauri invoke 래퍼
 */
export async function invokeCommand<T = any>(
  cmd: string,
  args?: Record<string, unknown>
): Promise<T> {
  if (tauriWindow.__TAURI__?.core?.invoke) {
    return tauriWindow.__TAURI__.core.invoke<T>(cmd, args);
  }
  console.debug(`[Mock IPC invoke] ${cmd}`, args);
  return Promise.resolve({} as T);
}

/**
 * Tauri 이벤트 수신 리스너 등록
 */
export async function listenEvent<T = any>(
  event: string,
  handler: (payload: T) => void
): Promise<UnlistenFn> {
  if (tauriWindow.__TAURI__?.event?.listen) {
    const unlisten = await tauriWindow.__TAURI__.event.listen<T>(event, (ev) => {
      handler(ev.payload);
    });
    return unlisten;
  }

  // 브라우저 개발 환경용 DOM CustomEvent 폴백
  const domHandler = (e: Event) => {
    const customEvent = e as CustomEvent<T>;
    handler(customEvent.detail);
  };
  window.addEventListener(event, domHandler);
  return () => window.removeEventListener(event, domHandler);
}

/**
 * Tauri 이벤트 전송(emit)
 */
export async function emitEvent(event: string, payload?: unknown): Promise<void> {
  if (tauriWindow.__TAURI__?.event?.emit) {
    return tauriWindow.__TAURI__.event.emit(event, payload);
  }

  // 브라우저 개발 환경용 DOM CustomEvent 발송
  console.debug(`[Mock IPC emit] ${event}`, payload);
  if (typeof window !== 'undefined') {
    window.dispatchEvent(new CustomEvent(event, { detail: payload }));
  }
}
