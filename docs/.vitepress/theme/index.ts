import DefaultTheme from 'vitepress/theme'
import type { Theme } from 'vitepress'
import ComparisonTable from './components/ComparisonTable.vue'
import VPButton from 'vitepress/dist/client/theme-default/components/VPButton.vue'
import "./custom.css"

export default {
  ...DefaultTheme,

  enhanceApp({ app }) {
    app.component('ComparisonTable', ComparisonTable),
    app.component('VPButton', VPButton)
  }
} satisfies Theme
