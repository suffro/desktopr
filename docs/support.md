---
title: FAQ
description: Frequently asked questions about Desktopr, desktop builds, native features, signing, distribution, and ownership.
sidebar: false
prev: false
next: false
head:
  - - meta
    - property: og:title
      content: "FAQ – Desktopr"
  - - meta
    - property: og:description
      content: "Frequently asked questions about Desktopr, desktop builds, native features, code signing, distribution, and app ownership."
  - - meta
    - property: og:image
      content: "https://desktopr.app/assets/logo/logo-color.png"
  - - meta
    - name: twitter:title
      content: "FAQ – Desktopr"
  - - meta
    - name: twitter:description
      content: "Frequently asked questions about Desktopr, desktop builds, native features, code signing, distribution, and app ownership."
---

# FAQ

Answers to common questions about Desktopr, desktop builds, native features, signing, updates, and distribution.

> You can also check our guide [here](/guide/).

## General

<details>
<summary>What is Desktopr?</summary>

Desktopr is a desktop app builder that turns an existing web app into an installable desktop application for Windows, macOS, and Linux.

You provide the URL of your web app, configure your app details, and Desktopr generates a native desktop wrapper around it using Tauri.

> Check out our guide [here](/guide/).


---
</details>

<details>
<summary>Do I need to rebuild my app from scratch?</summary>

No!

Desktopr is designed to work with your existing web app. Your app continues to live on the web, while Desktopr packages it inside a native desktop shell.

This means you can keep using the tools you already use, such as Bubble, SvelteKit, Next.js, React, Vue, or other web frameworks.


---
</details>

<details>
<summary>Is Desktopr supported for any kind of web apps?</summary>

Yes!

Desktopr can work with any public web app URL, as long as the app can be loaded inside the desktop wrapper, no metter the framework or underling stack.

</details>

<details>
<summary>What platforms can I build for?</summary>

Desktopr is designed to generate desktop apps for:

- Windows
- macOS
- Linux

The final available formats may depend on the platform, build configuration, and signing options selected for your app.


---
</details>

<details>
<summary>Is Desktopr based on Tauri or Electron?</summary>

Desktopr uses Tauri, which is a modern desktop app framework built around Rust and the system webview.

This usually allows apps to be lighter than traditional Electron-based wrappers, because the app does not need to bundle a full Chromium runtime.

> Check out our guide [here](/guide/what-is-tauri).

---
</details>

<details>
<summary>Do users need to install Desktopr to use my app?</summary>

No.

Your users install your generated desktop application, not Desktopr itself.

Desktopr is the builder used to create your app. The final desktop app belongs to you and can be distributed under your own brand.

</details>

## App ownership

<details>
<summary>Who owns the generated desktop app?</summary>

You, and only you.

Desktopr generates a custom desktop wrapper for your web app. The final binary is your application, with your app name, icon, configuration, and distribution settings.


---
</details>

<details>
<summary>Can I distribute the app myself?</summary>

Yes.

You can distribute the generated desktop app through your own website, private channels, customer portals, or other distribution methods.

Depending on your platform and use case, you may also prepare the app for more formal distribution channels.


---
</details>

<details>
<summary>Does Desktopr host my web app?</summary>

No.

Desktopr does not replace your existing hosting. Your web app remains hosted wherever it already lives.

The desktop app loads your configured app URL inside a native desktop wrapper.


---
</details>

<details>
<summary>What happens when I update my web app?</summary>

Since the desktop app loads your existing web app URL, normal web app changes can become available without rebuilding the desktop app.

However, if you change desktop-specific configuration, app metadata, native permissions, signing, icons, or wrapper-level features, you may need to create a new desktop build.

</details>

## Native features

<details>
<summary>Can my web app access native desktop features?</summary>

Yes.

Desktopr provides a Bridge API that lets your web app communicate with the native desktop wrapper.

Through the bridge, your app can access desktop-native functionality such as notifications, deep links, window controls, tray/menu features, file system access, diagnostics, and other native modules.

> Check out our guide [here](/guide/bridge/overview).

---
</details>

<details>
<summary>What are the available native features?</summary>

