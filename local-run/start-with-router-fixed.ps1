# Golem Services + Router Complete Start Script for Windows
# PowerShell script that starts all services INCLUDING the nginx router

param(
    [int]$RouterPort = 80,
    [switch]$SkipRouter = $false
)

# Clean up any existing data
if (Test-Path ".\local-run\data\shard-manager") { 
    Remove-Item -Recurse -Force ".\local-run\data\shard-manager" 
}

# Create directories
New-Item -ItemType Directory -Path ".\local-run\data\redis", ".\local-run\data\shard-manager", ".\local-run\logs", ".\local-run\nginx" -Force | Out-Null

# Set environment variables
$env:RUST_BACKTRACE = "1"
$env:GOLEM__TRACING__FILE_DIR = "$PWD\local-run\logs"
$env:GOLEM__TRACING__FILE__ANSI = "true"
$env:GOLEM__TRACING__FILE__ENABLED = "true"
$env:GOLEM__TRACING__FILE__JSON = "false"
$env:GOLEM__TRACING__STDOUT__ENABLED = "false"

$ADMIN_TOKEN = "5c832d93-ff85-4a8f-9803-513950fdfdb1"
$FS_BLOB_STORAGE_DIR = "$PWD\local-run\data\blob_storage"

# Port configurations
$CLOUD_SERVICE_HTTP_PORT = 8080
$COMPONENT_COMPILATION_SERVICE_HTTP_PORT = 8081
$COMPONENT_SERVICE_HTTP_PORT = 8082
$SHARD_MANAGER_HTTP_PORT = 8083
$WORKER_EXECUTOR_HTTP_PORT = 8084
$WORKER_SERVICE_HTTP_PORT = 8085
$WORKER_SERVICE_CUSTOM_REQUEST_HTTP_PORT = 8086
$DEBUGGING_SERVICE_HTTP_PORT = 8087

$CLOUD_SERVICE_GRPC_PORT = 9090
$COMPONENT_COMPILATION_SERVICE_GRPC_PORT = 9091
$COMPONENT_SERVICE_GRPC_PORT = 9092
$SHARD_MANAGER_GRPC_PORT = 9093
$WORKER_EXECUTOR_GRPC_PORT = 9094
$WORKER_SERVICE_GRPC_PORT = 9095

Write-Host "Starting Golem services with Router on Windows..." -ForegroundColor Cyan

# Array to store service processes for cleanup
$processes = @()

# Function to check if nginx is available
function Test-NginxAvailability {
    $nginxPath = ""
    $possiblePaths = @(
        "nginx",
        "C:\nginx\nginx.exe",
        "C:\Program Files\nginx\nginx.exe",
        "$env:ProgramFiles\nginx\nginx.exe"
    )

    foreach ($path in $possiblePaths) {
        try {
            $null = Get-Command $path -ErrorAction Stop
            return $path
        }
        catch {
            # Continue searching
        }
    }
    return $null
}

