# 설정 (`menu.json`)

> 관련 문서: [에이전트 연동](agent-integration.md) · [권한 & 보안](elevation-and-security.md) · [디자인 핸드오프 §3](../UI_designs/HANDOFF.md)

## 1. 위치

| 파일 | 경로 | 쓰는 쪽 |
|---|---|---|
| 메뉴 설정 | `%APPDATA%\RadialMenu\menu.json` | 앱(항목 추가 창), `rmctl`, 직접 편집 |
| 직전 백업 | `%APPDATA%\RadialMenu\menu.json.bak` | 저장할 때마다 자동 |
| 관리자 항목 승인 목록 | `%ProgramData%\RadialMenu\approvals.json` | **앱만** (관리자만 쓰기 가능) |
| 스키마 | 설치 폴더 `schema\menu.schema.json`, 레포 `schema/menu.schema.json` | 생성물 |

- 개발 중에는 환경 변수 `RADIAL_MENU_CONFIG`로 다른 파일을 가리킬 수 있다.
- 저장하면 앱이 바로 다시 읽는다. 검증에 실패하면 마지막 정상 설정을 유지하고 오류를 알린다.

## 2. 최상위 구조

```json
{
  "$schema": "https://radial-menu.local/schema/menu.schema.json",
  "version": 1,
  "accent": "#1E9BFF",
  "categories": [ ]
}
```

| 필드 | 설명 |
|---|---|
| `$schema` | 편집기·에이전트의 자동 검증용. 앱은 무시 |
| `version` | 설정 형식 버전. 현재 `1` |
| `accent` | 강조색. 선택지 `#1E9BFF`(기본), `#3D7BFF`, `#6E56CF`, `#D6409F` (핸드오프 §6) |
| `categories` | 분류 목록. **배열 순서가 화면 순서** (12시부터 시계 방향) |

## 3. 분류

```json
{ "id": "apps", "kind": "app", "label": "앱", "items": [ ] }
```

| 필드 | 설명 |
|---|---|
| `id` | 고유 식별자. 영문 소문자·숫자·`-`. 생략하면 저장 시 자동 생성 |
| `kind` | `app` \| `terminal` \| `folder` \| `system`. 한 분류에는 한 종류의 항목만 담는다 |
| `label` | 상위 칸에 표시할 이름 |
| `items` | 항목 목록. 배열 순서가 하위 칸 순서 |

- 상위 칸 수 = max(4, 분류 수). 하위 칸 수 = max(4, 항목 수 + 1) ("추가" 칸 포함). 배치 규칙은 핸드오프 §4.2.

## 4. 항목

모든 항목에 공통으로 `id`(생략 시 자동 생성)와 `label`이 있다. 아래 필드는 항목 추가 창의 입력란과 일대일로 대응한다.

> [!NOTE]
> **핸드오프와 다른 점**: 분류와 항목에 `id`를 추가했습니다. 에이전트가 특정 항목을 고치거나 지울 때, 그리고 관리자 항목 승인을 항목에 묶을 때 필요합니다.

### 4.1 앱 (`kind: "app"`)

```json
{ "id": "notepad", "label": "메모장", "path": "C:\\Windows\\System32\\notepad.exe",
  "args": "", "whenRunning": "focus", "runAsAdmin": false }
```

| 필드 | 값 | 기본 |
|---|---|---|
| `path` | 실행 파일 경로 (필수) | |
| `args` | 실행 인수 문자열 | `""` |
| `whenRunning` | `focus`(창 앞으로 가져오기) \| `new`(새로 실행) | `focus` |
| `runAsAdmin` | 관리자 권한으로 실행 | `false` |

아이콘은 실행 파일에서 자동으로 뽑는다.

### 4.2 터미널 (`kind: "terminal"`)

**새 창 열기** (`action: "open"`)

```json
{ "id": "ps-here", "label": "새 창", "icon": "window", "action": "open",
  "shell": "powershell", "startDir": "{현재 폴더}", "runAsAdmin": false, "reuseTab": true }
```

**명령·스크립트 실행** (`action: "run"`)

```json
{ "id": "convert", "label": "변환 스크립트", "icon": "terminal", "action": "run",
  "mode": "command", "command": "python convert.py \"{선택한 파일}\" --out \"{현재 폴더}\"",
  "shell": "powershell", "workDir": "{현재 폴더}", "window": "hidden",
  "runAsAdmin": false, "notifyOnDone": true, "copyOutput": false, "confirmFirst": false }
```

| 필드 | 값 | 기본 |
|---|---|---|
| `icon` | `terminal` \| `window` \| `folder` \| `note` \| `globe` \| `activity` \| `timer` \| `play` | `terminal` |
| `action` | `run` \| `open` (필수) | |
| `mode` | `command`(한 줄 명령) \| `file`(파일 지정) — `run`일 때 | `command` |
| `command` | 명령 문자열 — `mode: command`일 때 | |
| `file`, `args` | 스크립트 파일과 넘길 값 — `mode: file`일 때 | |
| `shell` | `powershell` \| `cmd` \| `wsl` \| `gitbash` | `powershell` |
| `workDir` / `startDir` | 작업 폴더 / 시작 폴더 | 사용자 홈 |
| `window` | `hidden` \| `visible` \| `keep`(끝난 뒤 창 유지) — `run`일 때 | `hidden` |
| `runAsAdmin` | 관리자 권한 | `false` |
| `notifyOnDone` | 끝나면 알림 | `false` |
| `copyOutput` | 출력을 클립보드에 복사 | `false` |
| `confirmFirst` | 실행 전에 한 번 확인 | `false` |
| `reuseTab` | 열린 터미널이 있으면 새 탭으로 — `open`일 때 | `true` |

