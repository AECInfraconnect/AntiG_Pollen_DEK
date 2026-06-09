import os

HEADER = """// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

"""

for root, dirs, files in os.walk('.'):
    if '.git' in root or 'target' in root:
        continue
    for f in files:
        path = os.path.join(root, f)
        if f == 'Cargo.toml':
            with open(path, 'r', encoding='utf-8') as file:
                content = file.read()
            if 'license.workspace' not in content and 'workspace.package' not in content:
                content = content.replace('edition = "2021"', 'edition = "2021"\nlicense.workspace = true')
                content = content.replace('edition = "2024"', 'edition = "2024"\nlicense.workspace = true')
                with open(path, 'w', encoding='utf-8') as file:
                    file.write(content)
        elif f.endswith('.rs'):
            with open(path, 'r', encoding='utf-8') as file:
                content = file.read()
            if 'SPDX-License-Identifier' not in content:
                with open(path, 'w', encoding='utf-8') as file:
                    file.write(HEADER + content)

print("License Tiering Applied Successfully!")
