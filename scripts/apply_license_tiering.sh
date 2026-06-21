#!/usr/bin/env bash
set -e

echo "Applying License Tiering..."

# Add license.workspace = true to all Cargo.toml
for toml in $(find crates plugins -name Cargo.toml); do
    if ! grep -q "license.workspace" "$toml"; then
        sed -i 's/edition = "2021"/edition = "2021"\nlicense.workspace = true/' "$toml"
        # Also handle 2024
        sed -i 's/edition = "2024"/edition = "2024"\nlicense.workspace = true/' "$toml"
    fi
done

# Add SPDX header to all .rs files
HEADER="// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect
"

for rs in $(find crates plugins -name "*.rs"); do
    if ! grep -q "SPDX-License-Identifier" "$rs"; then
        # Create a temporary file
        temp_file=$(mktemp)
        echo "$HEADER" > "$temp_file"
        cat "$rs" >> "$temp_file"
        mv "$temp_file" "$rs"
    fi
done

echo "License Tiering Applied Successfully!"
