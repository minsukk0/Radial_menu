#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Subagent Token Tracker
Calculates estimated token usage for Antigravity subagents from their transcript logs
and outputs a concise Markdown table.
"""

import sys
import os
import json
import glob
import argparse
from typing import Dict, List, Any, Optional

# Ensure UTF-8 output on Windows console
if sys.stdout.encoding != 'utf-8':
    try:
        sys.stdout.reconfigure(encoding='utf-8')
        sys.stderr.reconfigure(encoding='utf-8')
    except Exception:
        pass

def estimate_tokens(text: str) -> int:
    """
    Estimates token count with CJK and ASCII awareness:
    - CJK / Hangul characters: ~1.8 chars per token
    - ASCII / Code / Whitespace: ~3.8 chars per token
    """
    if not text:
        return 0
    cjk_count = 0
    other_count = 0
    for char in text:
        cp = ord(char)
        if (0xAC00 <= cp <= 0xD7A3) or (0x1100 <= cp <= 0x11FF) or (0x4E00 <= cp <= 0x9FFF):
            cjk_count += 1
        else:
            other_count += 1
    return int(round(cjk_count / 1.8 + other_count / 3.8))

def parse_transcript(transcript_path: str) -> Dict[str, Any]:
    """
    Parses a single transcript.jsonl file and aggregates token metrics.
    """
    stats = {
        "input_tokens": 0,
        "output_tokens": 0,
        "total_tokens": 0,
        "step_count": 0,
        "first_prompt": "",
        "status": "완료",
    }
    
    if not os.path.exists(transcript_path):
        stats["status"] = "로그 미발견"
        return stats

    try:
        with open(transcript_path, "r", encoding="utf-8", errors="ignore") as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                try:
                    step = json.loads(line)
                except Exception:
                    continue
                
                stats["step_count"] += 1
                step_type = step.get("type", "")
                content = step.get("content", "") or ""
                
                if step_type == "USER_INPUT":
                    stats["input_tokens"] += estimate_tokens(content)
                    if not stats["first_prompt"] and content:
                        stats["first_prompt"] = content.strip().split("\n")[0][:60]
                elif step_type == "PLANNER_RESPONSE":
                    stats["output_tokens"] += estimate_tokens(content)
                    thinking = step.get("thinking", "") or ""
                    if thinking:
                        stats["output_tokens"] += estimate_tokens(thinking)
                    tool_calls = step.get("tool_calls")
                    if tool_calls:
                        stats["output_tokens"] += estimate_tokens(json.dumps(tool_calls, ensure_ascii=False))
                elif step_type == "GENERIC":
                    stats["input_tokens"] += estimate_tokens(content)
                    
                if step_type == "PLANNER_RESPONSE":
                    if step.get("status") == "ERROR":
                        stats["status"] = "오류"
                    else:
                        stats["status"] = "완료"
    except Exception as e:
        stats["status"] = f"읽기 오류: {e}"

    stats["total_tokens"] = stats["input_tokens"] + stats["output_tokens"]
    return stats

def find_transcript_for_id(brain_dir: str, conv_id: str) -> Optional[str]:
    """Locates the transcript file for a given conversation ID."""
    # Direct folder check
    direct_path = os.path.join(brain_dir, conv_id, ".system_generated", "logs", "transcript.jsonl")
    if os.path.exists(direct_path):
        return direct_path
    
    # Pattern search
    matches = glob.glob(os.path.join(brain_dir, f"*{conv_id}*", ".system_generated", "logs", "transcript.jsonl"))
    if matches:
        return matches[0]
        
    return None

def main():
    parser = argparse.ArgumentParser(description="Calculate subagent token usage and output markdown table.")
    parser.add_argument("conv_ids", nargs="*", help="Conversation IDs or transcript file paths")
    parser.add_argument("--meta", help="Optional JSON string mapping conv_id to {'role': '...', 'model': '...', 'task': '...'}")
    parser.add_argument("--json", action="store_true", help="Output raw JSON instead of Markdown")
    args = parser.parse_args()

    app_data_dir = os.environ.get("USERPROFILE", "")
    brain_dir = os.path.join(app_data_dir, ".gemini", "antigravity", "brain")

    metadata = {}
    if args.meta:
        try:
            if os.path.isfile(args.meta):
                with open(args.meta, "r", encoding="utf-8") as f:
                    metadata = json.load(f)
            else:
                metadata = json.loads(args.meta)
        except Exception:
            pass

    records = []
    
    for item in args.conv_ids:
        role = metadata.get(item, {}).get("role", "서브에이전트")
        model = metadata.get(item, {}).get("model", "기본")
        task_desc = metadata.get(item, {}).get("task", "")

        if os.path.isfile(item):
            transcript_path = item
            conv_id = os.path.basename(os.path.dirname(os.path.dirname(os.path.dirname(item))))
        else:
            conv_id = item
            transcript_path = find_transcript_for_id(brain_dir, conv_id)

        if transcript_path:
            stats = parse_transcript(transcript_path)
        else:
            stats = {
                "input_tokens": 0,
                "output_tokens": 0,
                "total_tokens": 0,
                "step_count": 0,
                "first_prompt": "",
                "status": "대화 ID 미발견"
            }

        task = task_desc or stats.get("first_prompt") or "-"
        if len(task) > 40:
            task = task[:37] + "..."

        records.append({
            "id": conv_id[:8] if conv_id else "-",
            "role": role,
            "model": model,
            "input_tokens": stats["input_tokens"],
            "output_tokens": stats["output_tokens"],
            "total_tokens": stats["total_tokens"],
            "status": stats["status"],
            "task": task
        })

    if args.json:
        print(json.dumps(records, ensure_ascii=False, indent=2))
        return

    # Render Markdown Table
    total_inp = sum(r["input_tokens"] for r in records)
    total_out = sum(r["output_tokens"] for r in records)
    total_all = sum(r["total_tokens"] for r in records)

    print("### 📊 서브에이전트 토큰 사용량 요약")
    print("| 서브에이전트 (역할) | 모델 | 입력 토큰 | 출력 토큰 | 합계 토큰 | 상태 | 작업 내용 |")
    print("|---|---|---|---|---|---|---|")
    for r in records:
        print(f"| {r['role']} | {r['model']} | {r['input_tokens']:,} | {r['output_tokens']:,} | {r['total_tokens']:,} | {r['status']} | {r['task']} |")
    print(f"| **전체 합계** | - | **{total_inp:,}** | **{total_out:,}** | **{total_all:,}** | - | **완료** |")

if __name__ == "__main__":
    main()
