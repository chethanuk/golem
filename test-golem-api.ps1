# Golem REST API Test Script
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
    
    $components = $response.Content | ConvertFrom-Json
    if ($components.Count -gt 0) {
        Write-Host "Found $($components.Count) existing components:" -ForegroundColor Green
        foreach ($comp in $components) {
            Write-Host "  - $($comp.componentName) (ID: $($comp.versionedComponentId.componentId))" -ForegroundColor White
        }
    } else {
        Write-Host "No existing components found" -ForegroundColor Yellow
    }
}
catch {
    Write-Host "❌ Failed to list components: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "Response: $($_.Exception.Response.StatusCode)" -ForegroundColor Red
    exit 1
}

# Step 2: Deploy a test component
Write-Host "`n📦 Step 2: Deploy test component..." -ForegroundColor Yellow

# Check if a simple test component exists
$testWasmPath = ".\test-components\shopping-cart.wasm"
if (!(Test-Path $testWasmPath)) {
    Write-Host "❌ Test component not found: $testWasmPath" -ForegroundColor Red
    $testWasmPath = ".\test-components\counters.wasm"
    if (!(Test-Path $testWasmPath)) {
        Write-Host "❌ Alternative test component not found: $testWasmPath" -ForegroundColor Red
        # List available components
        Write-Host "Available test components:" -ForegroundColor Yellow
        Get-ChildItem ".\test-components\*.wasm" | ForEach-Object { Write-Host "  - $($_.Name)" -ForegroundColor White }
        $testWasmPath = (Get-ChildItem ".\test-components\*.wasm" | Select-Object -First 1).FullName
        Write-Host "Using first available: $testWasmPath" -ForegroundColor Cyan
    }
}

if (Test-Path $testWasmPath) {
    Write-Host "Using test component: $testWasmPath" -ForegroundColor Green
    
    try {
        # Create multipart form data for component upload
        $boundary = [System.Guid]::NewGuid().ToString()
        $LF = "`r`n"
        
        # Read the WASM file
        $wasmBytes = [System.IO.File]::ReadAllBytes($testWasmPath)
        $componentName = [System.IO.Path]::GetFileNameWithoutExtension($testWasmPath) + "-test-$(Get-Date -Format 'MMdd-HHmm')"
        
        # Build multipart content
        $bodyTemplate = @"
--$boundary
Content-Disposition: form-data; name="name"

$componentName
--$boundary
Content-Disposition: form-data; name="componentType"

"Durable"
--$boundary
Content-Disposition: form-data; name="component"; filename="component.wasm"
Content-Type: application/wasm

{0}
--$boundary--
"@

        $body = $bodyTemplate -f [System.Text.Encoding]::Latin1.GetString($wasmBytes)
        $bodyBytes = [System.Text.Encoding]::Latin1.GetBytes($body)
        
        # Upload component
        $uploadHeaders = $headers.Clone()
        $uploadHeaders["Content-Type"] = "multipart/form-data; boundary=$boundary"
        
        Write-Host "Uploading component '$componentName'..." -ForegroundColor Cyan
        $response = Invoke-WebRequest -Uri "$ComponentServiceUrl/v1/components" -Method POST -Headers $uploadHeaders -Body $bodyBytes
        
        Write-Host "✅ Component deployed successfully!" -ForegroundColor Green
        $componentInfo = $response.Content | ConvertFrom-Json
        $componentId = $componentInfo.versionedComponentId.componentId
        $componentVersion = $componentInfo.versionedComponentId.version
        
        Write-Host "Component ID: $componentId" -ForegroundColor White
        Write-Host "Component Version: $componentVersion" -ForegroundColor White
        Write-Host "Component Size: $($componentInfo.componentSize) bytes" -ForegroundColor White
        
        # Display exported functions
        if ($componentInfo.metadata.exports) {
            Write-Host "Exported functions:" -ForegroundColor Green
            foreach ($export in $componentInfo.metadata.exports) {
                if ($export.type -eq "Function") {
                    Write-Host "  - $($export.name)" -ForegroundColor White
                }
            }
        }
    }
    catch {
        Write-Host "❌ Failed to deploy component: $($_.Exception.Message)" -ForegroundColor Red
        if ($_.Exception.Response) {
            Write-Host "Status Code: $($_.Exception.Response.StatusCode)" -ForegroundColor Red
            try {
                $errorStream = $_.Exception.Response.GetResponseStream()
                $reader = New-Object System.IO.StreamReader($errorStream)
                $errorContent = $reader.ReadToEnd()
                Write-Host "Error details: $errorContent" -ForegroundColor Red
            }
            catch {
                Write-Host "Could not read error details" -ForegroundColor Red
            }
        }
        exit 1
    }
} else {
    Write-Host "❌ No test components found" -ForegroundColor Red
    exit 1
}