try {
    # Start Redis
    Write-Host "Starting Redis..."
    $redis_process = Start-Process -FilePath "redis-server" -ArgumentList "--port", "6380", "--appendonly", "yes", "--dir", ".\local-run\data\redis" -WindowStyle Hidden -PassThru
    $processes += $redis_process
    Start-Sleep -Seconds 2

    # Start Cloud Service
    Write-Host "Starting Cloud Service..."
    Set-Location "cloud-service"
    $env:RUST_LOG = "info,h2=warn,hyper=warn,tower=warn"
    $env:GOLEM__HTTP_PORT = $CLOUD_SERVICE_HTTP_PORT
    $env:GOLEM__GRPC_PORT = $CLOUD_SERVICE_GRPC_PORT
    $env:GOLEM__LOGIN__TYPE = $env:GOLEM_CLOUD_SERVICE_LOGIN_TYPE
    $env:GOLEM__DB__TYPE = "Sqlite"
    $env:GOLEM__DB__CONFIG__DATABASE = "..\local-run\data\golem_cloud_service.db"
    $env:GOLEM__CORS_ORIGIN_REGEX = "http://localhost:3000"
    $env:GOLEM__ACCOUNTS__ROOT__TOKEN = $ADMIN_TOKEN
    
    $cloud_process = Start-Process -FilePath "..\target\debug\cloud-service.exe" -WindowStyle Hidden -PassThru
    $processes += $cloud_process
    Set-Location ".."
    Start-Sleep -Seconds 3

    # Start Component Compilation Service
    Write-Host "Starting Component Compilation Service..."
    Set-Location "golem-component-compilation-service"
    $env:GOLEM__HTTP_PORT = $COMPONENT_COMPILATION_SERVICE_HTTP_PORT
    $env:GOLEM__GRPC_PORT = $COMPONENT_COMPILATION_SERVICE_GRPC_PORT
    $env:GOLEM__BLOB_STORAGE__TYPE = "LocalFileSystem"
    $env:GOLEM__BLOB_STORAGE__CONFIG__ROOT = $FS_BLOB_STORAGE_DIR
    $env:GOLEM__COMPONENT_SERVICE__CONFIG__PORT = $COMPONENT_SERVICE_GRPC_PORT
    $env:GOLEM__COMPONENT_SERVICE__CONFIG__ACCESS_TOKEN = $ADMIN_TOKEN
    
    $comp_compilation_process = Start-Process -FilePath "..\target\debug\golem-component-compilation-service.exe" -WindowStyle Hidden -PassThru
    $processes += $comp_compilation_process
    Set-Location ".."
    Start-Sleep -Seconds 3

    # Start Component Service
    Write-Host "Starting Component Service..."
    Set-Location "golem-component-service"
    $env:GOLEM__HTTP_PORT = $COMPONENT_SERVICE_HTTP_PORT
    $env:GOLEM__GRPC_PORT = $COMPONENT_SERVICE_GRPC_PORT
    $env:GOLEM__BLOB_STORAGE__TYPE = "LocalFileSystem"
    $env:GOLEM__BLOB_STORAGE__CONFIG__ROOT = $FS_BLOB_STORAGE_DIR
    $env:GOLEM__CLOUD_SERVICE__PORT = $CLOUD_SERVICE_GRPC_PORT
    $env:GOLEM__CLOUD_SERVICE__ACCESS_TOKEN = $ADMIN_TOKEN
    $env:GOLEM__COMPILATION__TYPE = "Enabled"
    $env:GOLEM__COMPILATION__CONFIG__PORT = $COMPONENT_COMPILATION_SERVICE_GRPC_PORT
    $env:GOLEM__DB__TYPE = "Sqlite"
    $env:GOLEM__DB__CONFIG__DATABASE = "..\local-run\data\golem_component.db"
    $env:GOLEM__CORS_ORIGIN_REGEX = "http://localhost:3000"
    
    $comp_service_process = Start-Process -FilePath "..\target\debug\golem-component-service.exe" -WindowStyle Hidden -PassThru
    $processes += $comp_service_process
    Set-Location ".."
    Start-Sleep -Seconds 3

    # Start Shard Manager
    Write-Host "Starting Shard Manager..."
    Set-Location "golem-shard-manager"
    $env:GOLEM__HTTP_PORT = $SHARD_MANAGER_HTTP_PORT
    $env:GOLEM__GRPC_PORT = $SHARD_MANAGER_GRPC_PORT
    $env:GOLEM__PERSISTENCE__TYPE = "FileSystem"
    $env:GOLEM__PERSISTENCE__CONFIG__PATH = "..\local-run\data\shard-manager\data.bin"
    
    $shard_process = Start-Process -FilePath "..\target\debug\golem-shard-manager.exe" -WindowStyle Hidden -PassThru
    $processes += $shard_process
    Set-Location ".."
    Start-Sleep -Seconds 3

    # Start Worker Executor
    Write-Host "Starting Worker Executor..."
    Set-Location "golem-worker-executor"
    $env:RUST_LOG = "info"
    $env:GOLEM__HTTP_PORT = $WORKER_EXECUTOR_HTTP_PORT
    $env:GOLEM__PORT = $WORKER_EXECUTOR_GRPC_PORT
    $env:GOLEM__PUBLIC_WORKER_API__PORT = $WORKER_SERVICE_GRPC_PORT
    $env:GOLEM__PUBLIC_WORKER_API__ACCESS_TOKEN = $ADMIN_TOKEN
    $env:GOLEM__BLOB_STORAGE__TYPE = "LocalFileSystem"
    $env:GOLEM__BLOB_STORAGE__CONFIG__ROOT = $FS_BLOB_STORAGE_DIR
    $env:GOLEM__PLUGIN_SERVICE__CONFIG__PORT = $COMPONENT_SERVICE_GRPC_PORT
    $env:GOLEM__PLUGIN_SERVICE__CONFIG__ACCESS_TOKEN = $ADMIN_TOKEN
    $env:GOLEM__SHARD_MANAGER_SERVICE__CONFIG__PORT = $SHARD_MANAGER_GRPC_PORT
    $env:GOLEM__COMPONENT_SERVICE__CONFIG__PORT = $COMPONENT_SERVICE_GRPC_PORT
    $env:GOLEM__COMPONENT_SERVICE__CONFIG__ACCESS_TOKEN = $ADMIN_TOKEN
    
    $worker_exec_process = Start-Process -FilePath "..\target\debug\worker-executor.exe" -WindowStyle Hidden -PassThru
    $processes += $worker_exec_process
    Set-Location ".."
    Start-Sleep -Seconds 3

    # Start Worker Service
    Write-Host "Starting Worker Service..."
    Set-Location "golem-worker-service"
    $env:RUST_LOG = "debug,h2=warn,hyper=warn,tower=warn"
    $env:GOLEM__PORT = $WORKER_SERVICE_HTTP_PORT
    $env:GOLEM__CUSTOM_REQUEST_PORT = $WORKER_SERVICE_CUSTOM_REQUEST_HTTP_PORT
    $env:GOLEM__WORKER_GRPC_PORT = $WORKER_SERVICE_GRPC_PORT
    $env:GOLEM__BLOB_STORAGE__TYPE = "LocalFileSystem"
    $env:GOLEM__BLOB_STORAGE__CONFIG__ROOT = $FS_BLOB_STORAGE_DIR
    $env:GOLEM__DB__TYPE = "Sqlite"
    $env:GOLEM__DB__CONFIG__DATABASE = "..\local-run\data\golem_worker.sqlite"
    $env:GOLEM__COMPONENT_SERVICE__PORT = $COMPONENT_SERVICE_GRPC_PORT
    $env:GOLEM__COMPONENT_SERVICE__ACCESS_TOKEN = $ADMIN_TOKEN
    $env:GOLEM__ROUTING_TABLE__PORT = $SHARD_MANAGER_GRPC_PORT
    $env:GOLEM__CORS_ORIGIN_REGEX = "http://localhost:3000"
    $env:GOLEM__CLOUD_SERVICE__PORT = $CLOUD_SERVICE_GRPC_PORT
    $env:GOLEM__CLOUD_SERVICE__ACCESS_TOKEN = $ADMIN_TOKEN
    
    $worker_service_process = Start-Process -FilePath "..\target\debug\golem-worker-service.exe" -WindowStyle Hidden -PassThru
    $processes += $worker_service_process
    Set-Location ".."
    Start-Sleep -Seconds 3

    # Start Debugging Service
    Write-Host "Starting Debugging Service..."
    Set-Location "golem-debugging-service"
    $env:RUST_LOG = "info"
    $env:GOLEM__HTTP_PORT = $DEBUGGING_SERVICE_HTTP_PORT
    $env:GOLEM__CLOUD_SERVICE__PORT = $CLOUD_SERVICE_GRPC_PORT
    $env:GOLEM__CLOUD_SERVICE__ACCESS_TOKEN = $ADMIN_TOKEN
    $env:GOLEM__PUBLIC_WORKER_API__PORT = $WORKER_SERVICE_GRPC_PORT
    $env:GOLEM__PUBLIC_WORKER_API__ACCESS_TOKEN = $ADMIN_TOKEN
    $env:GOLEM__BLOB_STORAGE__TYPE = "LocalFileSystem"
    $env:GOLEM__BLOB_STORAGE__CONFIG__ROOT = $FS_BLOB_STORAGE_DIR
    $env:GOLEM__PLUGIN_SERVICE__CONFIG__PORT = $COMPONENT_SERVICE_GRPC_PORT
    $env:GOLEM__PLUGIN_SERVICE__CONFIG__ACCESS_TOKEN = $ADMIN_TOKEN
    $env:GOLEM__COMPONENT_SERVICE__PORT = $COMPONENT_SERVICE_GRPC_PORT
    $env:GOLEM__COMPONENT_SERVICE__ACCESS_TOKEN = $ADMIN_TOKEN
    $env:GOLEM__CORS_ORIGIN_REGEX = "http://localhost:3000"
    
    $debug_process = Start-Process -FilePath "..\target\debug\golem-debugging-service.exe" -WindowStyle Hidden -PassThru
    $processes += $debug_process
    Set-Location ".."
    Start-Sleep -Seconds 3

    # Start Router (nginx) if not skipped
    $nginx_process = $null
    if (!$SkipRouter) {
        Write-Host "Starting Golem Router (nginx)..." -ForegroundColor Yellow
        
        $nginxPath = Test-NginxAvailability
        if ($nginxPath) {
            # Generate nginx config
            $nginxWorkDir = "$PWD\local-run\nginx"
            $templatePath = "$PWD\golem-router\golem-services.conf.template"
            $configPath = "$nginxWorkDir\nginx.conf"
            
            if (Test-Path $templatePath) {
                # Set router environment variables
                $env:GOLEM_COMPONENT_SERVICE_HOST = "localhost"
                $env:GOLEM_COMPONENT_SERVICE_PORT = $COMPONENT_SERVICE_HTTP_PORT
                $env:GOLEM_WORKER_SERVICE_HOST = "localhost"
                $env:GOLEM_WORKER_SERVICE_PORT = $WORKER_SERVICE_HTTP_PORT
                $env:GOLEM_COMPONENT_MAX_SIZE_ALLOWED = "50M"
                
                # Generate config
                $template = Get-Content $templatePath -Raw
                $config = $template -replace '\$GOLEM_COMPONENT_SERVICE_HOST', $env:GOLEM_COMPONENT_SERVICE_HOST
                $config = $config -replace '\$GOLEM_COMPONENT_SERVICE_PORT', $env:GOLEM_COMPONENT_SERVICE_PORT
                $config = $config -replace '\$GOLEM_WORKER_SERVICE_HOST', $env:GOLEM_WORKER_SERVICE_HOST
                $config = $config -replace '\$GOLEM_WORKER_SERVICE_PORT', $env:GOLEM_WORKER_SERVICE_PORT
                $config = $config -replace '\$GOLEM_COMPONENT_MAX_SIZE_ALLOWED', $env:GOLEM_COMPONENT_MAX_SIZE_ALLOWED
                $config = $config -replace 'resolver 127\.0\.0\.11;', 'resolver 8.8.8.8;'
                
                if ($RouterPort -ne 80) {
                    $config = $config -replace 'listen 80;', "listen $RouterPort;"
                }
                
                Set-Content -Path $configPath -Value $config -Encoding UTF8
                New-Item -ItemType Directory -Path "$nginxWorkDir\logs", "$nginxWorkDir\temp" -Force | Out-Null
                
                # Start nginx
                $nginx_process = Start-Process -FilePath $nginxPath -ArgumentList "-c", $configPath, "-p", $nginxWorkDir -PassThru -NoNewWindow
                $processes += $nginx_process
                Start-Sleep -Seconds 2
                
                if (!$nginx_process.HasExited) {
                    Write-Host "✅ Router started successfully on port $RouterPort" -ForegroundColor Green
                } else {
                    Write-Host "⚠️  Router failed to start" -ForegroundColor Red
                }
            } else {
                Write-Host "⚠️  Router config template not found, skipping router" -ForegroundColor Yellow
            }
        } else {
            Write-Host "⚠️  nginx not found, skipping router. Install nginx for Windows to enable routing." -ForegroundColor Yellow
        }
    }

    Write-Host "`n🚀 Golem Services Status:" -ForegroundColor Cyan
    Write-Host " - Redis:                         $($redis_process.Id)"
    Write-Host " - Cloud Service:                 $($cloud_process.Id)"
    Write-Host " - Component Compilation Service: $($comp_compilation_process.Id)"
    Write-Host " - Component Service:             $($comp_service_process.Id)"
    Write-Host " - Shard Manager:                 $($shard_process.Id)"
    Write-Host " - Worker Executor:               $($worker_exec_process.Id)"
    Write-Host " - Worker Service:                $($worker_service_process.Id)"
    Write-Host " - Debugging Service:             $($debug_process.Id)"
    if ($nginx_process -and !$nginx_process.HasExited) {
        Write-Host " - Router (nginx):                $($nginx_process.Id)" -ForegroundColor Green
    }
    
    Write-Host "`n🌐 Available Endpoints:" -ForegroundColor Cyan
    if ($nginx_process -and !$nginx_process.HasExited) {
        Write-Host "🔀 ROUTER (Recommended):" -ForegroundColor Green
        Write-Host "   Main API: http://localhost:$RouterPort" -ForegroundColor White
        Write-Host "   Components: http://localhost:$RouterPort/v1/components" -ForegroundColor White
        Write-Host "   Workers: http://localhost:$RouterPort/v1/components/{id}/workers" -ForegroundColor White
        Write-Host ""
    }
    Write-Host "📋 Direct Service Access:"
    Write-Host "   Cloud Service: http://localhost:$CLOUD_SERVICE_HTTP_PORT"
    Write-Host "   Component Service: http://localhost:$COMPONENT_SERVICE_HTTP_PORT"
    Write-Host "   Worker Service: http://localhost:$WORKER_SERVICE_HTTP_PORT"
    
    Write-Host "`nAll services are running. Press Ctrl+C to stop all services." -ForegroundColor Yellow
    
    # Wait for user interrupt
    while ($true) {
        Start-Sleep -Seconds 1
    }
}
catch {
    Write-Host "Error occurred: $_" -ForegroundColor Red
}
finally {
    # Cleanup - stop all processes
    Write-Host "`nStopping all services..."
    foreach ($process in $processes) {
        if ($process -and !$process.HasExited) {
            try {
                Stop-Process -Id $process.Id -Force
                Write-Host "Stopped process $($process.Id)"
            }
            catch {
                Write-Host "Failed to stop process $($process.Id): $_"
            }
        }
    }
    Write-Host "All services stopped."
}
