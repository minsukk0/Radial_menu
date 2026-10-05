/// Pure logic state machine for mouse & keyboard gesture detection.
/// No Win32 dependencies; fully unit-testable.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GestureState {
    Idle,
    Open,
    Committing,
    Executing,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GestureEvent {
    MButtonDown { ctrl_down: bool, x: i32, y: i32 },
    MButtonUp,
    CtrlUp,
    LButtonDown { x: i32, y: i32 },
    RButtonDown,
    RButtonUp,
    EscapeDown,
    EscapeUp,
    OtherKeyDown,
    OtherKeyUp,
    MouseWheel { delta: i32 },
    MouseMove { x: i32, y: i32 },
    CommitResponse(Option<String>),
    Timeout,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GestureAction {
    OpenMenu { x: i32, y: i32 },
    CloseMenu,
    RequestCommit,
    ExecuteTarget(String),
    CursorMove { x: i32, y: i32 },
}

#[derive(Debug, Clone)]
pub struct GestureStateMachine {
    state: GestureState,
    swallow_next_mbutton_up: bool,
    swallow_next_rbutton_up: bool,
    swallow_next_escape_up: bool,
}

impl Default for GestureStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl GestureStateMachine {
    pub fn new() -> Self {
        Self {
            state: GestureState::Idle,
            swallow_next_mbutton_up: false,
            swallow_next_rbutton_up: false,
            swallow_next_escape_up: false,
        }
    }

    pub fn state(&self) -> GestureState {
        self.state
    }

    pub fn is_open(&self) -> bool {
        matches!(self.state, GestureState::Open | GestureState::Committing)
    }

    /// Process an input event and return `(swallow_input, actions)`.
    pub fn handle_event(&mut self, event: GestureEvent) -> (bool, Vec<GestureAction>) {
        match event {
            GestureEvent::MButtonDown { ctrl_down, x, y } => {
                if ctrl_down {
                    self.swallow_next_mbutton_up = true;
                    match self.state {
                        GestureState::Idle => {
                            self.state = GestureState::Open;
                            (true, vec![GestureAction::OpenMenu { x, y }])
                        }
                        GestureState::Open | GestureState::Committing => {
                            self.state = GestureState::Idle;
                            (true, vec![GestureAction::CloseMenu])
                        }
                        GestureState::Executing => (true, vec![]),
                    }
                } else {
                    if self.is_open() {
                        self.state = GestureState::Idle;
                        (false, vec![GestureAction::CloseMenu])
                    } else {
                        (false, vec![])
                    }
                }
            }

            GestureEvent::MButtonUp => {
                if self.swallow_next_mbutton_up {
                    self.swallow_next_mbutton_up = false;
                    (true, vec![])
                } else {
                    (false, vec![])
                }
            }

            GestureEvent::CtrlUp => {
                match self.state {
                    GestureState::Open => {
                        self.state = GestureState::Committing;
                        // "Ctrl 뗌은 삼키지 않는다 (키 상태가 꼬이지 않게)"
                        (false, vec![GestureAction::RequestCommit])
                    }
                    _ => (false, vec![]),
                }
            }

            GestureEvent::LButtonDown { .. } => {
                match self.state {
                    GestureState::Open => {
                        self.state = GestureState::Committing;
                        (true, vec![GestureAction::RequestCommit])
                    }
                    _ => (false, vec![]),
                }
            }

            GestureEvent::RButtonDown => {
                if self.is_open() {
                    self.swallow_next_rbutton_up = true;
                    self.state = GestureState::Idle;
                    (true, vec![GestureAction::CloseMenu])
                } else {
                    (false, vec![])
                }
            }

            GestureEvent::RButtonUp => {
                if self.swallow_next_rbutton_up {
                    self.swallow_next_rbutton_up = false;
                    (true, vec![])
                } else {
                    (false, vec![])
                }
            }

            GestureEvent::EscapeDown => {
                if self.is_open() {
                    self.swallow_next_escape_up = true;
                    self.state = GestureState::Idle;
                    (true, vec![GestureAction::CloseMenu])
                } else {
                    (false, vec![])
                }
            }

            GestureEvent::EscapeUp => {
                if self.swallow_next_escape_up {
                    self.swallow_next_escape_up = false;
                    (true, vec![])
                } else {
                    (false, vec![])
                }
            }

            GestureEvent::OtherKeyDown => {
                if self.is_open() {
                    self.state = GestureState::Idle;
                    (false, vec![GestureAction::CloseMenu])
                } else {
                    (false, vec![])
                }
            }

            GestureEvent::OtherKeyUp => {
                (false, vec![])
            }

            GestureEvent::MouseWheel { .. } => {
                if self.is_open() {
                    (true, vec![])
                } else {
                    (false, vec![])
                }
            }

            GestureEvent::MouseMove { x, y } => {
                if self.state == GestureState::Open {
                    (false, vec![GestureAction::CursorMove { x, y }])
                } else {
                    (false, vec![])
                }
            }

            GestureEvent::CommitResponse(target) => {
                if self.state == GestureState::Committing || self.state == GestureState::Open {
                    self.state = GestureState::Idle;
                    match target {
                        Some(t) => (false, vec![GestureAction::ExecuteTarget(t), GestureAction::CloseMenu]),
                        None => (false, vec![GestureAction::CloseMenu]),
                    }
                } else {
                    (false, vec![])
                }
            }

            GestureEvent::Timeout => {
                if self.state == GestureState::Committing {
                    self.state = GestureState::Idle;
                    (false, vec![GestureAction::CloseMenu])
                } else {
                    (false, vec![])
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rbutton_closes_menu() {
        let mut sm = GestureStateMachine::new();
        sm.handle_event(GestureEvent::MButtonDown {
            ctrl_down: true,
            x: 100,
            y: 100,
        });
        assert_eq!(sm.state(), GestureState::Open);

        // Right-click while open closes menu and swallows click
        let (swallow, actions) = sm.handle_event(GestureEvent::RButtonDown);
        assert!(swallow);
        assert_eq!(actions, vec![GestureAction::CloseMenu]);
        assert_eq!(sm.state(), GestureState::Idle);

        // Subsequent right-button up is swallowed
        let (swallow_up, _) = sm.handle_event(GestureEvent::RButtonUp);
        assert!(swallow_up);
    }

    #[test]
    fn test_normal_open_and_ctrl_up_commit() {
        let mut sm = GestureStateMachine::new();
        assert_eq!(sm.state(), GestureState::Idle);

        // 1. Ctrl + 휠 누름
        let (swallow, actions) = sm.handle_event(GestureEvent::MButtonDown {
            ctrl_down: true,
            x: 500,
            y: 300,
        });
        assert!(swallow);
        assert_eq!(actions, vec![GestureAction::OpenMenu { x: 500, y: 300 }]);
        assert_eq!(sm.state(), GestureState::Open);

        // 2. 휠 뗌 (짝이 맞으므로 삼켜져야 함)
        let (swallow, actions) = sm.handle_event(GestureEvent::MButtonUp);
        assert!(swallow);
        assert!(actions.is_empty());
        assert_eq!(sm.state(), GestureState::Open);

        // 3. Ctrl 뗌 -> 커밋 요청 (키 상태 보존을 위해 삼키지 않음)
        let (swallow, actions) = sm.handle_event(GestureEvent::CtrlUp);
        assert!(!swallow);
        assert_eq!(actions, vec![GestureAction::RequestCommit]);
        assert_eq!(sm.state(), GestureState::Committing);

        // 4. 화면 응답
        let (swallow, actions) = sm.handle_event(GestureEvent::CommitResponse(Some("item-notepad".into())));
        assert!(!swallow);
        assert_eq!(
            actions,
            vec![
                GestureAction::ExecuteTarget("item-notepad".into()),
                GestureAction::CloseMenu
            ]
        );
        assert_eq!(sm.state(), GestureState::Idle);
    }

    #[test]
    fn test_wheel_click_without_ctrl_not_swallowed() {
        let mut sm = GestureStateMachine::new();

        // Ctrl 없이 휠클릭
        let (swallow, actions) = sm.handle_event(GestureEvent::MButtonDown {
            ctrl_down: false,
            x: 100,
            y: 100,
        });
        assert!(!swallow, "Ctrl 없는 휠 누름은 삼키지 않아야 함");
        assert!(actions.is_empty());

        let (swallow, actions) = sm.handle_event(GestureEvent::MButtonUp);
        assert!(!swallow, "짝이 되는 휠 뗌도 삼키지 않아야 함");
        assert!(actions.is_empty());
        assert_eq!(sm.state(), GestureState::Idle);
    }

    #[test]
    fn test_lbutton_click_while_open_requests_commit() {
        let mut sm = GestureStateMachine::new();

        sm.handle_event(GestureEvent::MButtonDown {
            ctrl_down: true,
            x: 200,
            y: 200,
        });
        assert_eq!(sm.state(), GestureState::Open);

        // 왼쪽 클릭 -> RequestCommit 및 삼킴
        let (swallow, actions) = sm.handle_event(GestureEvent::LButtonDown { x: 220, y: 220 });
        assert!(swallow);
        assert_eq!(actions, vec![GestureAction::RequestCommit]);
        assert_eq!(sm.state(), GestureState::Committing);
    }

    #[test]
    fn test_escape_cancels_and_swallows() {
        let mut sm = GestureStateMachine::new();

        sm.handle_event(GestureEvent::MButtonDown {
            ctrl_down: true,
            x: 200,
            y: 200,
        });
        assert_eq!(sm.state(), GestureState::Open);

        let (swallow, actions) = sm.handle_event(GestureEvent::EscapeDown);
        assert!(swallow);
        assert_eq!(actions, vec![GestureAction::CloseMenu]);
        assert_eq!(sm.state(), GestureState::Idle);
        
        let (swallow, _) = sm.handle_event(GestureEvent::EscapeUp);
        assert!(swallow);
    }

    #[test]
    fn test_other_key_cancels_and_does_not_swallow() {
        let mut sm = GestureStateMachine::new();

        sm.handle_event(GestureEvent::MButtonDown {
            ctrl_down: true,
            x: 200,
            y: 200,
        });
        assert_eq!(sm.state(), GestureState::Open);

        let (swallow, actions) = sm.handle_event(GestureEvent::OtherKeyDown);
        assert!(!swallow);
        assert_eq!(actions, vec![GestureAction::CloseMenu]);
        assert_eq!(sm.state(), GestureState::Idle);
        
        let (swallow, _) = sm.handle_event(GestureEvent::OtherKeyUp);
        assert!(!swallow);
    }

    #[test]
    fn test_wheel_scroll_swallowed_when_open() {
        let mut sm = GestureStateMachine::new();

        // 닫혀 있을 때의 휠 굴림 -> 삼키지 않음
        let (swallow, _) = sm.handle_event(GestureEvent::MouseWheel { delta: 120 });
        assert!(!swallow);

        // 열림
        sm.handle_event(GestureEvent::MButtonDown {
            ctrl_down: true,
            x: 300,
            y: 300,
        });

        // 열려 있을 때의 휠 굴림 -> 삼킴 (확대/축소 방지)
        let (swallow, actions) = sm.handle_event(GestureEvent::MouseWheel { delta: 120 });
        assert!(swallow);
        assert!(actions.is_empty());
    }

    #[test]
    fn test_toggle_close_on_second_ctrl_mbutton() {
        let mut sm = GestureStateMachine::new();

        sm.handle_event(GestureEvent::MButtonDown {
            ctrl_down: true,
            x: 100,
            y: 100,
        });
        assert_eq!(sm.state(), GestureState::Open);

        // 열린 상태에서 다시 Ctrl + 휠클릭 -> 닫기
        let (swallow, actions) = sm.handle_event(GestureEvent::MButtonDown {
            ctrl_down: true,
            x: 100,
            y: 100,
        });
        assert!(swallow);
        assert_eq!(actions, vec![GestureAction::CloseMenu]);
        assert_eq!(sm.state(), GestureState::Idle);
    }

    #[test]
    fn test_committing_timeout() {
        let mut sm = GestureStateMachine::new();

        sm.handle_event(GestureEvent::MButtonDown {
            ctrl_down: true,
            x: 100,
            y: 100,
        });
        sm.handle_event(GestureEvent::CtrlUp);
        assert_eq!(sm.state(), GestureState::Committing);

        // 타임아웃
        let (swallow, actions) = sm.handle_event(GestureEvent::Timeout);
        assert!(!swallow);
        assert_eq!(actions, vec![GestureAction::CloseMenu]);
        assert_eq!(sm.state(), GestureState::Idle);
    }
}