> [!IMPORTANT]
> `runAsAdmin: true` 항목은 **처음 실행할 때 승인 창**이 뜹니다. 항목 추가 창에서 사용자가 직접 만든 항목은 저장할 때 승인됩니다. 항목 내용이나 `file`로 지정한 스크립트 내용이 바뀌면 다시 승인해야 합니다. ([권한 & 보안 §4](elevation-and-security.md#4-관리자-항목-승인))

### 4.3 폴더·주소 (`kind: "folder"`)

```json
{ "id": "pictures", "label": "사진", "icon": "folder", "target": "C:\\Users\\me\\Pictures", "openWith": "default" }
```

| 필드 | 값 | 기본 |
|---|---|---|
| `target` | 폴더 경로, 파일 경로 또는 URL (필수) | |
| `icon` | 위 터미널 아이콘 목록과 같음 | `folder` (URL이면 `globe`) |
| `openWith` | `default` \| `explorer` \| `browser` | `default` |

### 4.4 시스템 (`kind: "system"`)

```json
{ "id": "lock", "label": "잠금", "fn": "lock" }
```

| 필드 | 값 |
|---|---|
| `fn` | `lock` \| `sleep` \| `settings` \| `taskmgr` \| `mute` \| `brightness` \| `screenoff` \| `emptybin` (필수) |

- 한 기능은 한 번만 등록할 수 있다 (같은 `fn` 중복 금지).
- 아이콘은 기능에 따라 정해진다.

## 5. 자리표시

명령, 경로, 작업 폴더에 넣을 수 있다. 값은 **메뉴를 연 순간** 기준이다.

| 자리표시 | 값 |
|---|---|
| `{클립보드}` | 클립보드의 텍스트 |
| `{선택한 파일}` | 메뉴를 열 때 앞에 있던 탐색기 창에서 선택한 파일 (여러 개면 공백으로 구분, 각각 따옴표) |
| `{현재 폴더}` | 앞에 있던 탐색기 창의 폴더. 탐색기가 아니면 사용자 홈 |
| `{활성 창 제목}` | 메뉴를 열 때 앞에 있던 창의 제목 |

**치환 방식**: 문자열에 값을 그대로 이어 붙이지 않는다. 값을 환경 변수(`RM_CLIPBOARD`, `RM_SELECTED`, `RM_CWD`, `RM_WINDOW_TITLE`)로 넘기고, 자리표시는 셸별 변수 참조로 바꾼다.

| 셸 | `{클립보드}`가 바뀌는 모양 |
|---|---|
| `powershell` | `$env:RM_CLIPBOARD` |
| `cmd` | `!RM_CLIPBOARD!` (지연 확장 켬) |
| `wsl`, `gitbash` | `"$RM_CLIPBOARD"` |

이렇게 하면 클립보드에 따옴표나 `;` 같은 문자가 있어도 명령이 깨지거나 다른 명령이 끼어들지 않는다. 관리자 항목에서는 특히 중요하다.

## 6. 전체 예시

[`examples/menu.example.json`](../examples/menu.example.json) (구현 단계에서 작성)

```json
{
  "$schema": "https://radial-menu.local/schema/menu.schema.json",
  "version": 1,
  "accent": "#1E9BFF",
  "categories": [
    { "id": "apps", "kind": "app", "label": "앱", "items": [
      { "id": "notepad", "label": "메모장", "path": "C:\\Windows\\System32\\notepad.exe" }
    ] },
    { "id": "terminal", "kind": "terminal", "label": "터미널", "items": [
      { "id": "ps-here", "label": "새 창", "icon": "window", "action": "open", "shell": "powershell", "startDir": "{현재 폴더}" },
      { "id": "ps-admin", "label": "관리자 PS", "icon": "terminal", "action": "open", "shell": "powershell", "runAsAdmin": true },
      { "id": "flushdns", "label": "DNS 초기화", "icon": "activity", "action": "run", "mode": "command",
        "command": "ipconfig /flushdns", "window": "hidden", "runAsAdmin": true, "notifyOnDone": true }
    ] },
    { "id": "folders", "kind": "folder", "label": "폴더·주소", "items": [
      { "id": "downloads", "label": "다운로드", "target": "%USERPROFILE%\\Downloads" }
    ] },
    { "id": "system", "kind": "system", "label": "시스템", "items": [
      { "id": "lock", "label": "잠금", "fn": "lock" },
      { "id": "taskmgr", "label": "작업 관리자", "fn": "taskmgr" }
    ] }
  ]
}
```
