---
title: Desktopr Branding
description: Official Desktopr brand assets, colors, and basic usage guidelines.
next: 
  text: Get Started
  link: /guide/getting-started
---

# Desktopr Branding

Use these assets when referencing Desktopr in articles, documentation, integrations, marketplace listings, launch pages, or community posts.

## Brand color

Desktopr’s primary brand color is:

<div class="brand-color-card">
  <div class="brand-color-preview"></div>
  <div>
    <strong>#FD9A00</strong>
    <button class="copy-button" onclick="navigator.clipboard.writeText('#FD9A00').then(() => { const toast = document.querySelector('#primary-color-copied-toast'); toast.classList.add('is-visible'); clearTimeout(window.__desktoprCopyToastTimer); window.__desktoprCopyToastTimer = setTimeout(() => toast.classList.remove('is-visible'), 1400); });">Copy</button>
    <span class="copy-toast" id="primary-color-copied-toast">Copied!</span>
  </div>
</div>

<div class="brand-card">

<details>

<summary> CSS Properties</summary>

```css
--desktopr-primary: #FD9A00;
```

```css
--desktopr-primary-rgb: 253, 154, 0;
```

```css
color: #FD9A00;
```

```css
background-color: #FD9A00;
```

```css
border-color: #FD9A00;
```

```css
outline-color: #FD9A00;
```

```css
box-shadow: 0 0 0 3px rgba(var(--desktopr-primary-rgb), 0.25);
```

```css
text-decoration-color: #FD9A00;
```

```css
fill: #FD9A00;
```

```css
stroke: #FD9A00;
```

---

### Tailwind CSS

```js
// tailwind.config.js or .ts
export default {
  theme: {
    extend: {
      colors: {
        desktopr: {
          DEFAULT: '#FD9A00',
          50: '#FFF7E8',
          100: '#FFEBC2',
          200: '#FFD685',
          300: '#FFC047',
          400: '#FFAD1F',
          500: '#FD9A00',
          600: '#D98200',
          700: '#A86400',
          800: '#754600',
          900: '#472A00'
        }
      }
    }
  }
}
```

</details>

</div>

## Logo assets

### SVG icon

Use the SVG icon when you need a scalable mark, favicon, or compact brand symbol.

<div class="logo-card logo-card-icon">
  <img
    src="/public/assets/logo/logo.svg"
    alt="Desktopr SVG icon"
  />
</div>

<a href="/public/assets/logo/logo.svg" download="desktopr-logo.svg"> Download SVG logo </a>

### PNG logo

Use the PNG logo when you need the full Desktopr logo in presentations, landing pages, product listings, or visual previews.

<div class="logo-card logo-card-wide">
  <img
    src="/public/assets/logo/sizes/x150.png"
    alt="Desktopr PNG logo"
  />
</div>

#### Download PNG logo:

|**Sizes**|<center>x71</center>|<center>x150</center>|<center>x300</center>|<center>x512</center>|<center>x1024</center>|<center>x2048</center>|
|--|--|--|--|--|--|--|
|**Links**|<center> <a href="/public/assets/logo/sizes/x71.png" download="desktopr-logox71.png"> Download </a> </center>|<center> <a href="/public/assets/logo/sizes/x150.png" download="desktopr-logox150.png"> Download </a> </center>|<center> <a href="/public/assets/logo/sizes/x300.png" download="desktopr-logox300.png"> Download </a> </center>|<center> <a href="/public/assets/logo/sizes/x512.png" download="desktopr-logox512.png"> Download </a> </center>|<center> <a href="/public/assets/logo/sizes/x1024.png" download="desktopr-logox1024.png"> Download </a> </center>|<center> <a href="/public/assets/logo/sizes/x2048.png" download="desktopr-logox2048.png"> Download </a> </center>|

## Usage guidelines

When using the Desktopr brand, please keep the logo readable, clear, and visually separated from surrounding elements.

<div class="guidelines-grid">
  <div class="guideline-card">
    <strong>Use enough spacing</strong>
    <span>Leave clear space around the logo so it does not feel cramped.</span>
  </div>

  <div class="guideline-card">
    <strong>Keep proportions</strong>
    <span>Do not stretch, squeeze, rotate, or distort the logo.</span>
  </div>

  <div class="guideline-card">
    <strong>Use the official color</strong>
    <span>Use <code>#FD9A00</code> as the main accent color when referencing Desktopr.</span>
  </div>

  <div class="guideline-card">
    <strong>Prefer contrast</strong>
    <span>Place the logo on backgrounds where it remains easy to read.</span>
  </div>
