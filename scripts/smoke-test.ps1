# KnowteQuiz Smoke Test
# Verifies core functionality before release.
# Usage: .\scripts\smoke-test.ps1
#
# Prerequisites:
#   - `npm run build` completed (frontend built to dist/)
#   - Rust release binary built (npm run tauri:build) OR use --web flag for web-only
#   - Default LLM endpoint (Ollama localhost:11434) or set KQ_LLM_URL

param(
    [switch]$Web = $false  # Run web-only smoke test (no Tauri binary needed)
)

$ErrorActionPreference = 'Stop'
$baseUrl = 'http://127.0.0.1:14200'

Write-Host "=== KnowteQuiz Smoke Test ===" -ForegroundColor Cyan

# 1. Start server
Write-Host "[1/9] Starting web server..." -ForegroundColor Yellow
$server = Start-Process -FilePath "cargo" -ArgumentList "run -- --mode=web" -WorkingDirectory (Join-Path $PSScriptRoot ".." "src-tauri") -PassThru -NoNewWindow

try {
    # Wait for server
    $maxWait = 60
    for ($i = 0; $i -lt $maxWait; $i++) {
        try {
            $null = Invoke-WebRequest -Uri "$baseUrl/api/data/status" -TimeoutSec 2 -ErrorAction Stop
            Write-Host "  Server ready after $i seconds" -ForegroundColor Green
            break
        } catch {
            if ($i -eq $maxWait - 1) { throw "Server did not start within $maxWait seconds" }
            Start-Sleep -Seconds 1
        }
    }

    # 2. Load settings
    Write-Host "[2/9] Loading settings..." -ForegroundColor Yellow
    $settings = Invoke-RestMethod -Uri "$baseUrl/api/settings"
    Write-Host "  OK - settings loaded (llm model: $($settings.llm.model))" -ForegroundColor Green

    # 3. Scan fixture notes
    Write-Host "[3/9] Scanning fixture notes..." -ForegroundColor Yellow
    $fixturePath = (Resolve-Path (Join-Path $PSScriptRoot ".." "fixture-notes")).Path
    $notes = Invoke-RestMethod -Uri "$baseUrl/api/notes/scan?root_path=$([uri]::EscapeDataString($fixturePath))"
    $mdFiles = ($notes | Where-Object { -not $_.is_dir }).Count
    Write-Host "  OK - found $mdFiles markdown files" -ForegroundColor Green

    # 4. Read a Markdown note
    Write-Host "[4/9] Reading a Markdown note..." -ForegroundColor Yellow
    $firstNote = ($notes | Where-Object { -not $_.is_dir } | Select-Object -First 1).path
    if (-not $firstNote) { throw "No markdown files found in fixture-notes" }
    $content = Invoke-RestMethod -Uri "$baseUrl/api/notes/read?path=$([uri]::EscapeDataString($firstNote))"
    Write-Host "  OK - read note: $($content.title) ($($content.content.Length) chars)" -ForegroundColor Green

    # 5. Verify data status (backup system)
    Write-Host "[5/9] Checking data file status..." -ForegroundColor Yellow
    $status = Invoke-RestMethod -Uri "$baseUrl/api/data/status"
    $existingFiles = ($status.files | Where-Object { $_.exists }).Count
    Write-Host "  OK - $existingFiles managed files exist" -ForegroundColor Green

    # 6. Save a mistake
    Write-Host "[6/9] Saving a test mistake..." -ForegroundColor Yellow
    $mistakeId = [guid]::NewGuid().ToString()
    $mistake = @{
        id = $mistakeId
        note_path = $firstNote
        note_title = "Smoke Test Note"
        question = "Smoke test question?"
        user_answer = "A"
        correct_answer = "B"
        explanation = "This is a smoke test."
        mode = "basic"
        created_at = (Get-Date -Format "yyyy-MM-ddTHH:mm:ss.fffZ")
        review_count = 0
    } | ConvertTo-Json
    Invoke-RestMethod -Uri "$baseUrl/api/mistakes" -Method Post -Body $mistake -ContentType "application/json" | Out-Null
    Write-Host "  OK - mistake saved" -ForegroundColor Green

    # 7. Verify mistake persists
    Write-Host "[7/9] Verifying mistake persistence..." -ForegroundColor Yellow
    $loaded = Invoke-RestMethod -Uri "$baseUrl/api/mistakes"
    $found = $loaded | Where-Object { $_.id -eq $mistakeId }
    if (-not $found) { throw "Saved mistake not found in loaded list" }
    Write-Host "  OK - mistake persisted ($($loaded.Count) total)" -ForegroundColor Green

    # 8. Settings persistence (write + read-back)
    Write-Host "[8/9] Testing settings write/read-back..." -ForegroundColor Yellow
    $origSettings = Invoke-RestMethod -Uri "$baseUrl/api/settings"
    $origLang = $origSettings.quiz.language
    $origSettings.quiz.language = 'ja'
    $body = $origSettings | ConvertTo-Json -Depth 10
    Invoke-RestMethod -Uri "$baseUrl/api/settings" -Method Post -Body $body -ContentType "application/json" | Out-Null
    $readBack = Invoke-RestMethod -Uri "$baseUrl/api/settings"
    if ($readBack.quiz.language -ne 'ja') { throw "Settings write/read-back failed: expected ja, got $($readBack.quiz.language)" }
    # Restore original
    $origSettings.quiz.language = $origLang
    $body = $origSettings | ConvertTo-Json -Depth 10
    Invoke-RestMethod -Uri "$baseUrl/api/settings" -Method Post -Body $body -ContentType "application/json" | Out-Null
    Write-Host "  OK - settings persisted and restored" -ForegroundColor Green

    # 9. Mark a mistake as reviewed
    Write-Host "[9/9] Testing mistake review flow..." -ForegroundColor Yellow
    Invoke-RestMethod -Uri "$baseUrl/api/mistakes/review" -Method Post -Body "{`"id`":`"$mistakeId`"}" -ContentType "application/json" | Out-Null
    $reviewed = Invoke-RestMethod -Uri "$baseUrl/api/mistakes"
    $reviewedEntry = $reviewed | Where-Object { $_.id -eq $mistakeId }
    if ($reviewedEntry.review_count -lt 1) { throw "Mistake review failed: review_count is $($reviewedEntry.review_count)" }
    Write-Host "  OK - mistake marked as reviewed (count: $($reviewedEntry.review_count))" -ForegroundColor Green

    Write-Host ""
    Write-Host "=== ALL SMOKE TESTS PASSED ===" -ForegroundColor Green

} finally {
    Write-Host "Stopping server..." -ForegroundColor Yellow
    Stop-Process -Id $server.Id -Force -ErrorAction SilentlyContinue
}
