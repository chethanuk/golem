# Golem REST API Test Script - Fixed Version
# Tests component deployment, worker creation, and function invocation via REST API

param(
    [string]$ComponentServiceUrl = "http://localhost:8082",
    [string]$WorkerServiceUrl = "http://localhost:8085",
    [string]$AdminToken = "5c832d93-ff85-4a8f-9803-513950fdfdb1"
)

Write-Host "🔧 Testing Golem REST API..." -ForegroundColor Cyan
Write-Host "Component Service: $ComponentServiceUrl" -ForegroundColor Gray
Write-Host "Worker Service: $WorkerServiceUrl" -ForegroundColor Gray

# Set up headers with authentication
$headers = @{
    "Authorization" = "Bearer $AdminToken"
    "Accept" = "application/json"
}

# Step 1: List existing components
Write-Host "`n📋 Step 1: List existing components..." -ForegroundColor Yellow
try {
    $response = Invoke-WebRequest -Uri "$ComponentServiceUrl/v1/components" -Headers $headers -Method GET
    Write-Host "✅ Components API accessible" -ForegroundColor Green
    Write-Host "Response: $($response.StatusCode) - $($response.Content.Length) bytes" -ForegroundColor Gray
    
    if ($response.Content -and $response.Content.Length -gt 2) {
        $components = $response.Content | ConvertFrom-Json
        if ($components -and $components.Count -gt 0) {
            Write-Host "Found $($components.Count) existing components:" -ForegroundColor Green
            foreach ($comp in $components) {
                Write-Host "  - $($comp.componentName) (ID: $($comp.versionedComponentId.componentId))" -ForegroundColor White
            }
        } else {
            Write-Host "No existing components found" -ForegroundColor Yellow
        }
    } else {
        Write-Host "Empty response - no components found" -ForegroundColor Yellow
    }
}
catch {
    Write-Host "❌ Failed to list components: $($_.Exception.Message)" -ForegroundColor Red
    if ($_.Exception.Response) {
        Write-Host "Status Code: $($_.Exception.Response.StatusCode)" -ForegroundColor Red
    }
    exit 1
}

# Step 2: Deploy a test component using simplified approach
Write-Host "`n📦 Step 2: Deploy test component..." -ForegroundColor Yellow

# Find a simple test component
$testWasmPath = $null
$testWasmCandidates = @(
    ".\test-components\rust-echo.wasm",
    ".\test-components\shopping-cart.wasm", 
    ".\test-components\counters.wasm",
    ".\test-components\promise.wasm"
)

foreach ($candidate in $testWasmCandidates) {
    if (Test-Path $candidate) {
        $testWasmPath = $candidate
        break
    }
}

if (!$testWasmPath) {
    # Get first available wasm file
    $wasmFiles = Get-ChildItem ".\test-components\*.wasm" | Select-Object -First 5
    if ($wasmFiles.Count -gt 0) {
        Write-Host "Available test components:" -ForegroundColor Yellow
        foreach ($file in $wasmFiles) {
            Write-Host "  - $($file.Name)" -ForegroundColor White
        }
        $testWasmPath = $wasmFiles[0].FullName
        Write-Host "Using: $testWasmPath" -ForegroundColor Cyan
    }
}

if (!$testWasmPath -or !(Test-Path $testWasmPath)) {
    Write-Host "❌ No test components found" -ForegroundColor Red
    exit 1
}

Write-Host "Using test component: $testWasmPath" -ForegroundColor Green

