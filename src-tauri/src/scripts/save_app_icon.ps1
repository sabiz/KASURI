$ExePathArr = @({EXE_PATH_ARR})
$OutputPathArr = @({OUTPUT_PATH_ARR})

Add-Type -AssemblyName System.Drawing

for ( $index = 0; $index -lt $ExePathArr.Count; $index++){
    $path = $ExePathArr[$index]
    $outputPath = $OutputPathArr[$index]
    if (-not(Test-Path $path)) {
        Write-Error "Path does not exist: $path"
        continue
    }
    
    try {
        $extension = [System.IO.Path]::GetExtension($path).ToLower()
        
        if ($extension -eq '.lnk') {
            $shell = New-Object -ComObject WScript.Shell
            $shortcut = $shell.CreateShortcut($path)
            if (Test-Path $shortcut.TargetPath) {
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
        $resizedBitmap = New-Object System.Drawing.Bitmap(64, 64)
        $graphics = [System.Drawing.Graphics]::FromImage($resizedBitmap)
        $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
        $graphics.DrawImage($bitmap, 0, 0, 64, 64)
        $graphics.Dispose()

        $resizedBitmap.Save($outputPath, [System.Drawing.Imaging.ImageFormat]::Png)
        $resizedBitmap.Dispose()
        $bitmap.Dispose()
        $icon.Dispose()
        
    } catch {
        Write-Error "Error extracting icon from $path : $_"
    }
}