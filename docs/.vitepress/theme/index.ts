import DefaultTheme from 'vitepress/theme'
import type { Theme } from 'vitepress'
import { h } from 'vue'
import ComparisonTable from './components/ComparisonTable.vue'
import PageActions from './components/PageActions.vue'
import VPButton from 'vitepress/dist/client/theme-default/components/VPButton.vue'
import "./custom.css"

export default {
  ...DefaultTheme,

  /**
   * The page actions are registered once, in the layout's `doc-before` slot, rather than written
   * into every page — which is what makes it impossible to forget on one.
   */
  Layout() {
    return h(DefaultTheme.Layout, null, {
      "doc-before": () => h(PageActions),
    })
  },

  enhanceApp({ app }) {
    app.component('ComparisonTable', ComparisonTable),
    app.component('VPButton', VPButton)
  }
} satisfies Theme
