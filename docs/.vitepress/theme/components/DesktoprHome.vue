<template>
  <div class="desktopr-home">
    
    <!-- Hero Section -->
    <section class="hero-section">
      <div class="hero-glow"></div>
      
      <h1 class="hero-title">
        Build <span class="text-gradient">desktop</span> apps <br> with <span class="text-gradient">web</span> stacks.
      </h1>
      
      <p class="hero-tagline">
        Easily build and sign lightweight desktop applications using web stacks,<br class="hidden-mobile" />
        with Desktopr GitHub actions and a straightforward Node API.
      </p>
      
      <div class="hero-actions">
        <a href="/guide/getting-started" class="btn btn-primary">Get Started</a>
        <a href="/guide/bridge/overview" class="btn btn-secondary">Bridge API</a>
        <a href="https://github.com/suffro/desktopr" target="_blank" class="btn btn-secondary">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor"><path d="M12 2C6.477 2 2 6.477 2 12c0 4.42 2.865 8.166 6.839 9.489.5.092.682-.217.682-.482 0-.237-.008-.866-.013-1.7-2.782.603-3.369-1.34-3.369-1.34-.454-1.156-1.11-1.462-1.11-1.462-.908-.62.069-.608.069-.608 1.003.07 1.531 1.03 1.531 1.03.892 1.529 2.341 1.087 2.91.831.092-.646.35-1.086.636-1.336-2.22-.253-4.555-1.11-4.555-4.943 0-1.091.39-1.984 1.029-2.683-.103-.253-.446-1.27.098-2.647 0 0 .84-.269 2.75 1.025A9.578 9.578 0 0112 6.836c.85.004 1.705.114 2.504.336 1.909-1.294 2.747-1.025 2.747-1.025.546 1.377.203 2.394.1 2.647.64.699 1.028 1.592 1.028 2.683 0 3.842-2.339 4.687-4.566 4.935.359.309.678.919.678 1.852 0 1.336-.012 2.415-.012 2.743 0 .267.18.578.688.48C19.138 20.161 22 16.418 22 12c0-5.523-4.477-10-10-10z"/></svg>
          GitHub
        </a>
      </div>
    </section>

    <!-- Main Features -->
    <section class="main-features">
      <div class="feature-card" v-for="feature in mainFeatures" :key="feature.title">
        <h3>{{ feature.title }}</h3>
        <p>{{ feature.details }}</p>
      </div>
    </section>

    <!-- Native Bridge Section -->
    <section class="native-bridge">
      <div class="bridge-header">
        <h2>Native Bridge Features</h2>
        <p>The native bridge provides a set of powerful features to enhance your desktop app experience by integrating deeply with the operating system.</p>
      </div>
      
      <div class="bento-grid">
        <div class="bento-card" v-for="feat in bridgeFeatures" :key="feat.name">
          <div class="bento-icon">{{ feat.icon }}</div>
          <h4>{{ feat.name }}</h4>
          <p>{{ feat.desc }}</p>
        </div>
      </div>
    </section>

    <!-- CTA Section -->
    <section class="cta-section">
      <div class="cta-content">
        <h2 style="border: none; padding: 0; margin-top: 15px;">Go Desktop</h2>
        <p>Desktopr lets creators, developers and teams bring their web experiences to every desktop. Clone the repository, point it at your web app and build — the toolchain is yours to keep.</p>
        <a href="/guide/getting-started" class="btn btn-primary">Quickstart</a>
      </div>
    </section>
  </div>
</template>

<script setup>
// Dati per i blocchi principali
const mainFeatures = [
  { title: 'Multi-platform Build', details: 'Build and package your web app for Windows, macOS and Linux from one repository, on your machine or with GitHub Actions.' },
  { title: 'Web/Desktop Bridge', details: 'Use native OS features directly from your web app through a typed JavaScript SDK.' },
  { title: 'Your Own Signing', details: 'Sign and notarize with your own certificates, kept as your repository secrets. Unsigned builds work too.' },
  { title: 'No Lock-in', details: 'Apache 2.0, no account and no backend. The apps you build are yours, and so is the toolchain.' },
]

// Dati per la Bento Grid
const bridgeFeatures = [
  { icon: '🔔', name: 'Notifications', desc: 'Manage system notifications seamlessly.' },
  { icon: '🔴', name: 'Badges', desc: 'Control app badges for user alerts.' },
  { icon: 'tray', name: 'Tray Integration', desc: 'Add your app to the system tray for quick access.' },
  { icon: '🍔', name: 'Menu Integration', desc: 'Customize system menus with app-specific options.' },
  { icon: '📁', name: 'File System', desc: 'Access and manage the isolated file system securely.' },
  { icon: '⌨️', name: 'Shortcuts', desc: 'Register global shortcuts for enhanced user control.' },
  { icon: '🔗', name: 'Deep Links', desc: 'Handle custom protocol links efficiently.' },
  { icon: '⚡', name: 'Events', desc: 'Manage window and system events effectively.' },
  { icon: '📄', name: 'Files', desc: 'Handle file operations and dialogs natively.' },
  { icon: '🖱️', name: 'Drag & Drop', desc: 'Enable drag and drop interactions within your app.' },
  { icon: '🧩', name: 'WASM Plugins', desc: 'Extend functionality with sandboxed WebAssembly plugins.' },
  { icon: '🌐', name: 'Network', desc: 'Monitor and control network activities.' },
  { icon: '🔲', name: 'Window', desc: 'Manage window states and behaviors programmatically.' },
  { icon: '🚀', name: 'Autostart', desc: 'Configure your app to start automatically on system boot.' },
]
</script>