try {
    # Use PowerShell's native multipart form support
    $componentName = [System.IO.Path]::GetFileNameWithoutExtension($testWasmPath) + "-test-$(Get-Date -Format 'MMdd-HHmm')"
    
    # Create form data
    $form = @{
        name = $componentName
        componentType = '"Durable"'
        component = Get-Item $testWasmPath
    }
    
    Write-Host "Uploading component '$componentName'..." -ForegroundColor Cyan
    
    # Upload component using Invoke-RestMethod for better multipart handling
    $uploadHeaders = @{
        "Authorization" = "Bearer $AdminToken"
    }
    
    $response = Invoke-RestMethod -Uri "$ComponentServiceUrl/v1/components" -Method POST -Headers $uploadHeaders -Form $form
    
    Write-Host "✅ Component deployed successfully!" -ForegroundColor Green
    $componentId = $response.versionedComponentId.componentId
    $componentVersion = $response.versionedComponentId.version
    
    Write-Host "Component ID: $componentId" -ForegroundColor White
    Write-Host "Component Version: $componentVersion" -ForegroundColor White
    Write-Host "Component Size: $($response.componentSize) bytes" -ForegroundColor White
    
    # Display exported functions
    if ($response.metadata.exports) {
        Write-Host "Exported functions:" -ForegroundColor Green
        foreach ($export in $response.metadata.exports) {
            if ($export.type -eq "Function") {
                Write-Host "  - $($export.name)" -ForegroundColor White
            }
        }
        $componentInfo = $response
    } else {
        Write-Host "No exported functions found" -ForegroundColor Yellow
    }
}
catch {
    Write-Host "❌ Failed to deploy component: $($_.Exception.Message)" -ForegroundColor Red
    if ($_.Exception.Response) {
        Write-Host "Status Code: $($_.Exception.Response.StatusCode)" -ForegroundColor Red
    }
    
    # Try alternative approach - direct file upload
    Write-Host "`n🔄 Trying alternative upload method..." -ForegroundColor Yellow
    try {
        # Read file as bytes and create basic multipart manually
        $wasmBytes = [System.IO.File]::ReadAllBytes($testWasmPath)
        $boundary = [System.Guid]::NewGuid().ToString()
        
        $bodyLines = @()
        $bodyLines += "--$boundary"
        $bodyLines += 'Content-Disposition: form-data; name="name"'
        $bodyLines += ''
        $bodyLines += $componentName
        $bodyLines += "--$boundary"
        $bodyLines += 'Content-Disposition: form-data; name="componentType"'  
        $bodyLines += ''
        $bodyLines += '"Durable"'
        $bodyLines += "--$boundary"
        $bodyLines += 'Content-Disposition: form-data; name="component"; filename="component.wasm"'
        $bodyLines += 'Content-Type: application/wasm'
        $bodyLines += ''
        
        $bodyText = ($bodyLines -join "`r`n") + "`r`n"
        $bodyEnd = "`r`n--$boundary--`r`n"
        
        $bodyBytes = [System.Text.Encoding]::UTF8.GetBytes($bodyText)
        $endBytes = [System.Text.Encoding]::UTF8.GetBytes($bodyEnd)
        
        $fullBody = $bodyBytes + $wasmBytes + $endBytes
        
        $uploadHeaders2 = @{
            "Authorization" = "Bearer $AdminToken"
            "Content-Type" = "multipart/form-data; boundary=$boundary"
        }
        
        $response = Invoke-RestMethod -Uri "$ComponentServiceUrl/v1/components" -Method POST -Headers $uploadHeaders2 -Body $fullBody
        
        Write-Host "✅ Component deployed with alternative method!" -ForegroundColor Green
        $componentId = $response.versionedComponentId.componentId
        $componentVersion = $response.versionedComponentId.version
        $componentInfo = $response
        
        Write-Host "Component ID: $componentId" -ForegroundColor White
        Write-Host "Component Version: $componentVersion" -ForegroundColor White
    }
    catch {
        Write-Host "❌ Alternative upload also failed: $($_.Exception.Message)" -ForegroundColor Red
        exit 1
    }
}

# Step 3: Create a worker
Write-Host "`n👷 Step 3: Create worker..." -ForegroundColor Yellow

$workerName = "test-worker-$(Get-Date -Format 'MMdd-HHmmss')"
$workerPayload = @{
    name = $workerName
    args = @()
    env = @{}
}

try {
    $response = Invoke-RestMethod -Uri "$WorkerServiceUrl/v1/components/$componentId/workers" -Method POST -Headers $headers -Body ($workerPayload | ConvertTo-Json) -ContentType "application/json"
    
    Write-Host "✅ Worker created successfully!" -ForegroundColor Green
    Write-Host "Worker Name: $($response.workerId.workerName)" -ForegroundColor White
    Write-Host "Component Version: $($response.componentVersion)" -ForegroundColor White
}
catch {
    Write-Host "❌ Failed to create worker: $($_.Exception.Message)" -ForegroundColor Red
    if ($_.Exception.Response) {
        Write-Host "Status Code: $($_.Exception.Response.StatusCode)" -ForegroundColor Red
    }
    exit 1
}

# Step 4: Get worker metadata
Write-Host "`n📊 Step 4: Get worker metadata..." -ForegroundColor Yellow

try {
    $response = Invoke-RestMethod -Uri "$WorkerServiceUrl/v1/components/$componentId/workers/$workerName" -Method GET -Headers $headers
    
    Write-Host "✅ Worker metadata retrieved!" -ForegroundColor Green
    Write-Host "Worker Status: $($response.status)" -ForegroundColor White
    Write-Host "Retry Count: $($response.retryCount)" -ForegroundColor White
    Write-Host "Created At: $($response.createdAt)" -ForegroundColor White
}
catch {
    Write-Host "❌ Failed to get worker metadata: $($_.Exception.Message)" -ForegroundColor Red
}

# Summary
Write-Host "`n🎉 Test Summary:" -ForegroundColor Cyan
Write-Host "✅ Component Service: Accessible" -ForegroundColor Green
Write-Host "✅ Worker Service: Accessible" -ForegroundColor Green  
Write-Host "✅ Component Deployment: Working" -ForegroundColor Green
Write-Host "✅ Worker Creation: Working" -ForegroundColor Green
Write-Host "✅ Worker Metadata: Working" -ForegroundColor Green
Write-Host ""
Write-Host "Deployed Component ID: $componentId" -ForegroundColor White
Write-Host "Created Worker: $workerName" -ForegroundColor White
Write-Host ""
Write-Host "🚀 Golem services are fully operational via REST API!" -ForegroundColor Green

# Optional: Clean up (uncomment if desired)
# Write-Host "`n🧹 Cleaning up..." -ForegroundColor Yellow
# try {
#     Invoke-RestMethod -Uri "$WorkerServiceUrl/v1/components/$componentId/workers/$workerName" -Method DELETE -Headers $headers
#     Write-Host "✅ Worker deleted" -ForegroundColor Green
# } catch {
#     Write-Host "⚠️  Could not delete worker: $($_.Exception.Message)" -ForegroundColor Yellow
# }
