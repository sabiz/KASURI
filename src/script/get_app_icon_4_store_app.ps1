$package = Get-AppxPackage <AppName>
$packageName = $package.PackageFullName
$installPath = $package.InstallLocation
$manifest = Get-AppxPackageManifest $packageName
$iconPath = $manifest.Package.Applications.Application[0].VisualElements.Square44x44Logo[0]
$absoluteIconPath = Join-Path $installPath $iconPath
$absoluteIconPath = $absoluteIconPath.Replace(".png", ".scale-*.png")
$absoluteIconPath = (Get-ChildItem $absoluteIconPath)[0].fullName
$imageBytes = [System.IO.File]::ReadAllBytes($absoluteIconPath)
$base64String = [Convert]::ToBase64String($imageBytes)
$dataUri = "data:image/png;base64," + $base64String
