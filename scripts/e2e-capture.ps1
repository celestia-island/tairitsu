#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Batch screenshot capture for Tairitsu E2E visual testing.
.DESCRIPTION
    Navigates to each registered page, takes screenshots, and saves them
    to target/e2e_screenshots/<timestamp>/. Requires dev server running
    on localhost:3000 (start with: just dev --daemon).
.PARAMETER BaseUrl
    Base URL of the dev server. Default: http://localhost:3000
.PARAMETER OutputDir
    Output directory for screenshots. Default: target/e2e_screenshots/<timestamp>
.PARAMETER Baseline
    If set, copies output to target/e2e_screenshots/baseline/ instead of timestamped dir.
.EXAMPLE
    .\scripts\e2e-capture.ps1
.EXAMPLE
    .\scripts\e2e-capture.ps1 -BaseUrl http://localhost:8080 -Baseline
#>

param(
    [string]$BaseUrl = "http://localhost:3000",
    [string]$OutputDir = "",
    [switch]$Baseline
)

$ErrorActionPreference = "Stop"

$Timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
if ($OutputDir -eq "") {
    $OutputDir = "target/e2e_screenshots/$Timestamp"
}
if ($Baseline) {
    $OutputDir = "target/e2e_screenshots/baseline"
}

New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null

$Pages = @(
    @{ url="/"; name="home"; category="pages" },
    @{ url="/components/layer1/button"; name="button"; category="layer1" },
    @{ url="/components/layer1/form"; name="form"; category="layer1" },
    @{ url="/components/layer1/search"; name="search"; category="layer1" },
    @{ url="/components/layer1/switch"; name="switch"; category="layer1" },
    @{ url="/components/layer1/feedback"; name="feedback"; category="layer1" },
    @{ url="/components/layer1/display"; name="display"; category="layer1" },
    @{ url="/components/layer1/avatar"; name="avatar"; category="layer1" },
    @{ url="/components/layer1/image"; name="image"; category="layer1" },
    @{ url="/components/layer1/tag"; name="tag"; category="layer1" },
    @{ url="/components/layer1/empty"; name="empty"; category="layer1" },
    @{ url="/components/layer1/comment"; name="comment"; category="layer1" },
    @{ url="/event-test"; name="event_test"; category="system" }
)

Write-Host "=== Tairitsu E2E Screenshot Capture ===" -ForegroundColor Cyan
Write-Host "Base URL : $BaseUrl"
Write-Host "Output   : $OutputDir"
Write-Host "Pages    : $($Pages.Count)"
Write-Host ""

$Pass = 0
$Fail = 0
$Results = @()

foreach ($P in $Pages) {
    $Url = "$BaseUrl$($P.url)"
    $OutFile = "$OutputDir/$($P.name).png"
    
    Write-Host "  [$($P.category)/$($P.name)] " -NoNewline
    
    try {
        # Use Playwright MCP or direct Playwright if available
        # For now, use a simple approach with Node.js + Playwright if installed
        $NodeScript = @"
const { chromium } = require('playwright');
(async () => {
    const browser = await chromium.launch();
    const page = await browser.newPage();
    await page.goto('$Url', { waitUntil: 'networkidle', timeout: 30000 });
    await page.screenshot({ path: '$OutFile', fullPage: false });
    
    // Check for console errors
    const errors = [];
    page.on('console', msg => {
        if (msg.type() === 'error' && !msg.text().includes('favicon')) {
            errors.push(msg.text());
        }
    });
    
    // Reload to capture console errors
    await page.goto('$Url', { waitUntil: 'networkidle', timeout: 30000 });
    await browser.close();
    
    // Output result as JSON
    console.log(JSON.stringify({ success: true, errors }));
})().catch(e => { console.log(JSON.stringify({ success: false, error: e.message })); });
"@
        
        $TempScript = Join-Path $env:TEMP "tairitsu-e2e-capture-$($P.name).mjs"
        $NodeScript | Out-File -FilePath $TempScript -Encoding utf8
        
        $Result = node $TempScript 2>&1
        Remove-Item $TempScript -Force -ErrorAction SilentlyContinue
        
        $Json = $Result | Where-Object { $_ -match '^\{' } | Select-Object -Last 1
        if ($Json) {
            $Data = $Json | ConvertFrom-Json
            if ($Data.success) {
                Write-Host "[OK]" -ForegroundColor Green
                $Pass++
                $Status = "PASS"
                if ($Data.errors -and $Data.errors.Count -gt 0) {
                    $Status = "WARN"
                    Write-Host "  ($($Data.errors.Count) console warnings)" -ForegroundColor Yellow
                }
            } else {
                Write-Host "[FAIL] $($Data.error)" -ForegroundColor Red
                $Fail++
                $Status = "FAIL"
            }
        } else {
            # Fallback: check if file was created anyway
            if (Test-Path $OutFile) {
                Write-Host "[OK]" -ForegroundColor Green
                $Pass++
                $Status = "PASS"
            } else {
                Write-Host "[FAIL] no output" -ForegroundColor Red
                $Fail++
                $Status = "FAIL"
            }
        }
    }
    catch {
        Write-Host "[ERROR] $($_.Exception.Message)" -ForegroundColor Red
        $Fail++
        $Status = "ERROR"
    }
    
    $Results += [PSCustomObject]@{
        Name = $P.name
        Category = $P.category
        Status = $Status
        Path = $OutFile
    }
}

Write-Host ""
Write-Host "=== Summary ===" -ForegroundColor Cyan
Write-Host "  Passed : $Pass" -ForegroundColor Green
Write-Host "  Failed : $Fail" -ForegroundColor $(if ($Fail -gt 0) { "Red" } else { "Green" })
Write-Host "  Total  : $($Pages.Count)"
Write-Host "  Output : $OutputDir"

if ($Fail -gt 0) {
    Write-Host ""
    Write-Host "Failed pages:" -ForegroundColor Red
    foreach ($R in $Results | Where-Object { $_.Status -ne "PASS" }) {
        Write-Host "  - [$($R.category)] $($R.Name) ($($R.Status))" -ForegroundColor Red
    }
    exit 1
}

$Results | ConvertTo-Json | Out-File "$OutputDir/results.json"
exit 0
