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

$inputPath = Join-Path $folder "%d.$inputType"
$folderName = Split-Path -Path $folder -Leaf
$outputPath = Join-Path output "$folderName.mp4"


ffmpeg -i "$inputPath" -r 24 -c:v libx264 -preset slow -vf format=yuv420p -crf 18 $outputPath