You can check the full list of Desktopr features [here](/guide/bridge/overview#modules).

---
</details>

<details>
<summary>How does the Bridge API work?</summary>

Your web app can call Desktopr APIs from JavaScript.

In the desktop environment, these calls are handled by the native Tauri wrapper. In the browser, you should check whether the Desktopr bridge is available before calling desktop-only functionality.

> Check out our guide [here](/guide/bridge/overview).

---
</details>

<details>
<summary>Can I use Desktopr native features from Bubble.io?</summary>

Yes.

Desktopr is designed to support Bubble apps through a Bubble plugin and bridge integration.

This allows Bubble builders to call native desktop features without writing Rust or Tauri code.

> Check out our guide [here](/guide/bubble-plugin).

---
</details>

<details>
<summary>What happens if users open my web app in a normal browser?</summary>

Desktop-only APIs are only available inside the generated Desktopr app.

If your app is opened in a normal browser, the Desktopr bridge may not be available. Your app should handle this gracefully, for example by hiding desktop-only buttons or showing a message when a feature requires the desktop app.

</details>

## Desktopr Companion

> You can also check our guide [here](/guide/companion).

<details>
<summary>What is Desktopr Companion?</summary>

Desktopr Companion is a helper app used during development to test and preview Desktopr Bridge features before shipping a production desktop build.

It lets you open your web app inside a Desktopr-compatible desktop environment, so you can check how native features behave outside the browser.

> Check out our guide [here](/guide/companion).
---
</details>

<details>
<summary>Do I need Desktopr Companion to distribute my app?</summary>

No.

Desktopr Companion is mainly a development and testing tool.

Your final users do not need to install Desktopr Companion. They install your generated desktop app.

---
</details>

<details>
<summary>Why should I use Desktopr Companion?</summary>

Desktopr Companion is useful when you want to test desktop-only features while you are still developing your web app.

For example, you can use it to test Bridge API calls, notifications, desktop detection, deep links, window behavior, and other native integrations before creating a final build.

> Check out our guide [here](/guide/companion).
---
</details>

<details>
<summary>Is Desktopr Companion the same as my generated app?</summary>

No.

Desktopr Companion is a generic testing environment.

Your generated app is the final desktop application built with your own app name, icon, configuration, signing settings, and distribution setup.

</details>

## Signing and security

> You can also check our guide [here](/guide/signing/what).

<details>
<summary>Do I need to sign my desktop app?</summary>

For production distribution, yes.

Unsigned apps can still be useful for internal testing, but they may trigger operating system warnings during download, installation, or first launch.

Signing helps users and operating systems identify the publisher of the app, and it is usually required for a professional distribution experience.

---
</details>

<details>
<summary>Can Desktopr sign my app?</summary>

Yes, Desktopr is designed to support signing workflows for generated desktop apps.

However, signing requires platform-specific certificates and credentials owned by you or your organization.

Desktopr can help automate the signing process, but you still need to provide the required certificate files, passwords, identities, or developer account credentials depending on the platform.

---
</details>

<details>
<summary>Can I distribute an unsigned app?</summary>

Yes, but it is mainly recommended for testing, internal previews, or early development builds.

For public distribution, unsigned apps are not ideal because users may see warnings from Windows SmartScreen, macOS Gatekeeper, browsers, antivirus tools, or other security systems.

---
</details>

<details>
<summary>What happens if my Windows app is unsigned?</summary>

Windows may show warnings when users download, install, or launch the app.

The app may still be installable, but users might see messages from Windows SmartScreen or browser security checks saying that the publisher is unknown or that the app is not commonly downloaded.

This can make the app feel less trustworthy, especially for non-technical users.

---
</details>

<details>
<summary>What do I need to sign a Windows app?</summary>

To sign a Windows desktop app, you usually need a valid code signing certificate issued by a certificate authority.

Desktopr can use your signing certificate during the build or signing process so that the generated Windows artifacts are signed under your publisher identity.

Depending on your certificate provider, you may receive the certificate as a `.pfx` file or through a hardware/cloud signing method.

---
</details>

<details>
<summary>Does Windows signing remove all SmartScreen warnings?</summary>

No.

Signing helps identify the publisher and improves trust, but Windows SmartScreen may still show warnings for new apps, new certificates, rarely downloaded files, or apps without enough reputation.

Signing is still strongly recommended for production, but SmartScreen reputation can take time to build.

---
</details>

<details>
<summary>What happens if my macOS app is unsigned?</summary>

macOS may block or warn users when they try to open the app.

Unsigned or unnotarized apps can trigger Gatekeeper warnings, and users may need to manually override macOS security protections to launch the app.

This is not ideal for public distribution.

---
</details>

<details>
<summary>What do I need to sign and notarize a macOS app?</summary>

For distribution outside the Mac App Store, you usually need an Apple Developer account, a Developer ID Application certificate, and access to Apple notarization credentials.

Desktopr can use these credentials to sign and notarize the generated macOS app, so users can open it with fewer security warnings.

---
</details>

<details>
<summary>What is macOS notarization?</summary>

Notarization is Apple’s security review process for macOS software distributed outside the Mac App Store.

After the app is signed, it can be submitted to Apple for notarization. If accepted, the notarization ticket can be attached to the app so macOS can recognize it as checked by Apple.

---
</details>

<details>
<summary>Is signing the same as notarization on macOS?</summary>

No.

Signing identifies the developer and confirms that the app has not been modified after signing.

Notarization is an additional Apple process that checks the signed app before distribution outside the Mac App Store.

For a smoother macOS installation experience, production apps should generally be both signed and notarized.

---
</details>

<details>
<summary>Can Desktopr help with macOS notarization?</summary>

Yes.

Desktopr is designed to support macOS signing and notarization workflows using your Apple Developer credentials.

You provide the required certificates and notarization credentials, and Desktopr handles the technical signing/notarization steps during the build or release process.

---
</details>

<details>
<summary>Do I need different signing credentials for Windows and macOS?</summary>

Yes.

Windows and macOS use different signing systems.

Windows requires a Windows code signing certificate, while macOS requires Apple Developer certificates and notarization credentials.

These credentials are separate and must be obtained from the appropriate providers.

---
</details>

<details>
<summary>Do I need signing for Linux builds?</summary>

Linux distribution usually works differently from Windows and macOS.

Depending on the package format and distribution method, you may not need the same kind of code signing certificate. However, some package repositories, update systems, or enterprise environments may still require signed packages or trusted repositories.

---
</details>

<details>
<summary>Can I publish my Desktopr app to the Mac App Store?</summary>

Desktopr is primarily designed for independent desktop distribution outside the Mac App Store.

Publishing to the Mac App Store may require additional Apple-specific packaging, sandboxing, entitlement configuration, App Store review requirements, and compliance work.

---
</details>

<details>
<summary>Can I publish my Desktopr app to the Microsoft Store?</summary>

Yes.

Desktopr can generate an optional MSIX package for Microsoft Store submission when the required Microsoft Store identity fields are provided in your app settings.

The normal Windows installer is still useful for direct distribution, while the MSIX package is intended for Microsoft Store publishing.

> You can also check our guide [here](/guide/signing/microsoft-store).
---
</details>

<details>
<summary>What is MSIX?</summary>

MSIX is Microsoft’s modern packaging format for Windows apps.

For Desktopr, MSIX is useful when you want to submit your Windows app to Microsoft Partner Center and distribute it through the Microsoft Store.

---
</details>

<details>
<summary>When does Desktopr generate an MSIX package?</summary>

Desktopr generates the MSIX package only when the Microsoft Store identity fields are filled in.

If those fields are empty, Desktopr skips MSIX generation and continues with the normal Windows build.

---
</details>

<details>
<summary>Where do I get the Microsoft Store identity values?</summary>

You must get them from Microsoft Partner Center.

Create or reserve your app in Partner Center, then copy the identity values from the product identity page into Desktopr.

Do not guess or manually invent these values. They must match the identity assigned by Microsoft Partner Center.

---
</details>

<details>
<summary>Which Microsoft Store fields does Desktopr use?</summary>

Desktopr can use fields such as:

- `store_identity_name`
- `store_identity_publisher`
- `store_publisher_display_name`
- `store_package_display_name`
- `store_logo_background_color`
- `store_short_name`

These values are used to generate the MSIX package identity, publisher information, display name, visual configuration, and Store-compatible metadata.
You can set these values in you app page.

---
</details>

<details>
<summary>Do I need a Windows code signing certificate for Microsoft Store distribution?</summary>

For normal direct distribution outside the Store, a Windows code signing certificate is usually recommended.

For Microsoft Store MSIX distribution, Microsoft signs the package as part of the Store certification and publishing flow.

---
</details>

<details>
<summary>Is the MSIX package used for Desktopr auto-updates?</summary>

No.

The MSIX package is intended for Microsoft Store submission.

Desktopr treats the MSIX output separately from the normal Windows installer and updater distribution flow.

---
</details>

<details>
<summary>What happens if MSIX generation fails?</summary>

If the traditional Windows installer was generated successfully, the regular Windows build is still usable.

MSIX generation may fail or be skipped because of missing Store identity fields, incorrect identity values, invalid version format, missing Windows SDK packaging tools, or invalid manifest/visual asset configuration.

---
</details>

<details>
<summary>Should I sign test builds?</summary>

Not always.

For quick internal testing, unsigned builds can be acceptable.

For client demos, beta testers, or public downloads, signing is recommended because security warnings can confuse users and reduce trust.

</details>


## Distribution

<details>
<summary>How do I distribute my app?</summary>

However you like.

Desktopr builds the installers and stops there. You can publish them as GitHub Release assets, host them on your own website or CDN, push them to an internal file server, or submit them to a store. Nothing is uploaded to a Desktopr server, because there is no Desktopr server.

See the [distribution guide](/guide/configuration/distribution).

---
</details>

<details>
<summary>Can I publish to the Microsoft Store?</summary>

Yes.

The build workflow can produce an MSIX package with the Store identity from your Partner Center account, which you then upload to Partner Center. The Store signs it for you. See [Microsoft Store](/guide/signing/microsoft-store).

---
</details>

<details>
<summary>Can my app update itself?</summary>

Yes, if you host the update manifest.

The updater is disabled by default and is not compiled in. Enable it by providing your own HTTPS update endpoint and signing key, as described in [App Settings](/guide/configuration/app-settings#updates).

---
</details>

<details>
<summary>Can I distribute unsigned builds?</summary>

Yes, but mainly for testing and internal previews.

Unsigned builds work, but users see warnings from SmartScreen or Gatekeeper on first launch. For public distribution, sign your builds with your own certificates.

---
</details>

## Questions and bug reports

Desktopr is maintained on GitHub. Open an [issue](https://github.com/suffro/desktopr/issues) with your platform, the Desktopr version or commit and steps to reproduce.

Security problems must be reported privately, as described in [SECURITY.md](https://github.com/suffro/desktopr/blob/main/SECURITY.md).
