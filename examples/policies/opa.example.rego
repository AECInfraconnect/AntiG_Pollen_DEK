# SPDX-License-Identifier: Apache-2.0
# Copyright (c) 2026 AEC Infraconnect

package example.policy

default allow := false

allow {
    input.context.has_valid_mtls == true
    input.context.user_department == "engineering"
}