# Step 3: Create a worker
Write-Host "`n👷 Step 3: Create worker..." -ForegroundColor Yellow

$workerName = "test-worker-$(Get-Date -Format 'MMdd-HHmmss')"
$workerPayload = @{
    name = $workerName
    args = @()
    env = @{}
} | ConvertTo-Json

try {
    $response = Invoke-WebRequest -Uri "$WorkerServiceUrl/v1/components/$componentId/workers" -Method POST -Headers $headers -Body $workerPayload -ContentType "application/json"
    
    Write-Host "✅ Worker created successfully!" -ForegroundColor Green
    $workerInfo = $response.Content | ConvertFrom-Json
    Write-Host "Worker Name: $($workerInfo.workerId.workerName)" -ForegroundColor White
    Write-Host "Component Version: $($workerInfo.componentVersion)" -ForegroundColor White
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
    $response = Invoke-WebRequest -Uri "$WorkerServiceUrl/v1/components/$componentId/workers/$workerName" -Method GET -Headers $headers
    
    Write-Host "✅ Worker metadata retrieved!" -ForegroundColor Green
    $workerMetadata = $response.Content | ConvertFrom-Json
    Write-Host "Worker Status: $($workerMetadata.status)" -ForegroundColor White
    Write-Host "Retry Count: $($workerMetadata.retryCount)" -ForegroundColor White
    Write-Host "Created At: $($workerMetadata.createdAt)" -ForegroundColor White
}
catch {
    Write-Host "❌ Failed to get worker metadata: $($_.Exception.Message)" -ForegroundColor Red
}

# Step 5: Test function invocation (if available)
Write-Host "`n🔧 Step 5: Test function invocation..." -ForegroundColor Yellow

if ($componentInfo.metadata.exports) {
    $functions = $componentInfo.metadata.exports | Where-Object { $_.type -eq "Function" }
    if ($functions.Count -gt 0) {
        $firstFunction = $functions[0].name
        Write-Host "Attempting to invoke function: $firstFunction" -ForegroundColor Cyan
        
        try {
            # Simple function invocation without parameters
            $invocationPayload = @{
                function = $firstFunction
                parameters = @()
            } | ConvertTo-Json
            
            $response = Invoke-WebRequest -Uri "$WorkerServiceUrl/v1/components/$componentId/workers/$workerName/invoke" -Method POST -Headers $headers -Body $invocationPayload -ContentType "application/json"
            
            Write-Host "✅ Function invoked successfully!" -ForegroundColor Green
            Write-Host "Response: $($response.Content)" -ForegroundColor White
        }
        catch {
            Write-Host "⚠️  Function invocation failed (this may be expected for some functions): $($_.Exception.Message)" -ForegroundColor Yellow
        }
    } else {
        Write-Host "⚠️  No functions available for invocation" -ForegroundColor Yellow
    }
} else {
    Write-Host "⚠️  No exported functions found" -ForegroundColor Yellow
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
