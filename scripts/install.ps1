<#
.SYNOPSIS
    Radial Menu 관리자 권한 자동 시작 및 Program Files 설치 스크립트
.DESCRIPTION
    1. C:\Program Files\RadialMenu 디렉터리에 실행 파일 및 리소스 복사
    2. 시스템 환경 변수 PATH에 설치 경로 추가 (rmctl CLI 전역 실행 지원)
    3. C:\ProgramData\RadialMenu 디렉터리 생성 및 관리자 전용 쓰기 ACL 설정
    4. 윈도우 작업 스케줄러에 최고 권한(RunLevel=Highest) 로그온 자동 시작 등록
#>

# 관리자 권한 확인
$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Error "이 스크립트는 관리자 권한으로 실행해야 합니다. PowerShell을 관리자 권한으로 열고 다시 실행해 주세요."
    exit 1
}

$InstallDir = "$env:ProgramFiles\RadialMenu"
$DataDir = "$env:ProgramData\RadialMenu"
$TaskName = "RadialMenu_AutoStart"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$RepoRoot = Split-Path -Parent $ScriptDir

Write-Host "=== Radial Menu 설치 시작 ===" -ForegroundColor Cyan

# 1. 기존 프로세스 종료
Get-Process -Name "RadialMenu", "rmctl" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

# 2. 설치 디렉터리 생성
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

# 3. 바이너리 복사 (release 빌드 우선)
$ExeSource = "$RepoRoot\src-tauri\target\release\RadialMenu.exe"
if (-not (Test-Path $ExeSource)) {
    $ExeSource = "$RepoRoot\target\release\RadialMenu.exe"
}
$RmctlSource = "$RepoRoot\target\release\rmctl.exe"
if (-not (Test-Path $RmctlSource)) {
    $RmctlSource = "$RepoRoot\crates\rmctl\target\release\rmctl.exe"
}

if (Test-Path $ExeSource) {
    Copy-Item -Path $ExeSource -Destination "$InstallDir\RadialMenu.exe" -Force
    Write-Host "[OK] RadialMenu.exe 복사 완료" -ForegroundColor Green
} else {
    Write-Warning "빌드된 RadialMenu.exe를 찾을 수 없습니다. 설치 후 바이너리를 수동으로 배치하거나 cargo build --release를 실행하세요."
}

if (Test-Path $RmctlSource) {
    Copy-Item -Path $RmctlSource -Destination "$InstallDir\rmctl.exe" -Force
    Write-Host "[OK] rmctl.exe 복사 완료" -ForegroundColor Green
}

# 스키마 파일 복사
if (Test-Path "$RepoRoot\schema\menu.schema.json") {
    $SchemaDir = "$InstallDir\schema"
    if (-not (Test-Path $SchemaDir)) { New-Item -ItemType Directory -Path $SchemaDir -Force | Out-Null }
    Copy-Item -Path "$RepoRoot\schema\menu.schema.json" -Destination "$SchemaDir\menu.schema.json" -Force
}

# 4. ProgramData 보안 디렉터리 설정 (관리자만 쓰기 허용)
if (-not (Test-Path $DataDir)) {
    New-Item -ItemType Directory -Path $DataDir -Force | Out-Null
}
$Acl = Get-Acl $DataDir
$Acl.SetAccessRuleProtection($true, $false)
$AdminRule = New-Object System.Security.AccessControl.FileSystemAccessRule("Administrators", "FullControl", "ContainerInherit,ObjectInherit", "None", "Allow")
$SystemRule = New-Object System.Security.AccessControl.FileSystemAccessRule("SYSTEM", "FullControl", "ContainerInherit,ObjectInherit", "None", "Allow")
$UsersRule = New-Object System.Security.AccessControl.FileSystemAccessRule("Users", "ReadAndExecute", "ContainerInherit,ObjectInherit", "None", "Allow")
$Acl.AddAccessRule($AdminRule)
$Acl.AddAccessRule($SystemRule)
$Acl.AddAccessRule($UsersRule)
Set-Acl -Path $DataDir -AclObject $Acl
Write-Host "[OK] ProgramData ACL 보안 설정 완료 (관리자만 쓰기 허용)" -ForegroundColor Green

# 5. 시스템 PATH 등록
$MachinePath = [Environment]::GetEnvironmentVariable("Path", "Machine")
if ($MachinePath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$MachinePath;$InstallDir", "Machine")
    Write-Host "[OK] 시스템 PATH에 rmctl 등록 완료" -ForegroundColor Green
}

# 6. 작업 스케줄러 등록 (사용자 로그온 시 최고 권한으로 상주)
$Action = New-ScheduledTaskAction -Execute "$InstallDir\RadialMenu.exe"
$Trigger = New-ScheduledTaskTrigger -AtLogOn
$Principal = New-ScheduledTaskPrincipal -UserId $env:USERNAME -RunLevel Highest
$Settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -ExecutionTimeLimit 0
Register-ScheduledTask -TaskName $TaskName -Action $Action -Trigger $Trigger -Principal $Principal -Settings $Settings -Force | Out-Null
Write-Host "[OK] 작업 스케줄러 최고 권한(Highest) 등록 완료: $TaskName" -ForegroundColor Green

Write-Host "=== Radial Menu 설치가 완료되었습니다. ===" -ForegroundColor Cyan