</div>

## Short description

You can use this short description when linking to Desktopr:

<div class="brand-card">
  <div>
    <p>Desktopr turns your web app into a native desktop application for Windows, macOS, and Linux, giving you access to powerful native desktop capabilities. No lock-in, full ownership.</p>
    <button class="copy-button" onclick="navigator.clipboard.writeText('Desktopr turns your web app into a native desktop application for Windows, macOS, and Linux, giving you access to powerful native desktop capabilities. No lock-in, full ownership.').then(() => { const toast = document.querySelector('#short-description-copied-toast'); toast.classList.add('is-visible'); clearTimeout(window.__desktoprCopyToastTimer); window.__desktoprCopyToastTimer = setTimeout(() => toast.classList.remove('is-visible'), 1400); });">Copy</button>
    <span class="copy-toast" id="short-description-copied-toast">Copied!</span>
  </div>
</div>

## Longer description

A more detailed version for product pages, forums, blog posts, directories, press kits, partner listings etc.

<div class="brand-card">
  <div>
    <p>Desktopr is a desktop app builder that wraps your existing web application into a native Tauri-based desktop app for Windows, macOS, and Linux. It allows you to access powerful native desktop capabilities, providing package installers, and app distribution. No lock-in, full ownership.</p>
    <button class="copy-button" onclick="navigator.clipboard.writeText('Desktopr is a desktop app builder that wraps your existing web application into a native Tauri-based desktop app for Windows, macOS, and Linux. It allows you to access powerful native desktop capabilities, providing package installers, and app distribution. No lock-in, full ownership.').then(() => { const toast = document.querySelector('#long-description-copied-toast'); toast.classList.add('is-visible'); clearTimeout(window.__desktoprCopyToastTimer); window.__desktoprCopyToastTimer = setTimeout(() => toast.classList.remove('is-visible'), 1400); });">Copy</button>
    <span class="copy-toast" id="long-description-copied-toast">Copied!</span>
  </div>
</div>


## Embeddable badges

<br>

<a href="https://desktopr.dev" target="_blank" rel="noopener noreferrer" style="display:inline-flex;align-items:center;gap:14px;padding:14px 18px;min-width:120px;border:1px solid rgba(255,255,255,0.22);border-radius:18px;background:#1f1f1f;color:#ffffff;text-decoration:none;font-family:Inter,ui-sans-serif,system-ui,-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;">
  <span style="display:inline-flex;align-items:center;justify-content:center;width:34px;height:34px;background:transparent;overflow:hidden;">
    <img src="https://desktopr.dev/assets/logo/logo.svg" alt="Desktopr" style="width:100%;height:100%;object-fit:contain;display:block;" />
  </span>
  <span style="display:flex;flex-direction:column;line-height:1.05;">
    <span style="color:rgba(255,255,255,0.68);font-size:8px;font-weight:700;text-transform:uppercase;letter-spacing:0.02em;">Get started with</span>
    <span style="margin-top:5px;color:#ffffff;font-size:18px;font-weight:800;letter-spacing:-0.03em;">Desktopr</span>
  </span>
</a>

```html
<a href="https://desktopr.dev" target="_blank" rel="noopener noreferrer" style="display:inline-flex;align-items:center;gap:14px;padding:14px 18px;min-width:120px;border:1px solid rgba(255,255,255,0.22);border-radius:18px;background:#1f1f1f;color:#ffffff;text-decoration:none;font-family:Inter,ui-sans-serif,system-ui,-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;">
  <span style="display:inline-flex;align-items:center;justify-content:center;width:34px;height:34px;background:transparent;overflow:hidden;">
    <img src="https://desktopr.dev/assets/logo/logo.svg" alt="Desktopr" style="width:100%;height:100%;object-fit:contain;display:block;" />
  </span>
  <span style="display:flex;flex-direction:column;line-height:1.05;">
    <span style="color:rgba(255,255,255,0.68);font-size:8px;font-weight:700;text-transform:uppercase;letter-spacing:0.02em;">Get started with</span>
    <span style="margin-top:5px;color:#ffffff;font-size:18px;font-weight:800;letter-spacing:-0.03em;">Desktopr</span>
  </span>
</a>
```

---

