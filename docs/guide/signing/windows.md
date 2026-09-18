---
title: Windows Code Signing
description: A guide to sign and distribute your Windows apps with Desktopr.
prev:
    text: MacOS Signing and Notarization
    link: guide/signing/macos
next:
    text: Microsoft Store and MSIX
    link: guide/signing/microsoft-store
---

# Windows Code Signing with Desktopr

This guide is for **independent Windows distribution**, outside the Microsoft Store. If you want to distribute through the Microsoft Store, use the MSIX / Store package flow instead.

## Why Windows signing matters

Unsigned Windows installers may show warnings such as **Unknown Publisher** or additional Windows security prompts. Signing your installer helps Windows display a verified publisher name and allows the file signature to be checked. Desktopr signs Windows builds with Tauri's SignTool-compatible signing during the build.


:::tip How to upload to Microsoft Store instead?
For Microsoft Store distribution, which does not require signing from your side, use the [MSIX / Microsoft Store](/guide/signing/microsoft-store.md) flow instead.

---
Store packages (`.msix`) use Store identity fields and are handled differently from independent `.exe` / `.msi` distribution.
:::

Official Microsoft docs:
- [SignTool](https://learn.microsoft.com/en-us/windows/win32/seccrypto/signtool)
- [Use SignTool to sign a file](https://learn.microsoft.com/en-us/windows/win32/seccrypto/using-signtool-to-sign-a-file)
- [Code signing options for Windows app developers](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options)

:::warning
Windows signing does not automatically remove every possible SmartScreen warning. Reputation, certificate trust, distribution channel, and Windows security settings can still affect what users see.
:::

## Step 1: Get a Windows code signing certificate

To sign a Windows app independently, you need a code signing certificate issued by a trusted provider.

Common options include:

1. **OV Code Signing Certificate** from a trusted Certificate Authority.
2. **EV Code Signing Certificate**, usually with stricter validation and hardware-backed key storage.
3. **Microsoft Trusted Signing / Azure Artifact Signing**, where available for your account type and region.

Official Microsoft docs:
- [Code signing options for Windows app developers](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options)
- [Sign your app for Smart App Control compliance](https://learn.microsoft.com/en-us/windows/apps/develop/smart-app-control/code-signing-for-smart-app-control)

> **Why this matters:** A self-signed certificate is useful for local testing, but it is not suitable for public distribution to normal users unless they explicitly trust your certificate on their machine.

## Step 2: Export or obtain a `.pfx` certificate file

Desktopr expects a `.pfx` file for Windows signing.

A `.pfx` file usually contains:

- the public certificate;
- the private key;
- optionally, intermediate certificate chain data.

Depending on your certificate provider, you may either download/export a `.pfx` file directly or receive instructions for using a hardware-backed certificate or cloud signing service.

:::warning
Keep your `.pfx` file private. Anyone with the `.pfx` file and its password may be able to sign Windows binaries as your publisher identity.
:::

## Step 3: Set a secure PFX password

When exporting or receiving a `.pfx`, it should be protected with a password.

Desktopr needs this password to unlock the certificate during the signing step.

This is **not** your Microsoft account password and **not** your Desktopr account password. It is the password specifically associated with the `.pfx` certificate file.

## Step 4: Sign your Windows build with Desktopr

Once you have the certificate file and password, you can fill the Windows signing fields in Desktopr and run the signing process.

Desktopr signs the Windows installer artifacts generated during the build, such as `.exe` and `.msi` files when they are present in the Windows bundle.

## Required Fields and Assets

### 1. Certificate (.pfx)

**What it is**

A `.pfx` file containing your Windows code signing certificate and private key.

Desktopr uses this file to sign Windows installer artifacts.

**How to get it**

You get it from your code signing certificate provider, or you export it from the certificate store if your provider allows export.

**Notes**

- The certificate must be valid for code signing.
- The private key must be available.
- For public distribution, use a certificate issued by a trusted provider.
- Keep the file private and store it securely.

Official Microsoft docs:
- [Use SignTool to sign a file](https://learn.microsoft.com/en-us/windows/win32/seccrypto/using-signtool-to-sign-a-file)
- [Code signing options for Windows app developers](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options)

### 2. Certificate Password

**What it is**

The password used to unlock the `.pfx` file.

**How to get it**

You create it when exporting the `.pfx`, or your certificate provider gives you the relevant setup/export instructions.

**Notes**

- This is not your Microsoft account password.
- If you lose it, you may need to re-export the certificate or follow your provider’s recovery process.
- Use a strong password and keep it secret.

### 3. Timestamping

**What it is**

Timestamping adds a trusted signing time to the signature.

This helps the signature remain verifiable even after the certificate itself expires, as long as the certificate was valid when the file was signed.

Microsoft explains that Authenticode timestamping allows signatures to remain verifiable after the signing certificate expires.

Official Microsoft docs:
- [Time Stamping Authenticode Signatures](https://learn.microsoft.com/en-us/windows/win32/seccrypto/time-stamping-authenticode-signatures)
- [SignTool](https://learn.microsoft.com/en-us/windows/win32/seccrypto/signtool)

**Notes**

- Desktopr uses a timestamp server during Windows signing.
- Timestamping is strongly recommended for production distribution.
- Your certificate provider may recommend a specific timestamp server.

## What Desktopr signs

Desktopr signs supported Windows installer files inside your Windows build bundle.

Typical Windows outputs may include:

| File type | Purpose |
| --------- | ------- |
| `.exe` | Windows setup installer |
| `.msi` | Windows installer package |

This Windows signing flow is not used for Microsoft Store MSIX distribution: Store-ready MSIX packages follow a separate identity and packaging process, and the Store signs them for you.

## Quick Setup Checklist

1. Buy or obtain a trusted Windows code signing certificate.
2. Export or receive the certificate as a `.pfx` file.
3. Save the `.pfx` password securely.
4. Base64-encode the `.pfx` and store it as the `WINDOWS_CERTIFICATE` repository secret.
5. Store the `.pfx` password as `WINDOWS_CERTIFICATE_PASSWORD`.
6. Run the **build** workflow: the Windows build is signed automatically when both secrets exist.
7. Test the signed installer on a clean Windows machine.

## Notes

:::tip MICROSOFT STORE
For Microsoft Store distribution, use the [MSIX / Microsoft Store](/guide/signing/microsoft-store.md) flow instead.
:::

:::warning
Signing improves trust and verifies publisher identity, but it does not guarantee that Windows SmartScreen will never show a warning. SmartScreen can also depend on file reputation, certificate reputation, download source, and user/device security settings.
:::
