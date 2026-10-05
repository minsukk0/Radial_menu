# 에이전트 연동: 코드로 항목 추가하기

> 관련 문서: [설정](configuration.md) · [권한 & 보안](elevation-and-security.md)

항목 추가 창으로 하나씩 넣기 귀찮을 때, 코딩 에이전트(Antigravity 등)나 스크립트가 메뉴를 고칠 수 있게 한다.

## 1. 세 가지 방법

| 방법 | 언제 | 비고 |
|---|---|---|
| **`rmctl` CLI** (권장) | 항목 추가·수정·삭제·순서 변경 | 검증, `id` 자동 생성, 잠금, 원자적 쓰기, 결과를 JSON으로 출력 |
| `menu.json` 직접 편집 | 여러 항목을 한꺼번에 바꿀 때 | 편집 후 `rmctl validate`로 확인 |
| 항목 추가 창 | 사람이 직접 | |

세 방법 모두 같은 파일(`%APPDATA%\RadialMenu\menu.json`)을 쓰고, 앱은 파일 변경을 감시해 바로 다시 읽는다. **앱을 재시작할 필요 없다.**

## 2. `rmctl` 명령

`rmctl`은 일반 권한 콘솔 프로그램이다. 설치 폴더(`C:\Program Files\RadialMenu\rmctl.exe`)에 있고 PATH에 등록된다. 개발 중에는 `cargo run -p rmctl -- <명령>`.

```
rmctl path                              # 설정 파일 경로 출력
rmctl schema                            # JSON Schema 출력
rmctl validate [--file <path>]          # 설정 검증
rmctl list [--category <id>]            # 분류·항목 목록

rmctl category add --kind <app|terminal|folder|system> --label <이름> [--id <id>] [--at <순서>]
rmctl category remove <id>

rmctl item add --category <id|kind> --json '<항목 JSON>' [--at <순서>]
rmctl item add --category <id|kind> --file item.json
rmctl item update <item-id> --json '<바꿀 필드만>'
rmctl item remove <item-id>
rmctl item move <item-id> --to <순서> [--category <id>]
```

### 출력 규칙 (에이전트가 읽기 쉽게)

- 모든 명령은 **`--json`이 기본**이다. 사람용 표는 `--pretty`.
- 성공: 종료 코드 `0`, 표준 출력에 `{"ok": true, ...}`.
- 실패: 종료 코드 `1`(검증 실패) 또는 `2`(사용법 오류), 표준 출력에 오류 목록.

```json
{ "ok": false, "errors": [
  { "path": "/categories/1/items/0/shell", "message": "'bash'는 허용되지 않음. powershell | cmd | wsl | gitbash" }
] }
```

- `--category`에 `kind`를 주면 그 종류의 **첫 분류**에 넣는다. 그 종류의 분류가 없으면 `--create-category`를 줄 때만 새로 만든다.
- 성공 응답에는 생성된 `id`와, 관리자 항목이면 `"needsApproval": true`가 들어간다.

### 예시

```powershell
# 앱 추가
rmctl item add --category app --json '{"label":"VS Code","path":"C:\\Users\\me\\AppData\\Local\\Programs\\Microsoft VS Code\\Code.exe"}'

# 터미널 명령 추가 (선택한 파일을 변환)
rmctl item add --category terminal --json '{
  "label": "PNG로 변환", "icon": "play", "action": "run", "mode": "command",
  "command": "magick \"{선택한 파일}\" \"{현재 폴더}\\out.png\"", "notifyOnDone": true }'

# 관리자 항목 (처음 실행할 때 승인 창이 뜬다)
rmctl item add --category terminal --json '{"label":"DNS 초기화","action":"run","command":"ipconfig /flushdns","runAsAdmin":true}'

# 폴더 분류가 없으면 만들면서 추가
rmctl item add --category folder --create-category --json '{"label":"다운로드","target":"%USERPROFILE%\\Downloads"}'
```

## 3. 직접 편집할 때

- 파일 첫 줄의 `"$schema"` 덕분에 편집기와 에이전트가 스키마로 자동 검증할 수 있다. 스키마 원본은 `rmctl schema` 또는 레포의 [`schema/menu.schema.json`](../schema/menu.schema.json).
- 편집이 끝나면 `rmctl validate`를 실행한다.
- 앱이 다시 읽다가 검증에 실패하면 **이전 설정을 유지**하고 트레이 알림으로 오류 경로를 보여 준다. 메뉴가 망가지지 않는다.
- `id`는 바꾸지 않는다. 관리자 항목 승인은 `id`에 묶여 있다.

## 4. 에이전트가 지킬 규칙

에이전트용 요약은 레포 루트의 `AGENTS.md`에 둔다 (구현 단계에서 작성). 내용은 다음과 같다.

1. **가능하면 `rmctl`을 쓴다.** 직접 편집했으면 반드시 `rmctl validate`를 실행한다.
2. 설정 형식은 `rmctl schema`가 기준이다. 문서와 다르면 스키마를 따른다.
3. 자리표시는 `{클립보드}`, `{선택한 파일}`, `{현재 폴더}`, `{활성 창 제목}` 네 가지뿐이다. 따옴표로 감싸서 쓴다.
4. **`runAsAdmin: true`를 넣어도 바로 관리자로 실행되지 않는다.** 사용자가 처음 실행할 때 승인해야 한다. 사용자에게 "처음 실행 시 승인 창이 뜬다"고 알린다.
5. 승인 목록(`%ProgramData%\RadialMenu\approvals.json`)은 건드리지 않는다. 일반 권한으로는 쓸 수도 없다.
6. 시스템 기능(`fn`)은 한 번씩만 등록할 수 있다.

## 5. 다른 작업 공간에서 쓰기 (선택)

이 레포가 아닌 곳에서 일하는 에이전트도 쓸 수 있게, `rmctl` 사용법을 담은 **전역 스킬**을 둘 수 있다 (예: "메뉴에 이 스크립트 추가해 줘"). 스킬은 이 문서 §2와 §4를 요약하고 `rmctl schema`로 최신 형식을 확인하도록 안내한다. Phase 4에서 결정한다.

## 6. 왜 CLI가 앱에 직접 명령을 보내지 않는가

`rmctl`이 실행 중인 앱에 파이프나 소켓으로 "항목 추가"를 보내게 하면 편하지만, 그 통로는 **일반 권한 프로그램 누구나 관리자 앱에 말을 걸 수 있는 문**이 된다. 그래서 `rmctl`은 사용자 권한으로 쓸 수 있는 설정 파일만 고치고, 관리자 권한이 필요한 부분(승인)은 사용자가 관리자 앱의 창에서 직접 누르게 했다. 자세한 내용은 [권한 & 보안](elevation-and-security.md).
