# NudgePredictor Integration Tests
# Run with: pwsh -File tests/powershell_integration_test.ps1

$ErrorActionPreference = "Stop"
$script:TestsPassed = 0
$script:TestsFailed = 0

function Write-TestResult {
    param(
        [string]$TestName,
        [bool]$Passed,
        [string]$Message = ""
    )

    if ($Passed) {
        Write-Host "[PASS] " -ForegroundColor Green -NoNewline
        Write-Host $TestName
        $script:TestsPassed++
    } else {
        Write-Host "[FAIL] " -ForegroundColor Red -NoNewline
        Write-Host "$TestName - $Message"
        $script:TestsFailed++
    }
}

function Test-PowerShellVersion {
    $passed = $PSVersionTable.PSVersion.Major -ge 7 -and
              ($PSVersionTable.PSVersion.Major -gt 7 -or $PSVersionTable.PSVersion.Minor -ge 2)

    Write-TestResult "PowerShell 7.2+ detected" $passed "Current: $($PSVersionTable.PSVersion)"
    return $passed
}

function Test-PSReadLineVersion {
    $psrl = Get-Module PSReadLine -ListAvailable | Select-Object -First 1
    $passed = $null -ne $psrl -and $psrl.Version -ge [Version]'2.2.0'

    $version = if ($psrl) { $psrl.Version } else { "Not found" }
    Write-TestResult "PSReadLine 2.2.0+ available" $passed "Current: $version"
    return $passed
}

function Test-ModuleManifest {
    $manifestPath = Join-Path $PSScriptRoot "..\shell\NudgePredictor\NudgePredictor.psd1"
    $passed = Test-Path $manifestPath

    if ($passed) {
        try {
            $manifest = Test-ModuleManifest -Path $manifestPath -ErrorAction Stop
            $passed = $null -ne $manifest
        } catch {
            $passed = $false
        }
    }

    Write-TestResult "Module manifest is valid" $passed
    return $passed
}

function Test-ModuleImport {
    $modulePath = Join-Path $PSScriptRoot "..\shell\NudgePredictor"

    try {
        Import-Module $modulePath -Force -ErrorAction Stop
        $module = Get-Module NudgePredictor
        $passed = $null -ne $module

        Write-TestResult "Module imports successfully" $passed
        return $passed
    } catch {
        Write-TestResult "Module imports successfully" $false $_.Exception.Message
        return $false
    }
}

function Test-PredictorRegistration {
    try {
        # Check if predictor is registered
        $predictors = [System.Management.Automation.Subsystem.SubsystemManager]::GetSubsystemInfo(
            [System.Management.Automation.Subsystem.SubsystemKind]::CommandPredictor
        )

        $nudgePredictor = $predictors | Where-Object { $_.Name -eq 'Nudge' }
        $passed = $null -ne $nudgePredictor

        Write-TestResult "Predictor registered with PSReadLine" $passed
        return $passed
    } catch {
        Write-TestResult "Predictor registered with PSReadLine" $false $_.Exception.Message
        return $false
    }
}

function Test-ExportedFunctions {
    $module = Get-Module NudgePredictor
    if (-not $module) {
        Write-TestResult "Exported functions available" $false "Module not loaded"
        return $false
    }

    $expectedFunctions = @(
        'Register-NudgePredictor',
        'Unregister-NudgePredictor',
        'Set-NudgePredictionOptions'
    )

    $exportedFunctions = $module.ExportedFunctions.Keys
    $allFound = $true

    foreach ($func in $expectedFunctions) {
        if ($func -notin $exportedFunctions) {
            $allFound = $false
            break
        }
    }

    Write-TestResult "Exported functions available" $allFound "Expected: $($expectedFunctions -join ', ')"
    return $allFound
}

function Test-NudgeBinaryAvailable {
    $nudge = Get-Command nudge -ErrorAction SilentlyContinue
    $passed = $null -ne $nudge

    Write-TestResult "Nudge binary available in PATH" $passed
    return $passed
}

function Test-UnregisterPredictor {
    try {
        Unregister-NudgePredictor -ErrorAction Stop

        $predictors = [System.Management.Automation.Subsystem.SubsystemManager]::GetSubsystemInfo(
            [System.Management.Automation.Subsystem.SubsystemKind]::CommandPredictor
        )

        $nudgePredictor = $predictors | Where-Object { $_.Name -eq 'Nudge' }
        $passed = $null -eq $nudgePredictor

        Write-TestResult "Predictor unregisters successfully" $passed
        return $passed
    } catch {
        Write-TestResult "Predictor unregisters successfully" $false $_.Exception.Message
        return $false
    }
}

function Test-ReregisterPredictor {
    try {
        Register-NudgePredictor -ErrorAction Stop

        $predictors = [System.Management.Automation.Subsystem.SubsystemManager]::GetSubsystemInfo(
            [System.Management.Automation.Subsystem.SubsystemKind]::CommandPredictor
        )

        $nudgePredictor = $predictors | Where-Object { $_.Name -eq 'Nudge' }
        $passed = $null -ne $nudgePredictor

        Write-TestResult "Predictor re-registers successfully" $passed
        return $passed
    } catch {
        Write-TestResult "Predictor re-registers successfully" $false $_.Exception.Message
        return $false
    }
}

# Main test execution
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  NudgePredictor Integration Tests" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Check prerequisites
$prereqsPassed = $true
$prereqsPassed = $prereqsPassed -and (Test-PowerShellVersion)
$prereqsPassed = $prereqsPassed -and (Test-PSReadLineVersion)

if (-not $prereqsPassed) {
    Write-Host ""
    Write-Host "Prerequisites not met. Some tests will be skipped." -ForegroundColor Yellow
    Write-Host ""
}

# Run tests
Write-Host ""
Write-Host "Module Tests:" -ForegroundColor Cyan
Write-Host "-------------"
Test-ModuleManifest | Out-Null

if ($prereqsPassed) {
    Test-ModuleImport | Out-Null
    Test-ExportedFunctions | Out-Null
    Test-PredictorRegistration | Out-Null
    Test-UnregisterPredictor | Out-Null
    Test-ReregisterPredictor | Out-Null
}

Write-Host ""
Write-Host "Environment Tests:" -ForegroundColor Cyan
Write-Host "------------------"
Test-NudgeBinaryAvailable | Out-Null

# Summary
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Test Summary" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Passed: $script:TestsPassed" -ForegroundColor Green
Write-Host "Failed: $script:TestsFailed" -ForegroundColor $(if ($script:TestsFailed -gt 0) { "Red" } else { "Green" })
Write-Host ""

if ($script:TestsFailed -gt 0) {
    exit 1
} else {
    exit 0
}
