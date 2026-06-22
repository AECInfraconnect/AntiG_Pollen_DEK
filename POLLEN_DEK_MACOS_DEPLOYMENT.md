# Pollen DEK — macOS Network Extension Deployment Guide

**Date:** June 21, 2026

This document outlines the strict deployment requirements for running the Pollen DEK `NEFilterDataProvider` Network Extension on macOS.

Unlike Linux (which only requires root/CAP_BPF) and Windows (which requires Admin rights for WFP), macOS enforces strict security guardrails around network traffic interception. This explains why the macOS implementation is currently treated as a "prepared prototype" for Phase 2.

## 1. Network Extension Type

Pollen DEK uses **`NEFilterDataProvider`** (part of the Content Filter provider framework).
This is the modern and Apple-recommended API for inspecting and dropping flows (replacing the deprecated `pf` interface). It allows:

- Transparent inspection of TCP, UDP, and other IP-level traffic.
- Inspecting traffic at the connection level (`handleNewFlow`) without breaking TLS or modifying the payload.
- As of macOS 26, supporting `NEURLFilterManager` to drop traffic based on URL paths efficiently.

## 2. Mandatory Security & Deployment Requirements

To deploy the extension, you **must** have all of the following:

### A. Developer Code Signing

The container app and the Network Extension must be signed using a valid **Apple Developer ID**. Ad-hoc signing will NOT work on production machines.

### B. Special Apple Entitlement

Network Extensions require special capabilities that are not granted by default. You must request and be approved for the following entitlement by Apple:

- `com.apple.developer.networking.networkextension`

*Note: This process requires an application to Apple detailing why your software needs to intercept network flows.*

### C. Application Notarization

Because the extension runs as part of the macOS system security layer, the entire bundle (Container App + Extension) must pass Apple's **Notarization** process to be distributed outside the Mac App Store.

### D. System Installation Approval (MDM)

Starting with modern macOS versions, users cannot simply "click" to approve a Network Extension without significant security friction. For enterprise deployments (which Pollen DEK targets), the extension must be deployed via **Mobile Device Management (MDM)**:

- A configuration profile containing a `SystemExtensions` payload (`com.apple.system-extension-policy`) must be pushed to allow the bundle identifier of the extension.
- A `WebContentFilter` payload configuration must be deployed to automatically activate the content filter without user interaction.

## 3. Communication (Container App ↔ Extension)

Because the extension runs in an isolated, sandboxed process managed by `sysextd`, communication with `dek-core` happens via **Inter-Process Communication (IPC)**:

- The macOS extension provides a Unix Domain Socket (e.g., `/var/run/pollen/nefilter.sock`) or an XPC connection.
- `dek-core` (`dek-macos-nefilter` client) pushes compiled network rules (Domain/IP/Port) as JSON payloads over this IPC mechanism.
- The extension returns telemetry (verdicts) through the same channel or a shared spool to ensure the AI agent's actions are fully observable.

## 4. Known Limitations & Research Notes

- **App Traffic Routing**: Some system traffic or traffic from browsers may bypass the data provider unless the configuration explicitly enables `filterBrowsers=true` and `filterSockets=true`.
- **Source IP Unknown**: In `handleNewFlow`, the initial source IP may appear as `0.0.0.0` or be unavailable. Policy enforcement should rely on destination hostname and port (`NWHostEndpoint`).
- **Cannot Modify Packets**: A Content Filter Provider can only return `.allow()` or `.drop()`. It cannot modify the payload or redirect traffic to a proxy.
