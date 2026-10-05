---
name: implementation
description: >-
  규모가 어느 정도 있는 기능 구현 및 복잡한 시스템 개발 시 메인 에이전트가 오케스트레이터(Orchestrator)로서
  작업을 총괄하고 서브에이전트를 조율하는 스킬. 일반 구현 및 단위 작업에는 gemini-3.8-flash,
  코드 감사(Audit) 및 고난도 미해결 문제에는 opus-5.5(pro) 서브에이전트를 생성(spawn)하여 위임하며,
  작업 완료 후 각 서브에이전트의 토큰 사용량을 매우 간결한 마크다운 표로 집계하여 출력합니다.
---

# Implementation Orchestration Skill

규모가 있는 기능 구현, 다중 모듈 개발, 대규모 리팩토링 시 사용하는 오케스트레이션 가이드입니다.
메인 에이전트는 직접 모든 코드를 작성하지 않고 **오케스트레이터(Orchestrator)** 역할을 수행하며, 작업 특성에 맞추어 서브에이전트를 적절한 모델 티어로 생성(spawn)하여 병렬/단계별로 실행합니다.

---

## 1. 역할 및 모델 배정 원칙 (Agent Roles & Model Tiering)

| 역할 (Role) | 담당 모델 | 적용 작업 범위 |
| :--- | :--- | :--- |
| **메인 에이전트 (Orchestrator)** | *호스트 기본 모델* | • 전체 아키텍처 및 작업 분해 (Decomposition)<br>• 서브에이전트 생성, 태스크 위임, 진행 관리<br>• 결과 통합(Integration) 및 회귀 테스트<br>• 서브에이전트 토큰 사용량 집계 및 완료 리포트 출력 |
| **일반 구현 서브에이전트** | **Gemini 3.8 Flash** (`flash`) | • 개별 모듈 및 컴포넌트 코드 구현<br>• 보일러플레이트, 스캐폴딩 생성<br>• 단위 테스트 코드 작성 및 기본 실행<br>• 파일 탐색, 레퍼런스 조사, 문서화 |
| **감사 및 난제 해결 서브에이전트** | **Opus 5.5 / Pro Tier** (`pro`) | • **Audit (감사)**: 보안 취약점, 동시성/레이스 컨디션, 엣지 케이스, 설계 무결성 심층 검토<br>• **미해결 문제 (Hard/Stuck Problems)**: Flash 모델이 여러 차례 시도해도 해결하지 못한 복잡한 버그, 알고리즘 병목, 심층 추론이 요구되는 문제 해결 |

---

## 2. 작업 단계별 실행 절차 (Workflow)

### 단계 1: 작업 분석 및 분해 (Planning & Task Decomposition)
1. 사용자의 요구사항과 코드베이스 구조를 파악합니다.
2. 작업을 독립적으로 병렬 처리할 수 있는 단위(Unit) 또는 순차 의존성이 있는 단계로 분해합니다.
3. 각 태스크의 성격에 따라 모델(`flash` vs `pro`)과 작업 범위를 결정합니다.

### 단계 2: 서브에이전트 생성 및 위임 (Spawning Subagents)
`invoke_subagent` 도구를 사용하여 서브에이전트를 호출합니다.

#### A. 일반 구현 작업 (`flash`)
```json
{
  "Subagents": [
    {
      "TypeName": "self",
      "Role": "Feature A Implementer",
      "Model": "flash",
      "Prompt": "D:/workspace/... 경로에서 모듈 A의 인터페이스와 핵심 로직을 구현하고 단위 테스트를 통과시키세요. 구체적 요구사항: ..."
    }
  ]
}
```

#### B. 코드 감사(Audit) 및 고난도 문제 해결 (`pro` - Opus 5.5)
```json
{
  "Subagents": [
    {
      "TypeName": "self",
      "Role": "Architecture & Security Auditor",
      "Model": "pro",
      "Prompt": "방금 구현된 모듈 A와 B의 보안 취약점, 레이스 컨디션, 예외 처리 누락 여부를 심층 감사(Audit)하고 개선 권고안 또는 직접 수정본을 보고하세요."
    }
  ]
}
```

> [!IMPORTANT]
> **리액티브 대기 원칙**:
> 서브에이전트를 실행한 후에는 `manage_task(Action='status')` 등으로 루프를 돌며 폴링하지 마십시오.
> 도구 호출을 종료하고 대기하면, 서브에이전트 작업 완료 시 시스템 메시지를 통해 자동으로 깨어납니다 (Reactive Wakeup).

### 단계 3: 통합 및 검증 (Integration & Verification)
1. 서브에이전트들의 작업 결과를 확인하고 필요한 경우 통합 충돌을 조정합니다.
2. 중요 모듈은 `pro` (Opus 5.5) 서브에이전트의 감사 결과가 통과되었는지 확인합니다.
3. 빌드 또는 전체 테스트 스위트를 실행하여 회귀(regression)가 없음을 검증합니다.

### 단계 4: 토큰 사용량 간결 집계 (Token Usage Report)
모든 작업이 완료되면, 실행된 각 서브에이전트의 토큰 사용량을 계산하여 **매우 간결한 마크다운 표**로 출력합니다.

#### 스크립트를 통한 집계 실행:
제공된 `scripts/token_tracker.py`를 실행하여 자동으로 집계할 수 있습니다:
```powershell
python ".agents/skills/implementation/scripts/token_tracker.py" <conversation_id_1> <conversation_id_2> ... --meta '{"<id_1>": {"role": "Worker A", "model": "gemini-3.8-flash", "task": "모듈 A 구현"}, "<id_2>": {"role": "Auditor", "model": "opus-5.5 (pro)", "task": "보안 감사"}}'
```

#### 최종 출력 표 예시:
```markdown
### 📊 서브에이전트 토큰 사용량 요약
| 서브에이전트 (역할) | 모델 | 입력 토큰 | 출력 토큰 | 합계 토큰 | 상태 | 작업 내용 |
|---|---|---|---|---|---|---|
| Feature Implementer | gemini-3.8-flash | 14,200 | 3,100 | 17,300 | 완료 | 데이터 파서 및 모델 정의 |
| Code Auditor | opus-5.5 (pro) | 18,500 | 4,200 | 22,700 | 완료 | 보안 취약점 및 동시성 엣지케이스 검증 |
| **전체 합계** | - | **32,700** | **7,300** | **40,000** | - | **성공** |
```

---

## 3. 핵심 규칙 체크리스트 (Best Practices)
1. **오케스트레이터의 집중**: 메인 에이전트는 서브에이전트의 산출물을 감독하고 인터페이스를 정합하는 데 집중하며, 장시간 코딩 작업은 반드시 서브에이전트에 분산합니다.
2. **비용 및 성능 최적화**: 표준 작업에는 속도와 비용 효율이 우수한 `gemini-3.8-flash`를 적극 활용하고, 고난도 추론 및 최종 검수(Audit)에 한해 `opus-5.5`(`pro`)를 투입합니다.
3. **간결한 마무리**: 구현 완료 후 군더더기 없이 최종 토큰 사용량 요약 표를 깔끔하게 제시합니다.
