# PowerShell script to generate a multi-size .ico file from kasuri_logo.svg using ImageMagick
# Requirements: ImageMagick must be installed and 'magick' command available in PATH

$ErrorActionPreference = 'Stop'

$svgPath = Join-Path $PSScriptRoot 'kasuri_logo.svg'
$outputIco = Join-Path $PSScriptRoot 'kasuri.ico'

# Define icon sizes
$sizes = @(16, 24, 32, 48, 64, 128, 256)
$pngFiles = @()

Write-Host "Generating PNGs from SVG..."
foreach ($size in $sizes) {
    $png = Join-Path $PSScriptRoot ("kasuri_logo_${size}.png")
    $pngFiles += $png
    magick -density 1200 -background none "$svgPath" -resize ${size}x${size} "$png"
    Write-Host "Generated: $png"
}


# Combine PNGs into multi-size ICO (pass each file as a separate argument)
Write-Host "Combining PNGs into multi-size ICO..."
magick @pngFiles "$outputIco"

# Clean up temporary PNGs
foreach ($png in $pngFiles) {
    Remove-Item $png -ErrorAction SilentlyContinue
}

Write-Host "Multi-size icon generated: $outputIco"