<style scoped>
.desktopr-home {
  max-width: 1152px;
  margin: 0 auto;
  padding: 0 clamp(16px, 4vw, 24px);
}

/* HERO SECTION */
.hero-section {
  position: relative;
  text-align: center;
  padding: clamp(48px, 10vw, 96px) 0 clamp(40px, 7vw, 64px);
  display: flex;
  flex-direction: column;
  align-items: center;
}

/**
 * The glow is decorative and wider than the text it sits behind, so it is sized relative to the
 * hero rather than in fixed pixels — a fixed 600px block would reach past a phone viewport and
 * give the whole page a horizontal scrollbar.
 */
.hero-glow {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: min(600px, 100%);
  height: clamp(240px, 50vw, 400px);
  background: radial-gradient(circle, var(--vp-c-brand-1) 0%, transparent 60%);
  filter: blur(100px);
  opacity: 0.15;
  z-index: -1;
  pointer-events: none;
}

.hero-brand-name-svg {
    width: 100% !important;
    opacity: 0.03;
    z-index: -10;
  stroke-color: white !important;
}

.hero-title {
  font-size: clamp(2.25rem, 8vw, 5rem);
  font-weight: 800;
  line-height: 1.1;
  letter-spacing: -0.02em;
  color: var(--vp-c-text-1);
  margin-bottom: 24px;
  text-wrap: balance;
}

.text-gradient {
  background: linear-gradient(135deg, var(--vp-c-brand-1), var(--vp-c-brand-3));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.hero-tagline {
  font-size: 1.25rem;
  line-height: 1.6;
  color: var(--vp-c-text-2);
  margin-bottom: 40px;
  max-width: 700px;
}

.hidden-mobile {
  display: none;
}
@media (min-width: 768px) {
  .hidden-mobile {
    display: inline;
  }
}

/* BUTTONS */
.hero-actions {
  display: flex;
  gap: 16px;
  flex-wrap: wrap;
  justify-content: center;
}

.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 12px 24px;
  border-radius: 8px;
  font-weight: 600;
  font-size: 16px;
  transition: all 0.2s ease;
  text-decoration: none !important;
}

.btn-primary {
  background-color: var(--vp-c-brand-1);
  color: var(--vp-button-brand-text);
}
.btn-primary:hover {
  background-color: var(--vp-c-brand-2);
  color: var(--vp-button-brand-text);
}

.btn-secondary {
  background-color: var(--vp-c-bg-soft);
  color: var(--vp-c-text-1);
  border: 1px solid var(--vp-c-border);
}
.btn-secondary:hover {
  border-color: var(--vp-c-brand-1);
  color: var(--vp-c-brand-1);
}

/* MAIN FEATURES */
.main-features {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 24px;
  padding: 64px 0;
}

.feature-card {
  padding: 24px;
  border-radius: 12px;
  background: var(--vp-c-bg-soft);
  border: 1px solid var(--vp-c-bg-soft);
  transition: border-color 0.2s;
}

.feature-card:hover {
  border-color: var(--vp-c-brand-soft);
}

.feature-card h3 {
  font-size: 1.125rem;
  font-weight: 600;
  margin-bottom: 12px;
  color: var(--vp-c-text-1);
}

.feature-card p {
  font-size: 0.95rem;
  color: var(--vp-c-text-2);
  line-height: 1.5;
}

/* DIVIDER */
.divider {
  height: 1px;
  background: var(--vp-c-border);
  margin: 32px 0;
  width: 100%;
}

/* NATIVE BRIDGE (BENTO GRID) */
.native-bridge {
  padding: 64px 0;
}

.bridge-header {
  text-align: center;
  max-width: 600px;
  margin: 0 auto 48px;
}

.bridge-header h2 {
  font-size: 2.5rem;
  font-weight: 700;
  margin-bottom: 16px;
}

.bridge-header p {
  color: var(--vp-c-text-2);
  font-size: 1.1rem;
}

.bento-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 16px;
}

.bento-card {
  background: var(--vp-c-bg-soft);
  border: 1px solid var(--vp-c-border);
  padding: 24px;
  border-radius: 16px;
  transition: transform 0.2s ease, border-color 0.2s ease;
}

.bento-card:hover {
  transform: translateY(-2px);
  border-color: var(--vp-c-brand-1);
}

.bento-icon {
  font-size: 1.5rem;
  margin-bottom: 16px;
  display: inline-block;
}

.bento-card h4 {
  font-size: 1rem;
  font-weight: 600;
  margin-bottom: 8px;
}

.bento-card p {
  font-size: 0.9rem;
  color: var(--vp-c-text-2);
  line-height: 1.4;
}

/* CTA SECTION */
.cta-section {
  padding: 96px 0;
  text-align: center;
}

.cta-content {
  background: var(--vp-c-bg-soft);
  border: 1px solid var(--vp-c-border);
  padding: 48px 24px;
  border-radius: 24px;
  max-width: 800px;
  margin: 0 auto;
}

.cta-content h2 {
  font-size: 2rem;
  font-weight: 700;
  margin-bottom: 16px;
}

.cta-content p {
  color: var(--vp-c-text-2);
  font-size: 1.1rem;
  margin-bottom: 32px;
  max-width: 600px;
  margin-inline: auto;
}
</style>