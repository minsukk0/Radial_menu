# Radial Menu — AI 에이전트 가이드 (`AGENTS.md`)

이 문서는 코딩 에이전트(Antigravity 등)나 외부 자동화 스크립트가 Radial Menu의 항목을 안전하고 정확하게 추가·수정할 수 있도록 안내하는 지침서입니다.

---

## 1. 기본 원칙
1. **가능하면 `rmctl` CLI를 사용하십시오.**
   - 경로: `rmctl.exe` (설치 시 PATH 등록, 개발 중: `cargo run -p rmctl -- <명령>`)
   - `rmctl`은 설정 잠금(`menu.json.lock`), 스키마 검증, `id` 자동 생성, 백업 파일 생성(`menu.json.bak`), 원자적 쓰기(Atomic write)를 자동으로 보장합니다.
2. **설정 형식의 기준은 `rmctl schema`입니다.**
   - 레포의 [`schema/menu.schema.json`](file:///d:/Python%20Workspace/Radial_menu/schema/menu.schema.json) 또는 `rmctl schema`의 JSON Schema를 엄격히 준수하십시오.
3. **직접 편집 시 반드시 `rmctl validate`를 실행하십시오.**
   - 설정 파일 위치: `%APPDATA%\RadialMenu\menu.json` (개발 중: `RADIAL_MENU_CONFIG` 환경 변수로 재정의 가능)
   - 앱이 실행 중인 상태에서 유효하지 않은 설정이 저장되면, 앱은 안전을 위해 마지막 정상 설정을 유지하고 트레이 알림으로 오류 경로를 표시합니다.
4. **기존 `id`는 임의로 변경하거나 재사용하지 마십시오.**
   - 관리자 실행 승인 기록은 항목 `id`와 연계되어 있습니다.

---

## 2. 지원하는 4대 분류 (Category Kinds)

한 분류에는 반드시 한 종류(`kind`)의 항목만 담깁니다.

### A. 앱 (`kind: "app"`)
```bash
rmctl item add --category app --json '{"label":"VS Code","path":"C:\\Users\\user\\AppData\\Local\\Programs\\Microsoft VS Code\\Code.exe","whenRunning":"focus","runAsAdmin":false}'
```
- `path`: 실행 파일 절대 경로 (필수)
- `args`: 커맨드라인 인수 문자열 (기본: `""`)
- `whenRunning`: `focus` (이미 실행 중이면 창을 포커스) | `new` (새 인스턴스 실행) (기본: `focus`)
- `runAsAdmin`: 관리자 권한 실행 여부 (기본: `false`)

### B. 터미널 (`kind: "terminal"`)
```bash
# 1) 새 창 열기 (open)
rmctl item add --category terminal --json '{"label":"현재 폴더 터미널","action":"open","shell":"powershell","startDir":"{현재 폴더}"}'

# 2) 명령 실행 (run)
rmctl item add --category terminal --json '{"label":"PNG 변환","action":"run","mode":"command","command":"magick \"{선택한 파일}\" \"{현재 폴더}\\out.png\"","notifyOnDone":true}'
```
- `action`: `open` (새 창 열기) | `run` (명령/스크립트 실행) (필수)
- `shell`: `powershell` | `cmd` | `wsl` | `gitbash` (기본: `powershell`)
- `window`: `hidden` (백그라운드) | `visible` (콘솔 창 표시) | `keep` (실행 후 창 유지) (기본: `hidden`)
- `notifyOnDone`: 완료 시 시스템 알림 표시 (기본: `false`)
- `copyOutput`: 완료 시 표준 출력을 클립보드에 복사 (기본: `false`)

### C. 폴더 및 주소 (`kind: "folder"`)
```bash
rmctl item add --category folder --json '{"label":"다운로드 폴더","target":"%USERPROFILE%\\Downloads","openIn":"explorer"}'
```
- `target`: 로컬 디렉터리 경로 또는 웹 URL (필수)
- `openIn`: `explorer` | `browser` | `current` (기본: `explorer`)
- `selectTarget`: 파일/폴더 선택 상태로 탐색기 열기 (기본: `false`)

### D. 시스템 기능 (`kind: "system"`)
```bash
rmctl item add --category system --json '{"label":"화면 잠금","fn":"lock"}'
```
- `fn`: `lock` | `sleep` | `screensaver` | `displayOff` | `volumeMute` | `radialSettings` (필수)
- **주의**: 시스템 기능은 메뉴 전체에서 기능별로 **최대 1회만** 등록 가능합니다.

---

## 3. 자리표시자 (Placeholders)

터미널 및 경로 필드에서 다음 4가지 동적 컨텍스트 자리표시자를 사용할 수 있습니다. 명령어 인젝션 및 공백 깨짐을 방지하기 위해 반드시 큰따옴표(`"..."`)로 감싸서 작성하십시오.

| 자리표시자 | 설명 | 수집 방식 |
|---|---|---|
| `{현재 폴더}` | 현재 포그라운드 파일 탐색기의 디렉터리 경로 | `IShellWindows` Win32 COM 인터페이스 |
| `{선택한 파일}` | 현재 탐색기에서 선택된 파일의 전체 경로 | `IShellWindows` 선택 항목 열거 |
| `{활성 창 제목}` | 메뉴를 열기 직전에 포커스되어 있던 창의 제목 | Win32 `GetForegroundWindow` / `GetWindowTextW` |
| `{클립보드}` | 시스템 클립보드에 저장된 텍스트 | Windows Clipboard API |

---

## 4. 관리자 권한 및 보안 승인 정책

Radial Menu는 편의성을 위해 시스템 관리자 권한으로 상주하지만, 강력한 샌드박스 및 권한 축소(De-elevation) 원칙을 준수합니다.

1. **기본 일반 권한 강제 (De-elevation)**:
   - `runAsAdmin: false`인 모든 일반 앱, 브라우저, 폴더, 터미널은 `explorer.exe`의 사용자 토큰을 복제(`CreateProcessWithTokenW`)하여 일반 권한으로 강제 축소 실행됩니다.
2. **에이전트 생성 관리자 항목의 승인 필수**:
   - 에이전트가 `rmctl` 또는 `menu.json` 편집으로 `runAsAdmin: true` 항목을 추가한 경우, **곧바로 관리자로 실행되지 않습니다.**
   - 사용자가 처음 해당 항목을 실행할 때, 앱 자체 관리자 다이얼로그(승인 창)가 화면에 나타납니다.
   - 사용자가 승인 버튼을 직접 클릭해야만 해시가 `%ProgramData%\RadialMenu\approvals.json`에 영구 기록되어 관리자로 실행됩니다.
   - **에이전트는 이 승인 절차를 우회할 수 없으며, 일반 권한 프로세스는 UIPI 보호로 인해 승인 창을 조작할 수 없습니다.**
   - 사용자에게 안내 시: *"해당 관리자 항목은 보안 정책에 따라 사용자가 처음 실행할 때 승인 창이 표시됩니다."* 라고 알리십시오.
