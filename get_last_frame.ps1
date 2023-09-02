param (
    [string]
    $folder,

    [string]
    $inputType = 'bmp'
)

if (-not (Test-Path $folder -PathType Container)) {
    Write-Output "$folder is not a folder!"
    return
}

$folderName = Split-Path -Path $folder -Leaf
$outputPath = Join-Path output "${folderName}_last.bmp"


Get-ChildItem -Path $folder -Filter *.bmp | Sort-Object LastWriteTime -Descending | Select-Object -First 1 | ForEach-Object { Copy-Item $_ $outputPath }