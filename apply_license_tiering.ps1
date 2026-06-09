$header = "// SPDX-License-Identifier: Apache-2.0`n// Copyright (c) 2026 AEC Infraconnect`n`n"

Get-ChildItem -Path "crates" -Recurse -Filter "*.rs" | ForEach-Object {
    $content = Get-Content $_.FullName -Raw
    if (-not $content.StartsWith("// SPDX-License-Identifier: Apache-2.0")) {
        $newContent = $header + $content
        Set-Content -Path $_.FullName -Value $newContent -NoNewline
        Write-Host "Added header to $($_.FullName)"
    }
}

Get-ChildItem -Path "crates" -Recurse -Filter "Cargo.toml" | ForEach-Object {
    $content = Get-Content $_.FullName -Raw
    if ($content -notmatch "license.workspace\s*=") {
        # Insert license.workspace = true after version.workspace = true or edition.workspace = true
        $newContent = $content -replace "(edition\.workspace\s*=\s*true)", "`$1`nlicense.workspace = true"
        if ($newContent -eq $content) {
            $newContent = $content -replace "(version\s*=\s*[^`n]+)", "`$1`nlicense.workspace = true"
        }
        if ($newContent -ne $content) {
            Set-Content -Path $_.FullName -Value $newContent -NoNewline
            Write-Host "Added license.workspace to $($_.FullName)"
        }
    }
}
