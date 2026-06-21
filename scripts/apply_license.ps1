Write-Host "Applying License Tiering..."
$header = "// SPDX-License-Identifier: Apache-2.0`n// Copyright (c) 2026 AEC Infraconnect`n`n"
Get-ChildItem -Path crates,plugins -Recurse -Filter Cargo.toml | ForEach-Object {
    $content = Get-Content $_.FullName -Raw
    if (-not $content.Contains("license.workspace")) {
        $content = $content -replace 'edition = "2021"', "edition = `"2021`"`nlicense.workspace = true"
        $content = $content -replace 'edition = "2024"', "edition = `"2024`"`nlicense.workspace = true"
        Set-Content -Path $_.FullName -Value $content
    }
}
Get-ChildItem -Path crates,plugins -Recurse -Filter *.rs | ForEach-Object {
    $content = Get-Content $_.FullName -Raw
    if (-not $content.Contains("SPDX-License-Identifier")) {
        $newContent = $header + $content
        Set-Content -Path $_.FullName -Value $newContent
    }
}
Write-Host "License Tiering Applied Successfully!"
