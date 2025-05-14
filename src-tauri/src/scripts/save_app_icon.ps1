$OutputEncoding = [Text.Encoding]::UTF8
$ExePathArr = @({EXE_PATH_ARR})
$OutputPathArr = @({OUTPUT_PATH_ARR})

Add-Type -AssemblyName System.Drawing

# Function to resize an image to 64x64 and save it
function Resize-And-Save-Image {
    param(
        [Parameter(Mandatory=$true)]
        [System.Drawing.Image]$sourceImage,
        
        [Parameter(Mandatory=$true)]
        [string]$outputPath
    )
    
    $resizedBitmap = New-Object System.Drawing.Bitmap(64, 64)
    $graphics = [System.Drawing.Graphics]::FromImage($resizedBitmap)
    $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $graphics.DrawImage($sourceImage, 0, 0, 64, 64)
    $graphics.Dispose()
    
    $resizedBitmap.Save($outputPath, [System.Drawing.Imaging.ImageFormat]::Png)
    $resizedBitmap.Dispose()
}

for ( $index = 0; $index -lt $ExePathArr.Count; $index++){
    $path = $ExePathArr[$index]
    $outputPath = $OutputPathArr[$index]
    
    # Determine app type based on path: if it contains \, it's a regular app, otherwise it's a store app
    $isStoreApp = $path -notmatch '\\'
    
    # Process based on app type
    if ($isStoreApp) {
        # Handle Windows Store App
        try {
            $appName = $path
            $package = Get-AppxPackage $appName
            if ($null -eq $package) {
                Write-Error "Store app not found: $appName"
                continue
            }

            $packageName = $package.PackageFullName
            $installPath = $package.InstallLocation
            $manifest = Get-AppxPackageManifest $packageName
            
            # Get Application, handling both array and non-array cases
            $application = $manifest.Package.Applications.Application
            if ($application -is [array]) {
                $application = $application[0]
            }
            
            # Get Square44x44Logo, handling both array and non-array cases
            $visualElements = $application.VisualElements
            $square44x44Logo = $visualElements.Square44x44Logo
            
            # Check if Square44x44Logo is an array or a single item
            if ($square44x44Logo -is [array]) {
                $iconPath = $square44x44Logo[0]
            } else {
                $iconPath = $square44x44Logo
            }
            
            $absoluteIconPath = Join-Path $installPath $iconPath
            $absoluteIconPath = $absoluteIconPath.Replace(".png", ".scale-*.png")
            
            # Get the highest resolution icon available
            $iconFiles = Get-ChildItem $absoluteIconPath
            if ($iconFiles.Count -eq 0) {
                Write-Error "No icon found for app: $appName"
                continue
            }
            $absoluteIconPath = $iconFiles[0].FullName

            # Read the image and resize it to 64x64
            $image = [System.Drawing.Image]::FromFile($absoluteIconPath)
            Resize-And-Save-Image -sourceImage $image -outputPath $outputPath
            $image.Dispose()

        } catch {
            Write-Error "Error extracting icon from store app $path : $_"
        }
    } else {
        # Handle Regular App
        if (-not(Test-Path $path)) {
            Write-Error "Path does not exist: $path"
            continue
        }
        
        try {
            $extension = [System.IO.Path]::GetExtension($path).ToLower()
            
            if ($extension -eq '.lnk') {
                $shell = New-Object -ComObject WScript.Shell
                $shortcut = $shell.CreateShortcut($path)
                if (![string]::IsNullOrEmpty($shortcut.TargetPath) -and (Test-Path $shortcut.TargetPath)) {
                    $icon = [System.Drawing.Icon]::ExtractAssociatedIcon($shortcut.TargetPath)
                } else {
                    $icon = [System.Drawing.Icon]::ExtractAssociatedIcon($path)
                }
                [System.Runtime.Interopservices.Marshal]::ReleaseComObject($shortcut) | Out-Null
                [System.Runtime.Interopservices.Marshal]::ReleaseComObject($shell) | Out-Null
            } else {
                $icon = [System.Drawing.Icon]::ExtractAssociatedIcon($path)
            }
            
            $bitmap = $icon.ToBitmap()
            Resize-And-Save-Image -sourceImage $bitmap -outputPath $outputPath
            $bitmap.Dispose()
            $icon.Dispose()
            
        } catch {
            Write-Error "Error extracting icon from $path : $_"
        }
    }
}