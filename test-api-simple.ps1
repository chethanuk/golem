# Simple Golem REST API Test
Write-Host "Testing Golem REST API..." -ForegroundColor Cyan

$token = "5c832d93-ff85-4a8f-9803-513950fdfdb1"
$compUrl = "http://localhost:8082"
$workerUrl = "http://localhost:8085"

$headers = @{
    "Authorization" = "Bearer $token"
    "Accept" = "application/json"
}

# Test 1: List components
Write-Host "`nStep 1: Testing Component Service..." -ForegroundColor Yellow
try {
    $response = Invoke-WebRequest -Uri "$compUrl/v1/components" -Headers $headers -Method GET
    Write-Host "SUCCESS: Component Service accessible" -ForegroundColor Green
    Write-Host "Status: $($response.StatusCode)" -ForegroundColor White
    Write-Host "Response length: $($response.Content.Length) bytes" -ForegroundColor White
}
catch {
    Write-Host "FAILED: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# Test 2: Find a simple component to use
Write-Host "`nStep 2: Finding test component..." -ForegroundColor Yellow
$wasmFile = Get-ChildItem ".\test-components\*.wasm" | Select-Object -First 1
if ($wasmFile) {
    Write-Host "Found: $($wasmFile.Name)" -ForegroundColor Green
} else {
    Write-Host "No WASM files found" -ForegroundColor Red
    exit 1
}

# Test 3: Try to deploy component using curl instead
Write-Host "`nStep 3: Testing component deployment with curl..." -ForegroundColor Yellow
$compName = "test-comp-" + (Get-Date -Format "HHmmss")

# Create a simple batch file to use curl for multipart upload
$curlScript = @"
@echo off
curl -X POST "$compUrl/v1/components" ^
  -H "Authorization: Bearer $token" ^
  -F "name=$compName" ^
  -F "componentType=\"Durable\"" ^
  -F "component=@$($wasmFile.FullName)" ^
  --silent --show-error
"@

$curlScript | Out-File -FilePath "upload-component.bat" -Encoding ASCII

try {
    $result = cmd /c "upload-component.bat"
    Write-Host "Curl result: $result" -ForegroundColor White
    
    if ($result -match '"componentId"') {
        Write-Host "SUCCESS: Component uploaded" -ForegroundColor Green
        
        # Extract component ID from JSON response
        $json = $result | ConvertFrom-Json
        $componentId = $json.versionedComponentId.componentId
        Write-Host "Component ID: $componentId" -ForegroundColor Cyan
        
        # Test 4: Create worker
        Write-Host "`nStep 4: Creating worker..." -ForegroundColor Yellow
        $workerName = "test-worker-" + (Get-Date -Format "HHmmss")
        $workerJson = @{
            name = $workerName
            args = @()
            env = @{}
        } | ConvertTo-Json -Compress
        
        try {
            $workerResponse = Invoke-RestMethod -Uri "$workerUrl/v1/components/$componentId/workers" -Method POST -Headers $headers -Body $workerJson -ContentType "application/json"
            Write-Host "SUCCESS: Worker created" -ForegroundColor Green
            Write-Host "Worker: $($workerResponse.workerId.workerName)" -ForegroundColor Cyan
            
            # Test 5: Get worker status
            Write-Host "`nStep 5: Getting worker status..." -ForegroundColor Yellow
            try {
                $statusResponse = Invoke-RestMethod -Uri "$workerUrl/v1/components/$componentId/workers/$workerName" -Method GET -Headers $headers
                Write-Host "SUCCESS: Worker status retrieved" -ForegroundColor Green
                Write-Host "Status: $($statusResponse.status)" -ForegroundColor Cyan
                Write-Host "Created: $($statusResponse.createdAt)" -ForegroundColor Cyan
                
                Write-Host "`n==== FINAL RESULT ====" -ForegroundColor Green
                Write-Host "✅ All tests PASSED!" -ForegroundColor Green
                Write-Host "✅ Golem services are working correctly!" -ForegroundColor Green
                Write-Host "Component ID: $componentId" -ForegroundColor White
                Write-Host "Worker Name: $workerName" -ForegroundColor White
            }
            catch {
                Write-Host "Worker status failed: $($_.Exception.Message)" -ForegroundColor Yellow
            }
        }
        catch {
            Write-Host "Worker creation failed: $($_.Exception.Message)" -ForegroundColor Yellow
        }
    } else {
        Write-Host "Component upload may have failed - checking response..." -ForegroundColor Yellow
        Write-Host "Response: $result" -ForegroundColor Red
    }
}
catch {
    Write-Host "Curl execution failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Cleanup
Remove-Item "upload-component.bat" -ErrorAction SilentlyContinue
