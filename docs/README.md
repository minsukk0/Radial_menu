# Radial Menu 문서

Ctrl+휠클릭으로 커서 위치에 뜨는 Windows용 2단 원형(radial) 메뉴 앱의 설계 문서입니다.

![메뉴 시안](../UI_designs/reference/screens/01-menu-apps.png)

## 문서 목록

| 문서 | 내용 |
|---|---|
| [architecture.md](architecture.md) | 기술 스택(Tauri 2), 프로세스·창 구성, 모듈 구조, 디렉터리 구조, 데이터 흐름 |
| [interaction.md](interaction.md) | Ctrl+휠클릭으로 열고 Ctrl을 떼면 실행하는 입력 방식, 상태 머신, 판정 규칙 |
| [configuration.md](configuration.md) | `menu.json` 위치·스키마, 분류별 항목 필드, 자리표시 |
| [agent-integration.md](agent-integration.md) | 에이전트가 코드로 항목을 추가하는 방법 (`rmctl` CLI, JSON Schema) |
| [elevation-and-security.md](elevation-and-security.md) | 관리자 권한 상주(전략 B), 권한 낮추기, 관리자 항목 승인, 설치 |
| [roadmap.md](roadmap.md) | 단계별 구현 계획, 먼저 할 실험, 미결정 사항 |

## 디자인 원본

화면 모양, 치수, 색, 모션의 기준은 [`UI_designs/`](../UI_designs/HANDOFF.md)입니다. 이 문서들은 디자인 핸드오프를 **수정하지 않고** 참조합니다. 핸드오프와 다른 결정은 각 문서에 "핸드오프와 다른 점"으로 적었습니다.

| 원본 | 쓰임 |
|---|---|
| `UI_designs/HANDOFF.md` | 화면 규격, 배치 규칙, 상태, 모션 |
| `UI_designs/tokens/tokens.css`, `tokens.json` | 디자인 토큰 (`--rm-*`) |
| `UI_designs/reference/radial-layout.js` | 칸 모양·위치·판정 계산 (그대로 TS로 이식) |
| `UI_designs/reference/menu-standalone.html` | 메뉴 화면 기준 구현 |
| `UI_designs/prototype/add-item.dc.html` | 항목 추가 창 마크업 |

## 한눈에 보기

- **스택**: Tauri 2 (Rust 백엔드 + WebView2) + TypeScript 화면
- **입력**: `Ctrl` + 휠클릭으로 열기 → 방향으로 가리키기 → `Ctrl`을 떼면 실행
- **메뉴**: 상위 칸 = 분류(앱 / 터미널 / 폴더·주소 / 시스템), 하위 칸 = 항목
- **설정**: `%APPDATA%\RadialMenu\menu.json` 한 파일. 화면(항목 추가 창), 에이전트(`rmctl` CLI), 직접 편집 모두 같은 파일을 씀
- **권한**: 앱은 관리자 권한으로 상주, 일반 프로그램은 권한을 낮춰 실행, 관리자 항목은 승인한 것만 실행
