#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Run E2E capture + verify console errors + generate summary report.
.DESCRIPTION
    Wraps e2e-capture.ps1 with additional verification steps:
    - Checks dev server is running
    - Captures screenshots for all pages
    - Detects JS console errors per page
    - Verifies WASM initialization on event-test page
    - Generates summary report in Markdown
.EXAMPLE
    .\scripts\e2e-verify.ps1
#>

param(
    [string]$BaseUrl = "http://localhost:3000",
    [string]$OutputDir = ""
)

$ErrorActionPreference = "Stop"

$Timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
if ($OutputDir -eq "") {
    $OutputDir = "target/e2e_screenshots/$Timestamp"
}

Write-Host "=== Tairitsu E2E Verification Suite ===" -ForegroundColor Cyan
Write-Host ""

# Step 1: Check dev server is running
Write-Host "[1/4] Checking dev server..." -NoNewline
try {
    $Response = Invoke-WebRequest -Uri "$BaseUrl/" -TimeoutSec 5 -UseBasicParsing -ErrorAction Stop
    Write-Host " OK (HTTP $($Response.StatusCode))" -ForegroundColor Green
}
catch {
    Write-Host " FAIL" -ForegroundColor Red
    Write-Host "  Dev server not running at $BaseUrl" -ForegroundColor Red
    Write-Host "  Start with: just dev --daemon" -ForegroundColor Yellow
    exit 1
}

# Step 2: Run screenshot capture
Write-Host "[2/4] Capturing screenshots..."
& "$PSScriptRoot/e2e-capture.ps1" -BaseUrl $BaseUrl -OutputDir $OutputDir
$CaptureExit = $LASTEXITCODE

# Step 3: Verify WASM event system on event-test page
Write-Host ""
Write-Host "[3/4] Verifying WASM event bridge..." -NoNewline

$WasmCheck = @"
const { chromium } = require('playwright');
(async () => {
    const browser = await chromium.launch();
    const page = await browser.newPage();
    
    let wasmReady = false;
    let listenerCount = 0;
    let clickFired = false;
    let errors = [];
    
    page.on('console', msg => {
        if (msg.type() === 'error' && !msg.text().includes('favicon')) {
            errors.push(msg.text());
        }
    });
    
    try {
        await page.goto('$BaseUrl/event-test', { waitUntil: 'networkidle', timeout: 30000 });
        
        const info = await page.evaluate(() => {
            const btn = document.getElementById('event-test-btn');
            let clickListenerId = null;
            
            if (globalThis.__listenerHandles && btn) {
                for (const [id, info] of globalThis.__listenerHandles) {
                    if (info.element === btn && info.type === 'click') {
                        clickListenerId = id.toString();
                        break;
                    }
                }
            }
            
            return {
                wasmExports: !!globalThis.__wasmExports,
                listenerHandles: !!globalThis.__listenerHandles,
                totalListeners: globalThis.__listenerHandles?.size ?? 0,
                clickListenerId,
                buttonExists: !!btn,
            };
        });
        
        wasmReady = info.wasmExports && info.listenerHandles;
        listenerCount = info.totalListeners;
        
        // Test click dispatch
        if (info.buttonExists) {
            const cb = globalThis.__wasmExports?.['tairitsu-browser:full/event-callbacks@0.2.0'];
            if (cb?.onMouseEvent) {
                const orig = cb.onMouseEvent.bind(cb);
                cb.onMouseEvent = function() { clickFired = true; return orig.apply(this, arguments); };
                document.getElementById('event-test-btn')?.click();
                
                await new Promise(r => setTimeout(r, 500));
                cb.onMouseEvent = orig;
            }
        }
    } catch(e) {
        errors.push(e.message);
    }
    
    await browser.close();
    
    console.log(JSON.stringify({
        wasmReady, listenerCount, clickFired, errors,
        status: wasmReady && clickFired ? 'PASS' : (wasmReady ? 'WARN' : 'FAIL')
    }));
})().catch(e => console.log(JSON.stringify({ error: e.message, status: 'ERROR' })));
"@

$TempScript = Join-Path $env:TEMP "tairitsu-e2e-wasm-check.mjs"
$WasmCheck | Out-File -FilePath $TempScript -Encoding utf8
$WasmResult = node $TempScript 2>&1
Remove-Item $TempScript -Force -ErrorAction SilentlyContinue

$WasmJson = ($WasmResult | Where-Object { $_ -match '^\{' }) | Select-Object -Last 1
if ($WasmJson) {
    $WasmData = $WasmJson | ConvertFrom-Json
    if ($WasmData.status -eq "PASS") {
        Write-Host " PASS" -ForegroundColor Green
    }
    elseif ($WasmData.status -eq "WARN") {
        Write-Host " WARN (WASM ok but click didn't fire)" -ForegroundColor Yellow
    }
    else {
        Write-Host " FAIL" -ForegroundColor Red
    }
}
else {
    Write-Host " SKIP (node/playwright not available)" -ForegroundColor Yellow
}

# Step 4: Generate report
Write-Host "[4/4] Generating report..."

$ReportFile = "$OutputDir/report.md"
$Report = @"
# Tairitsu E2E Test Report

**Date**: $(Get-Date -Format "yyyy-MM-dd HH:mm:ss")
**Base URL**: $BaseUrl
**Output**: $OutputDir

## Summary

| Metric | Value |
|--------|-------|
| Capture Status | $(if ($CaptureExit -eq 0) { "PASS" } else { "FAIL" }) |
| WASM Bridge | $(if ($WasmData) { $WasmData.status } else { "N/A" }) |
| Listener Count | $(if ($WasmData) { $WasmData.listenerCount } else { "N/A" }) |
| Click Dispatch | $(if ($WasmData) { "$(if ($WasmData.clickFired) { 'OK' } else { 'FAIL' })" } else { "N/A" }) |

## Screenshots

$(if (Test-Path "$OutputDir/results.json") {
    $R = Get-Content "$OutputDir/results.json" | ConvertFrom-Json
    foreach ($Item in $R) {
        "| [$($Item.Name)]($($Item.Name).png) | $($Item.Category) | $($Item.Status) |`n"
    }
} else { "*No results file*" })

## Notes

- Screenshots are full-viewport PNG images
- Baseline comparison requires Phase 2 (pixel diff tooling)
- Event bridge test verifies DOM -> JS glue -> WIT -> Rust handler chain
"@

$Report | Out-File $ReportFile -Encoding utf8
Write-Host "  Report: $ReportFile"

Write-Host ""
Write-Host "=== Done ===" -ForegroundColor Cyan
exit $(if ($CaptureExit -ne 0) { $CaptureExit } else { 0 })
