# Pollek DEK Example Policies

This directory contains simple generic policies for each of the supported Policy Decision Point (PDP) engines in the Pollek DEK ecosystem.

## Supported PDPs

### 1. AWS Cedar (`cedar.example.txt`)

AWS Cedar is a policy language and evaluation engine for identity-based and resource-based access control.

### 2. Open Policy Agent / OPA (`opa.example.rego`)

OPA is a general-purpose policy engine. Policies are written in Rego. Pollek DEK executes OPA policies as compiled WebAssembly (WASM) modules.

### 3. OpenFGA (`openfga.example.json`)

OpenFGA is a Relationship-Based Access Control (ReBAC) engine inspired by Google Zanzibar.

## Usage

When configuring a route in `dek-router-builder`, point the configuration to the compiled binary paths or endpoint URLs of these policies to test access control at the edge.

All examples provided here are licensed under the Apache 2.0 License.
