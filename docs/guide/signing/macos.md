---
title: MacOS Signing and Notarization
description: A guide to sign and distribute your MacOS apps with Desktopr.
prev:
    text: Why do I need this?
    link: guide/signing/what
next:
    text: Windows Signing
    link: guide/signing/windows
---


# MacOS Signing with Desktopr

On macOS, unsigned apps are blocked by default by Gatekeeper. Users must go through deep system settings to bypass the warning, which creates a high barrier to entry, that's why signing your app is essential in order to distribute it.

## Step 1: Generate a CSR on your Mac
1. Open **Keychain Access**.
2. Navigate to **Menu** > **Keychain Access** > **Certificate Assistant** > **Request a Certificate From a Certificate Authority…**
3. Fill in the following details:
    * **User Email Address:** Your email.
    * **Common Name:** A recognizable name (e.g., *Desktopr Developer ID*).
    * **Request is:** Select **Saved to disk**.
4. Save the `.certSigningRequest` file (CSR).

> **Why this matters:** The CSR step creates the private key in your Keychain. You need that private key later to export a working .p12 file.

## Step 2: Create "Developer ID Application" in Apple Developer Portal
1. Go to **Apple Developer** > **Certificates** and click the **+** (plus) icon.
2. Select **Developer ID Application**.
3. When prompted, upload the **CSR** you generated in Step 1.
4. Download the resulting `.cer` certificate.

## Step 3: Install the Certificate on your Mac
1. Double-click the downloaded `.cer` file to add it to your Keychain.
2. In **Keychain Access** > **My Certificates**, confirm you see:
    * `Developer ID Application: [Name]` with a **private key** nested underneath.

## Step 4: Get Credentials and Assets

### 1. Certificate (.p12)
**What it is**
A .p12 file containing your macOS Developer ID certificate and its associated private key. It is used to sign the app during the build process.

**How to get it**
Open **Keychain Access** > **My Certificates** > Select **Developer ID Application** (ensure the private key is selected) > **Export…** > Choose **.p12** format.

**Notes**
* The certificate must appear under "My Certificates" with a private key; otherwise, the export will not work for signing.
* Keep this file private. Anyone with the .p12 and its password can sign apps as you.

### 2. Certificate Password
**What it is**
The password you set manually when exporting the .p12 from Keychain Access.

**How to get it**
You create it at the moment of export. If you do not remember it, you must export a new .p12 and set a new password.

**Notes**
* This is **not** your Apple ID password.
* Use a strong password and store it securely.

### 3. Dev ID Application
**What it is**
The exact signing identity name for your Developer ID Application certificate (the text label). This tells the signing tool which certificate to use.

**Expected Format**
`Developer ID Application: Your Name (TEAMID)`

**How to find it**
Run the following command in Terminal:
`security find-identity -v`

**Notes**
* Must match exactly (case, spaces, and punctuation).
* This field is for signing .app and .dmg files. It is different from "Developer ID Installer".

### 4. Team ID
**What it is**
Your 10-character Apple Developer Team ID (e.g., UC22LVXXXX).

**How to find it**
* In the **Apple Developer Account** membership details.
* Inside the signing identity string parentheses, e.g., (UC22LVXXXX).

### 5. Apple ID (Email)
**What it is**
The Apple ID email used for notarization (typically your Developer account email).

**Notes**
* Used for notarization authentication.
* It must have access to the developer team.

### 6. App-Specific Password
**What it is**
A unique password generated in your Apple ID account for automated services.

**How to create it**
Go to **Apple ID Account Settings** > **Sign-In and Security** > **App-Specific Passwords** > Generate a new one.

**Notes**
* Required for non-interactive notarization (CI/CD or build servers).
* If lost, you must generate a new one.

## Step 5: Sign your app

Store these elements as secrets of the repository that runs the build workflow
(**Settings → Secrets and variables → Actions**). They are never workflow inputs
and are never printed in the logs.

| Secret | Value |
| --- | --- |
| `APPLE_CERTIFICATE` | your Developer ID Application `.p12`, base64 encoded |
| `APPLE_CERTIFICATE_PASSWORD` | the password you set when exporting the `.p12` |
| `APPLE_SIGNING_IDENTITY` | the identity string, e.g. `Developer ID Application: Your Name (TEAMID)` |
| `APPLE_ID` | your Apple ID email |
| `APPLE_PASSWORD` | an app-specific password |
| `APPLE_TEAM_ID` | your Team ID, e.g. `UC22LVXXXX` |

## Quick Setup Checklist
1. **Export** Developer ID Application as a `.p12` from Keychain, then
   `base64 -i certificate.p12 | pbcopy` and store it as `APPLE_CERTIFICATE`.
2. **Store** the export password as `APPLE_CERTIFICATE_PASSWORD`.
3. **Copy** the identity string from `security find-identity -v` into `APPLE_SIGNING_IDENTITY`.
4. **Store** your **Team ID** as `APPLE_TEAM_ID`.
5. **Store** your **Apple ID email** as `APPLE_ID`.
6. **Generate** an **app-specific password** and store it as `APPLE_PASSWORD`.

With the three signing secrets the app is signed; adding the three notarization
secrets also notarizes and staples it, so it opens without any Gatekeeper prompt.

:::tip
Notarization needs an active Apple Developer Program membership and an accepted
Program License Agreement. Apple answers `403 ... a required agreement is
missing or has expired` both when an agreement is really pending **and** when the
app-specific password is no longer valid — regenerate the password before
hunting for a missing agreement.
:::

## Apple Resources

:::info Signing
You can find the **Apple** official guide for **app signing** [here](https://support.apple.com/en-gb/guide/security/sec3ad8e6e53/web).
:::

:::info Notarization
You can find the **Apple** official guide for **app notarization** [here](https://developer.apple.com/videos/play/wwdc2019/703/).
:::

:::info App-Specific Passwords
You can find the **Apple** official guide on how to get your **app-specific password** [here](https://support.apple.com/en-us/102654).
:::

:::info Developer ID Certificate
You can find the **Apple** official guide on how to get your **developer id certificate** [here](https://developer.apple.com/help/account/certificates/create-developer-id-certificates).
:::