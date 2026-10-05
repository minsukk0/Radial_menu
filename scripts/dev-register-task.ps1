<#
.SYNOPSIS
    개발 중인 Radial Menu 바이너리를 관리자 권한 작업 스케줄러로 등록
#>

$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Error "이 스크립트는 관리자 권한으로 실행해야 합니다."
    exit 1
}

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$RepoRoot = Split-Path -Parent $ScriptDir
$DevExe = "$RepoRoot\src-tauri\target\debug\RadialMenu.exe"
if (-not (Test-Path $DevExe)) {
    $DevExe = "$RepoRoot\target\debug\RadialMenu.exe"
}

$TaskName = "RadialMenu_DevTask"
$Action = New-ScheduledTaskAction -Execute $DevExe -WorkingDirectory $RepoRoot
$Trigger = New-ScheduledTaskTrigger -AtLogOn
$Principal = New-ScheduledTaskPrincipal -UserId $env:USERNAME -RunLevel Highest
$Settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -ExecutionTimeLimit 0

Register-ScheduledTask -TaskName $TaskName -Action $Action -Trigger $Trigger -Principal $Principal -Settings $Settings -Force | Out-Null
Write-Host "[OK] 개발용 작업 등록 완료: $TaskName ($DevExe)" -ForegroundColor Green
