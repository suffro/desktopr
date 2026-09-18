<script setup lang="ts">
type CellValue = string | boolean

const products: { name: string; highlight: boolean }[] = [
  { name: 'Desktopr', highlight: true },
  { name: 'Electron', highlight: false },
  { name: 'Tauri', highlight: false },
  { name: 'ToDesktop', highlight: false },
  { name: 'Neutralino.js', highlight: false },
]

const rows: { label: string; values: CellValue[] }[] = [
  {
    label: 'Under the hood',
    values: [
      'Tauri + Rust (managed)',
      'Chromium + Node.js',
      'Rust + OS WebView',
      'Electron (managed)',
      'OS WebView + C++',
    ],
  },
  {
    label: 'Installer size',
    values: ['~3–15 MB', '~100–200 MB', '~3–10 MB', '~100–200 MB', '~2–5 MB'],
  },
  {
    label: 'Languages required',
    values: ['JS / TS only', 'JS / TS', 'JS / TS + Rust', 'JS / TS', 'JS / TS'],
  },
  {
    label: 'Setup time',
    values: ['~5 min (dashboard)', 'Hours – days', 'Hours – days', 'Hours – days', '30 min – hours'],
  },
  {
    label: 'Native API access',
    values: ['Rich (15+ modules)', 'Very rich', 'Very rich', 'Rich', 'Basic'],
  },
  {
    label: 'Platforms',
    values: [
      'Win, Mac, Linux',
      'Win, Mac, Linux',
      'Win, Mac, Linux¹',
      'Win, Mac, Linux',
      'Win, Mac, Linux',
    ],
  },
  {
    label: 'Cloud builds',
    values: [true, false, false, true, false],
  },
  {
    label: 'Code signing',
    values: [true, false, false, true, false],
  },
  {
    label: 'Managed distribution',
    values: [true, false, false, true, false],
  },
  {
    label: 'Learning curve',
    values: ['Low', 'Medium', 'High', 'Medium', 'Low - medium'],
  },
  {
    label: 'Vendor lock-in',
    values: ['None', 'None', 'None', 'Medium - High', 'None'],
  },
  {
    label: 'Free to try',
    values: [
      'Free (open source) + Companion app',
      'Free (open source)',
      'Free (open source)',
      '7-day trial',
      'Free (open source)',
    ],
  },
  {
    label: 'Plugin system',
    values: [
      'WASM Plugins (unlimited)',
      'npm / native addons',
      'Rust plugin ecosystem',
      'npm packages',
      'Extensions (limited)',
    ],
  },
  {
    label: 'No-code support',
    values: [true, false, false, false, false],
  },
  {
    label: 'Pricing',
    values: [
      'Free (open source)',
      'Free (open source)',
      'Free (open source)',
      'From $100 / month',
      'Free (open source)',
    ],
  },
]
</script>

<template>
  <div class="ct-wrapper">
    <div class="ct-scroll">
      <table class="ct-table" aria-label="Desktop app framework comparison">
        <thead>
          <tr>
            <th class="ct-feature-col" scope="col">Feature</th>
            <th
              v-for="p in products"
              :key="p.name"
              scope="col"
              :class="['ct-product-col', { 'ct-highlight': p.highlight }]"
            >
              <span :class="['ct-product-name', { 'ct-brand': p.highlight }]">{{ p.name }}</span>
              <span v-if="p.highlight" class="ct-badge">recommended</span>
            </th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="(row, i) in rows"
            :key="row.label"
            :class="{ 'ct-row-alt': i % 2 !== 0 }"
          >
            <td class="ct-label">{{ row.label }}</td>
            <td
              v-for="(val, j) in row.values"
              :key="j"
              :class="['ct-cell', { 'ct-highlight': products[j].highlight }]"
            >
              <template v-if="typeof val === 'boolean'">
                <span v-if="val" class="ct-check" aria-label="Yes">✓</span>
                <span v-else class="ct-dash" aria-label="No">—</span>
              </template>
              <template v-else>
                <span class="ct-text">{{ val }}</span>
              </template>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
    <p class="ct-note">¹ Tauri has experimental iOS and Android support. Prices subject to change.</p>
  </div>
</template>

<style scoped>
.ct-wrapper {
  margin: 2rem 0;
}

.ct-scroll {
  overflow-x: auto;
  border-radius: 8px;
  border: 1px solid var(--vp-c-divider);
}

.ct-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.875rem;
  min-width: 640px;
}

/* Header */
.ct-table thead tr {
  background-color: var(--vp-c-bg-soft);
}

.ct-feature-col,
.ct-product-col {
  padding: 12px 14px;
  text-align: center;
  font-weight: 600;
  border-bottom: 2px solid var(--vp-c-divider);
  white-space: nowrap;
}

.ct-feature-col {
  text-align: left;
  min-width: 160px;
  color: var(--vp-c-text-2);
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.ct-product-col {
  min-width: 130px;
}

/* Highlighted column header */
.ct-highlight {
  background-color: rgba(254, 154, 1, 0.06);
  border-left: 2px solid rgba(254, 154, 1, 0.35);
  border-right: 2px solid rgba(254, 154, 1, 0.35);
}

.ct-product-name {
  display: block;
  font-size: 0.9rem;
}

.ct-brand {
  color: #FE9A01;
}

.ct-badge {
  display: inline-block;
  margin-top: 4px;
  font-size: 0.6rem;
  font-weight: 600;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: #FE9A01;
  background-color: rgba(254, 154, 1, 0.12);
  border: 1px solid rgba(254, 154, 1, 0.3);
  border-radius: 100px;
  padding: 1px 7px;
}

/* Rows */
.ct-table tbody tr {
  transition: background-color 0.15s;
}

.ct-table tbody tr:hover {
  background-color: var(--vp-c-bg-soft);
}

.ct-row-alt {
  background-color: rgba(128, 128, 128, 0.04);
}

.ct-label {
  padding: 10px 14px;
  font-weight: 500;
  color: var(--vp-c-text-1);
  border-right: 1px solid var(--vp-c-divider);
  white-space: nowrap;
}

.ct-cell {
  padding: 10px 14px;
  text-align: center;
  color: var(--vp-c-text-2);
  border-left: 1px solid transparent;
  border-right: 1px solid transparent;
  vertical-align: middle;
}

.ct-cell.ct-highlight {
  background-color: rgba(254, 154, 1, 0.04);
  border-left: 2px solid rgba(254, 154, 1, 0.35);
  border-right: 2px solid rgba(254, 154, 1, 0.35);
  color: var(--vp-c-text-1);
}

.ct-text {
  font-size: 0.82rem;
  line-height: 1.4;
}

.ct-check {
  color: #4caf80;
  font-size: 1rem;
  font-weight: 700;
}

.ct-dash {
  color: var(--vp-c-text-3, #888);
  font-size: 1rem;
}

/* Last row bottom border */
.ct-table tbody tr:last-child .ct-label,
.ct-table tbody tr:last-child .ct-cell {
  border-bottom: none;
}

/* Bottom cap on highlighted column */
.ct-table tbody tr:last-child .ct-cell.ct-highlight {
  border-bottom: 2px solid rgba(254, 154, 1, 0.35);
  border-bottom-left-radius: 6px;
  border-bottom-right-radius: 6px;
}

.ct-note {
  margin-top: 0.75rem;
  font-size: 0.75rem;
  color: var(--vp-c-text-3, #888);
  padding-left: 2px;
}
</style>