<a href="https://desktopr.dev" target="_blank" rel="noopener noreferrer" style="margin-top:10px;display:inline-flex;align-items:center;gap:9px;padding:9px 13px;border:1px solid rgba(255,255,255,0.18);border-radius:999px;background:#1f1f1f;color:rgba(255,255,255,0.76);text-decoration:none;font-family:Inter,ui-sans-serif,system-ui,-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;font-size:13px;font-weight:500;line-height:1;">
  <img src="https://desktopr.dev/assets/logo/logo.svg" alt="Desktopr" style="width:18px;height:18px;object-fit:contain;display:block;" />
  <span>Powered by <strong style="color:#ffffff;font-weight:750;">Desktopr</strong></span>
</a>

```html
<a href="https://desktopr.dev" target="_blank" rel="noopener noreferrer" style="display:inline-flex;align-items:center;gap:9px;padding:9px 13px;border:1px solid rgba(255,255,255,0.18);border-radius:999px;background:#1f1f1f;color:rgba(255,255,255,0.76);text-decoration:none;font-family:Inter,ui-sans-serif,system-ui,-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;font-size:13px;font-weight:500;line-height:1;">
  <img src="https://desktopr.dev/assets/logo/logo.svg" style="width:18px;height:18px;object-fit:contain;display:block;" />
  <span>Powered by <strong style="color:#ffffff;font-weight:750;">Desktopr</strong></span>
</a>
```

<style>
.brand-color-card {
  display: flex;
  align-items: center;
  gap: 18px;
  padding: 18px;
  margin: 18px 0 28px;
  border: 1px solid var(--vp-c-divider);
  border-radius: 14px;
  background: var(--vp-c-bg-soft);
  position: relative;
}

.brand-card {
  display: flex;
  align-items: center;
  gap: 0px;
  padding: 18px;
  margin: 18px 0 18px;
  border: 1px solid var(--vp-c-divider);
  border-radius: 14px;
  background: var(--vp-c-bg-soft);
  position: relative;
}

.brand-color-preview {
  width: 72px;
  height: 72px;
  border-radius: 18px;
  background: #FD9A00;
  box-shadow: 0 10px 30px rgba(253, 154, 0, 0.28);
}

.brand-color-card strong {
  display: block;
  font-size: 1.35rem;
  line-height: 1.2;
}

.brand-color-card span {
  display: block;
  margin-top: 4px;
  color: var(--vp-c-text-2);
}

.logo-card {
  display: flex;
  align-items: center;
  justify-content: center;
  margin: 18px 0;
  padding: 28px;
  border: 1px solid var(--vp-c-divider);
  border-radius: 16px;
  background:
    linear-gradient(45deg, rgba(125, 125, 125, 0.08) 25%, transparent 25%),
    linear-gradient(-45deg, rgba(125, 125, 125, 0.08) 25%, transparent 25%),
    linear-gradient(45deg, transparent 75%, rgba(125, 125, 125, 0.08) 75%),
    linear-gradient(-45deg, transparent 75%, rgba(125, 125, 125, 0.08) 75%);
  background-size: 24px 24px;
  background-position: 0 0, 0 12px, 12px -12px, -12px 0;
}

.logo-card-icon img {
  width: 96px;
  height: 96px;
  object-fit: contain;
}

.logo-card-wide img {
  max-width: min(360px, 100%);
  height: auto;
  object-fit: contain;
}

.guidelines-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 14px;
  margin: 20px 0 30px;
}

.guideline-card {
  padding: 18px;
  border: 1px solid var(--vp-c-divider);
  border-radius: 14px;
  background: var(--vp-c-bg-soft);
}

.guideline-card strong {
  display: block;
  margin-bottom: 6px;
}

.guideline-card span {
  color: var(--vp-c-text-2);
}

.copy-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 2px 10px;
  border: 1px solid rgba(255,255,255,0.8);
  border-radius: 10px;
  background: var(--vp-c-bg-soft);
  color: rgba(255,255,255,1);
  font-size: 9pt;
  font-weight: 500;
  cursor: pointer;
  opacity: 0.5;
  margin-top: 10px;
}

.copy-button:hover {
  border-color: #FD9A00;
  color: #FD9A00;
  opacity: 1;
}

.copy-toast {
  position: absolute;
  top: 14px;
  right: 14px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 5px 9px;
  border-radius: 999px;
  background: #FD9A00;
  color: #111 !important;
  font-size: 9pt;
  font-weight: 500;
  opacity: 0;
  pointer-events: none;
  transform: translateY(-4px);
  transition: opacity 0.18s ease, transform 0.18s ease;
}

.copy-toast.is-visible {
  opacity: 1;
  transform: translateY(0);
}

@media (max-width: 640px) {
  .brand-color-card {
    align-items: flex-start;
    flex-direction: column;
  }

  .guidelines-grid {
    grid-template-columns: 1fr;
  }
}


</style>
