# Radial_menu

Windows용 2단 원형(radial) 메뉴 앱. `Ctrl` + 휠클릭으로 커서 위치에 메뉴를 열고, `Ctrl`을 떼면 가리킨 항목을 실행합니다.

- 분류(앱 / 터미널 / 폴더·주소 / 시스템)와 항목으로 구성된 2단 메뉴
- 항목은 화면에서 직접 추가하거나, 에이전트가 `rmctl` CLI로 추가
- 관리자 권한으로 상주하며, 관리자 PowerShell 등은 승인한 항목만 실행

**스택**: Tauri 2 (Rust + WebView2) + TypeScript

- 설계 문서: [docs/](docs/README.md)
- 디자인 핸드오프: [UI_designs/](UI_designs/HANDOFF.md)
