import { readFileSync } from "node:fs";
import { defineConfig, HeadConfig } from "vitepress";
import { markdownFileFor, recordPage, writeLlmsFiles } from "./llms.mjs";

const version = JSON.parse(
  readFileSync(new URL("../../sdk/package.json", import.meta.url), "utf8"),
).version;

const HOSTNAME = "https://desktopr.dev";
const REPOSITORY = "https://github.com/suffro/desktopr";

const softwareApplicationSchema = {
  "@context": "https://schema.org",
  "@graph": [
    {
      "@type": "SoftwareApplication",
      name: "Desktopr",
      description:
        "Easily build and sign lightweight desktop applications using web stacks, with Desktopr GitHub actions and a straightforward Node API.",
      applicationCategory: "DeveloperApplication",
      operatingSystem: "Windows, macOS, Linux",
      url: HOSTNAME,
      license: "https://www.apache.org/licenses/LICENSE-2.0",
      isAccessibleForFree: true,
      offers: {
        "@type": "Offer",
        price: "0",
        priceCurrency: "USD",
      },
      codeRepository: REPOSITORY,
      screenshot: `${HOSTNAME}/assets/images/companion-screenshot.png`,
      featureList: [
        "Multi-platform desktop builds (Windows, macOS, Linux)",
        "Web/Desktop Bridge API",
        "Code signing and notarization with your own certificates",
        "Sandboxed WebAssembly plugins",
        "Native notifications",
        "File system access",
        "System tray integration",
        "Deep links",
        "Global shortcuts",
        "Window management",
      ],
    },
    {
      "@type": "WebSite",
      name: "Desktopr",
      url: HOSTNAME,
    },
  ],
};

const faqPageSchema = {
  "@context": "https://schema.org",
  "@type": "FAQPage",
  mainEntity: [
    {
      "@type": "Question",
      name: "What is Desktopr?",
      acceptedAnswer: {
        "@type": "Answer",
        text: "Desktopr is an open source toolkit that turns an existing web app into an installable desktop application for Windows, macOS and Linux. You point it at your web app URL or bundle local files, configure your app details, and build the native wrapper yourself with Tauri.",
      },
    },
    {
      "@type": "Question",
      name: "Do I need to rebuild my app from scratch?",
      acceptedAnswer: {
        "@type": "Answer",
        text: "No. Desktopr wraps your existing web app. Your app keeps living on the web while Desktopr packages it inside a native desktop shell, and you can keep using SvelteKit, Next.js, React, Vue or any other web framework.",
      },
    },
    {
      "@type": "Question",
      name: "What platforms can I build for?",
      acceptedAnswer: {
        "@type": "Answer",
        text: "Desktopr builds desktop apps for Windows, macOS and Linux. The available package formats depend on the platform and on the signing options you configure.",
      },
    },
    {
      "@type": "Question",
      name: "Is Desktopr based on Tauri or Electron?",
      acceptedAnswer: {
        "@type": "Answer",
        text: "Desktopr uses Tauri, a desktop app framework built around Rust and the system webview. Apps are lighter than Electron-based wrappers because they do not bundle a full Chromium runtime.",
      },
    },
    {
      "@type": "Question",
      name: "Do users need to install Desktopr to use my app?",
      acceptedAnswer: {
        "@type": "Answer",
        text: "No. Your users install the desktop application you built, not Desktopr itself. The final binary is your application, with your app name, icon and configuration.",
      },
    },
    {
      "@type": "Question",
      name: "What does Desktopr cost?",
      acceptedAnswer: {
        "@type": "Answer",
        text: "Nothing. Desktopr is free and open source under the Apache 2.0 license. You build it yourself, on your machine or with GitHub Actions, and there is no account, backend or hosted service involved.",
      },
    },
    {
      "@type": "Question",
      name: "Can my web app access native desktop features?",
      acceptedAnswer: {
        "@type": "Answer",
        text: "Yes. Desktopr provides a Bridge API that lets your web app talk to the native wrapper: notifications, deep links, window controls, tray and menus, file system access, diagnostics, networking, WebAssembly plugins and more.",
      },
    },
    {
      "@type": "Question",
      name: "What is Desktopr Companion?",
      acceptedAnswer: {
        "@type": "Answer",
        text: "Desktopr Companion is a desktop app built with Desktopr that you can use to try the bridge before building your own app. It loads any web app URL in a Desktopr window and includes a playground for the Bridge API.",
      },
    },
    {
      "@type": "Question",
      name: "Do I need to sign my desktop app?",
      acceptedAnswer: {
        "@type": "Answer",
        text: "For public distribution, yes. Unsigned apps trigger operating system warnings during download, installation or first launch. Signing identifies you as the publisher and is expected for a professional distribution experience.",
      },
    },
    {
      "@type": "Question",
      name: "Can Desktopr sign my app?",
      acceptedAnswer: {
        "@type": "Answer",
        text: "Yes. The build workflow signs Windows and macOS builds, and notarizes macOS builds, when you provide your own certificates and credentials as repository secrets. Without them the build is simply unsigned.",
      },
    },
    {
      "@type": "Question",
      name: "Can I distribute an unsigned app?",
      acceptedAnswer: {
        "@type": "Answer",
        text: "Yes, but it is mainly suitable for testing, internal previews and early builds. For public distribution users would see warnings from Windows SmartScreen, macOS Gatekeeper, browsers or antivirus tools.",
      },
    },
  ],
};

