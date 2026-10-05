<#
.SYNOPSIS
    JSON Schema 및 TypeScript 타입 재생성 스크립트
#>

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$RepoRoot = Split-Path -Parent $ScriptDir

Write-Host "=== 스키마 및 타입 생성 중 ===" -ForegroundColor Cyan

# 1. rmctl을 통해 JSON Schema 생성
$SchemaPath = "$RepoRoot\schema\menu.schema.json"
$env:Path = [System.Environment]::GetEnvironmentVariable("Path","User") + ";" + [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";$env:USERPROFILE\.cargo\bin"

Set-Location $RepoRoot
cargo run -p rmctl --quiet -- schema > $SchemaPath
Write-Host "[OK] $SchemaPath 생성 완료" -ForegroundColor Green
