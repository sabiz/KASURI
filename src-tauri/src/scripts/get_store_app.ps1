# Get all start menu apps
$startApps = Get-StartApps

# Get all installed packages
$packages = Get-AppxPackage -PackageTypeFilter Main
$results = @()

# Build a hashtable for quick package lookup by package family name
$packageByFamilyName = @{}
foreach ($package in $packages) {
    $packageByFamilyName[$package.PackageFamilyName] = $package
}

# Process each start app and filter for Windows Store apps
foreach ($app in $startApps) {
    # Skip if not a Windows Store app (AppIDs for Store apps contain "!")
    if ($app.AppID -notlike "*!*") {
        continue
    }
    
    try {
        # Extract the package family name from the AppID
        $appIdParts = $app.AppID -split "!"
        $packageFamilyName = $appIdParts[0]
        $appId = $appIdParts[1]
        
        # Skip if we can't find the corresponding package
        if (-not $packageByFamilyName.ContainsKey($packageFamilyName)) {
            continue
        }
        
        $package = $packageByFamilyName[$packageFamilyName]
        
        # Get manifest information
        $manifest = Get-AppxPackageManifest $package.PackageFullName
        
        $appFound = $false
        
        # Try to find the specific app in the manifest
        foreach ($manifestApp in $manifest.Package.Applications.Application) {
            if ($manifestApp.Id -eq $appId) {
                # Found the matching app in the manifest
                $results += [PSCustomObject]@{
                    name = $app.Name
                    app_id = $app.AppID
                    package_fullname = $package.PackageFullName
                }
                $appFound = $true
                break
            }
        }
        
        # If we didn't find a specific app match in the manifest, still include the app
        # This is a fallback in case the manifest structure is different than expected
        if (-not $appFound) {
            $results += [PSCustomObject]@{
                name = $app.Name
                app_id = $app.AppID
                package_fullname = $package.PackageFullName
            }
        }
    } catch {
        # Skip apps that cannot be processed
        Write-Host "Error processing app $($app.Name): $_" -ForegroundColor Yellow
    }
}

# Change Encoding to UTF-8
$PSDefaultParameterValues['*:Encoding'] = 'utf8'
$global:OutputEncoding = [System.Text.Encoding]::UTF8
[console]::OutputEncoding = [System.Text.Encoding]::UTF8

$results | ConvertTo-Json -Depth 10