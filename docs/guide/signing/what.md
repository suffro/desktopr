---
title: App Signing Guide
description: Sign and distribute your apps with Desktopr.
next:
    text: MacOS Signing
    link: guide/signing/macos
---

# App Signing with Desktopr

Code signing is a security technology used to certify the authorship and integrity of software. Whether you are developing for macOS or Windows, signing your application is essential for a professional and secure distribution.

:::warning Note:
Desktopr builds unsigned apps happily — signing is optional and uses only your own certificates, kept as secrets in your repository. But for public distribution you want signed builds: unsigned apps are blocked or flagged on first launch.
:::

Signing in Desktopr is configured with repository secrets used by the **build** workflow:

| Purpose | Secrets |
| --- | --- |
| Windows code signing | `WINDOWS_CERTIFICATE` (base64 `.pfx`), `WINDOWS_CERTIFICATE_PASSWORD` |
| macOS code signing | `APPLE_CERTIFICATE` (base64 `.p12`), `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY` |
| macOS notarization | `APPLE_ID`, `APPLE_PASSWORD` (app-specific password), `APPLE_TEAM_ID` |

If a platform's secrets are missing, that build is simply unsigned; if a set is incomplete, the workflow fails before building. Credentials are never workflow inputs and are never printed.

## 1. Establishing Developer Trust
Digital signatures act as a virtual "ID card" for your software.
* **macOS:** Links the app to your Apple Developer ID.
* **Windows:** Identifies the legal entity or individual developer via a Certificate Authority (CA).
Without this, the operating system treats the software as coming from an "Unknown Developer," which can lead to users abandoning the installation.

## 2. Preventing Security Warnings
Both operating systems have strict security filters to protect users from malicious software.
* **macOS (Gatekeeper):** Unsigned apps are blocked by default. Users must go through deep system settings to bypass the warning, which creates a high barrier to entry.
* **Windows (SmartScreen):** When a user downloads an unsigned `.exe` or `.msi`, Windows SmartScreen displays a prominent blue warning: *"Windows protected your PC."* Signing your app (especially with an EV certificate) helps establish reputation and removes this warning.

## 3. Guaranteeing Software Integrity
A digital signature creates a cryptographic "seal" over your code.
* If a third party, a virus, or malware modifies even a single byte of your application after it has been signed, the signature becomes invalid.
* The operating system will then alert the user that the app has been tampered with, preventing the execution of compromised code.

## 4. Seamless User Experience
Signing ensures that the installation process is smooth and professional.
* **Notarization (macOS):** Once signed, apps are sent to Apple for notarization. This automated scan confirms the app is free of known malware, allowing it to run with a simple double-click.
* **Silent Updates:** Signed apps can often update themselves more easily, as the system recognizes the new version's signature as matching the previous one.

## 5. Access to System Features
Modern OS features are often restricted to signed applications:
* **macOS:** Required for iCloud, Push Notifications, and Hardened Runtime.
* **Windows:** Necessary for certain kernel-mode drivers and hardware integrations.



**Summary:** Signing your app is the only way to prove you are the author, ensure your code hasn't been altered, and provide your users with a warning-free installation experience.