const head: HeadConfig[] = [
  ["link", { rel: "icon", href: "/favicon.svg" }],
  ["meta", { property: "og:site_name", content: "Desktopr" }],
  ["meta", { property: "og:type", content: "website" }],
  ["meta", { name: "twitter:card", content: "summary_large_image" }],
];

// https://vitepress.dev/reference/site-config
export default defineConfig({
  lang: "en-US",
  head,
  title: "Desktopr",
  description:
    "Easily build and sign lightweight desktop applications using web stacks, with Desktopr GitHub actions and a straightforward Node API.",
  cleanUrls: true,
  sitemap: {
    hostname: HOSTNAME,
  },
  transformHead({ pageData }) {
    // Recording the page is what feeds llms.txt and the Markdown twins; see ./llms.mjs.
    const route = recordPage(pageData);
    const canonicalUrl = `${HOSTNAME}${route}`;

    const heads: HeadConfig[] = [
      ["link", { rel: "canonical", href: canonicalUrl }],
      // The Markdown twin of this page, discoverable without asking for it. `functions/` serves the
      // same file to anything sending `Accept: text/markdown`, and `PageActions.vue` reads this
      // link rather than deriving the path again.
      [
        "link",
        {
          rel: "alternate",
          type: "text/markdown",
          href: `${HOSTNAME}/${markdownFileFor(route)}`,
        },
      ],
      ["meta", { property: "og:url", content: canonicalUrl }],
    ];

    if (pageData.relativePath === "index.md") {
      heads.push([
        "script",
        { type: "application/ld+json" },
        JSON.stringify(softwareApplicationSchema),
      ]);
    }

    if (pageData.relativePath === "support.md") {
      heads.push([
        "script",
        { type: "application/ld+json" },
        JSON.stringify(faqPageSchema),
      ]);
    }

    return heads;
  },
  // The Markdown surface, written after the pages are rendered: /llms.txt, /llms-full.txt and one
  // `.md` twin per page. See ./llms.mjs.
  async buildEnd(siteConfig) {
    const written = await writeLlmsFiles({
      outDir: siteConfig.outDir,
      srcDir: siteConfig.srcDir,
      hostname: HOSTNAME,
      version,
      sidebar: siteConfig.site.themeConfig.sidebar,
      siteDescription: siteConfig.site.description,
    });
    console.log(
      `generated llms.txt (${written.indexed} pages), llms-full.txt and ${written.twins} Markdown page twins`,
    );
  },

  appearance: {
    initialValue: "dark",
  },
  themeConfig: {
    logo: "/assets/logo/logo.svg",
    siteTitle: "Desktopr",

    search: {
      provider: "local",
    },
    nav: [
      { text: "Guide", link: "/guide/getting-started" },
      {
        text: "Resources",
        items: [
          { text: "Bridge API", link: "/guide/bridge/overview" },
          { text: "Companion", link: "/guide/companion" },
          { text: "Plugins", link: "/guide/bridge/api/plugins" },
          { text: "Signing", link: "/guide/signing/what" },
          { text: "Microsoft Store", link: "/guide/signing/microsoft-store" },
          { text: "Bubble.io", link: "/guide/bubble-plugin" },
          { text: "Comparison", link: "/comparison" },
          { text: "FAQ", link: "/support" },
        ],
      },
      { text: "GitHub", link: REPOSITORY, noIcon: true, target: "_blank" },
    ],
    footer: {
      message: `Released under the <a href="${REPOSITORY}/blob/main/LICENSE">Apache 2.0 License</a>.`,
      copyright: `© ${new Date().getFullYear()} Desktopr contributors`,
    },
    sidebar: [
      {
        text: "Introduction",
        collapsed: false,
        items: [
          { text: "Getting Started", link: "/guide/getting-started" },
          { text: "What is Desktopr", link: "/guide/" },
          {
            text: "Architecture Overview",
            link: "/guide/architecture-overview",
          },
          { text: "Tauri", link: "/guide/what-is-tauri" },
          { text: "Plugin Development", link: "/guide/plugin-development" },
          { text: "Branding", link: "/branding" },
        ],
      },
      {
        text: "Try Desktopr",
        collapsed: false,
        items: [{ text: "Companion", link: "/guide/companion" }],
      },
      {
        text: "Configuration",
        collapsed: true,
        items: [
          { text: "Project Structure", link: "/guide/configuration/project-structure" },
          { text: "App Settings", link: "/guide/configuration/app-settings" },
          { text: "Tauri Config", link: "/guide/configuration/tauri-config" },
          { text: "Distribution", link: "/guide/configuration/distribution" },
        ],
      },
      {
        text: "Signing",
        collapsed: true,
        items: [
          { text: "What is Signing?", link: "/guide/signing/what" },
          { text: "macOS", link: "/guide/signing/macos" },
          { text: "Windows", link: "/guide/signing/windows" },
          { text: "Microsoft Store", link: "/guide/signing/microsoft-store" },
        ],
      },
      {
        text: "Bridge API",
        link: "/guide/bridge/overview",
        collapsed: false,
        items: [
          { text: "Get Started", link: "/guide/bridge/overview" },
          {
            text: "Modules",
            items: [
              { text: "App", link: "/guide/bridge/api/app" },
              { text: "Plugins", link: "/guide/bridge/api/plugins" },
              { text: "Badge", link: "/guide/bridge/api/badge" },
              { text: "Menu", link: "/guide/bridge/api/menu" },
              { text: "Events", link: "/guide/bridge/api/events" },
              { text: "File System", link: "/guide/bridge/api/file-system" },
              { text: "Diagnostics", link: "/guide/bridge/api/diagnostics" },
              { text: "Shortcuts", link: "/guide/bridge/api/shortcuts" },
              { text: "Deeplinks", link: "/guide/bridge/api/deep-links" },
              { text: "Notifications", link: "/guide/bridge/api/notifications" },
              { text: "Files", link: "/guide/bridge/api/files" },
              { text: "Drag and Drop", link: "/guide/bridge/api/drag-and-drop" },
              { text: "Network", link: "/guide/bridge/api/network" },
              { text: "Window", link: "/guide/bridge/api/window" },
              { text: "Autostart", link: "/guide/bridge/api/autostart" },
              { text: "Clipboard", link: "/guide/bridge/api/clipboard" },
              { text: "Utilities", link: "/guide/bridge/api/utilities" },
            ],
          },
        ],
      },
      {
        text: "No-code Integrations",
        collapsed: false,
        items: [{ text: "Bubble.io", link: "/guide/bubble-plugin" }],
      },
      { text: "FAQ", link: "/support" },
      { text: "Comparison", link: "/comparison" },
    ],

    socialLinks: [{ icon: "github", link: REPOSITORY }],
  },
});
