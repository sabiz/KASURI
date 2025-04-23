$filePath = "<app_path>"
$icon = [System.Drawing.Icon]::ExtractAssociatedIcon($filePath)
$bitmap = $icon.ToBitmap()
# $outputPath = "<out_path>"
# $bitmap.Save($outputPath, [System.Drawing.Imaging.ImageFormat]::Png)
$ms = New-Object System.IO.MemoryStream
$bitmap.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
$byteArray = $ms.ToArray()
$base64String = [Convert]::ToBase64String($byteArray)
$dataUri = "data:image/png;base64," + $base64String
$icon.Dispose()
$bitmap.Dispose()
$ms.Dispose()
