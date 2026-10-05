<#
.SYNOPSIS
    Radial Menu 제거 스크립트
#>

$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Error "이 스크립트는 관리자 권한으로 실행해야 합니다."
    exit 1
}

$InstallDir = "$env:ProgramFiles\RadialMenu"
$TaskName = "RadialMenu_AutoStart"

Write-Host "=== Radial Menu 제거 시작 ===" -ForegroundColor Yellow

# 1. 프로세스 종료
Get-Process -Name "RadialMenu", "rmctl" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

# 2. 작업 스케줄러 삭제
Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false -ErrorAction SilentlyContinue
Write-Host "[OK] 작업 스케줄러 삭제 완료" -ForegroundColor Green

# 3. PATH 제거
$MachinePath = [Environment]::GetEnvironmentVariable("Path", "Machine")
if ($MachinePath -like "*$InstallDir*") {
    $NewPath = ($MachinePath.Split(';') | Where-Object { $_ -ne $InstallDir -and $_ -ne "" }) -join ';'
    [Environment]::SetEnvironmentVariable("Path", $NewPath, "Machine")
    Write-Host "[OK] 시스템 PATH에서 제거 완료" -ForegroundColor Green
}

# 4. 파일 삭제
if (Test-Path $InstallDir) {
    Remove-Item -Path $InstallDir -Recurse -Force -ErrorAction SilentlyContinue
    Write-Host "[OK] Program Files 디렉터리 삭제 완료" -ForegroundColor Green
}

Write-Host "=== Radial Menu 제거가 완료되었습니다. ===" -ForegroundColor Yellow
