<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import AppNavigation from './components/AppNavigation.vue'

const route = useRoute()

// Define page header information for each route
const pageHeaders: Record<string, { title: string; subtitle: string; icon: string }> = {
  'Today': {
    title: 'Daily Liturgy',
    subtitle: 'Today\'s liturgical celebrations and readings',
    icon: '📖'
  },
  'Week': {
    title: 'Weekly Calendar',
    subtitle: 'Week-by-week liturgical calendar overview',
    icon: '📅'
  },
  'Month': {
    title: 'Monthly Calendar',
    subtitle: 'Month-by-month liturgical calendar with feast details',
    icon: '🗓️'
  },
  'Search': {
    title: 'Search Liturgical Data',
    subtitle: 'Find specific feasts, commemorations, and liturgical information',
    icon: '🔍'
  },
  'Nerd': {
    title: 'Advanced Comparison',
    subtitle: 'Detailed comparison and analysis of liturgical calendars',
    icon: '🤓'
  },
  'Novena': {
    title: 'Upcoming Novenas',
    subtitle: 'Track nine-day prayer devotions and feast preparations',
    icon: '🙏'
  },
  'About': {
    title: 'About Liturgical Calendar',
    subtitle: 'Information about this application and liturgical traditions',
    icon: 'ℹ️'
  }
}

const currentHeader = computed(() => {
  const routeName = route.name as string
  return pageHeaders[routeName] || {
    title: 'Liturgical Calendar',
    subtitle: 'Comprehensive liturgical calendar interface',
    icon: '📅'
  }
})
</script>

<template>
  <div id="app">
    <!-- Global App Header -->
    <header class="app-header">
      <div class="app-header-content">
        <h1 class="app-title">
          <span class="app-icon">⛪</span>
          Liturgical Calendar
        </h1>
        <p class="app-tagline">Comprehensive liturgical calendar and devotional tracker</p>
      </div>
    </header>

    <!-- Page-specific Header -->
    <header class="page-header">
      <div class="header-content">
        <h2 class="header-title">
          <span class="header-icon">{{ currentHeader.icon }}</span>
          {{ currentHeader.title }}
        </h2>
        <p class="header-subtitle">{{ currentHeader.subtitle }}</p>
      </div>
    </header>

    <AppNavigation />

    <main>
      <router-view />
    </main>

    <footer>
      <p>Backend API: <code>/api/*</code> | Built with Vue 3 + Rust/Axum</p>
    </footer>
  </div>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

:root {
  --accent-color: rgb(141, 83, 255);
  --secondary-color: rgb(41, 214, 185);
  
  /* Base colors */
  --background-color: #000000;
  --text-primary: #ffffff;
  --text-secondary: #cccccc;
  --text-muted: #aaa;
  --text-subtle: #999;
  --text-disabled: #777;
  
  /* Surface colors */
  --surface-primary: #111;
  --surface-secondary: #222;
  --surface-elevated: #333;
  --surface-interactive: #444;
  --surface-disabled: #1a1a1a;
  
  /* Border colors */
  --border-color: #333;
  --border-primary: #333;
  --border-secondary: #444;
  --border-subtle: #555;
  
  /* Status colors */
  --error-color: #ff6b6b;
  --error-bg: #3a1a1a;
  --error-text: #ff6b6b;
  --error-border: #4a2828;
  --warning-color: #ffaa66;
  --success-color: #4CAF50;
  --info-color: #4a9eff;
  
  /* Special colors that should use accent */
  --highlight-color: var(--accent-color);
  --interactive-color: var(--accent-color);
  
  /* 
    FIXED LAYOUT SYSTEM
    
    This system prevents all pages from being reactive to their content.
    Instead of using max-width (which allows content to dictate size),
    we use fixed width values that only change based on screen size,
    never based on the data being displayed.
    
    Key principles:
    - width: var(--layout-fixed-width) instead of max-width
    - box-sizing: border-box for consistent sizing
    - max-width: 100vw as fallback for very small screens
    - Fixed breakpoints that only respond to viewport, not content
  */
  
  /* Layout dimensions - consistent across all views */
  --layout-max-width: 1200px;
  --layout-max-width-wide: 1400px; /* For views that need more space like MonthView */
  --layout-padding: 1rem;
  --layout-gap: 1rem;
  
  /* Fixed layout system - prevents reactivity to content */
  --layout-fixed-width: 1200px;
  --layout-fixed-width-wide: 1400px;
  --layout-min-height: 100vh;
}

/* Responsive fixed widths - still fixed, just smaller on mobile */
@media (max-width: 1440px) {
  :root {
    --layout-fixed-width: 1200px;
    --layout-fixed-width-wide: 1200px; /* Use standard width on smaller screens */
  }
}

@media (max-width: 1240px) {
  :root {
    --layout-fixed-width: 1000px;
    --layout-fixed-width-wide: 1000px;
  }
}

@media (max-width: 1040px) {
  :root {
    --layout-fixed-width: 900px;
    --layout-fixed-width-wide: 900px;
  }
}

@media (max-width: 940px) {
  :root {
    --layout-fixed-width: 100vw;
    --layout-fixed-width-wide: 100vw;
    --layout-padding: 0.5rem;
  }
}

body {
  font-family: Inter, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  margin: 0;
  padding: 0;
  background-color: var(--background-color);
  color: var(--text-primary);
}

/* Global text color defaults */
h1, h2, h3, h4, h5, h6 {
  color: var(--text-primary);
}

p, div, span, td, th, label {
  color: var(--text-primary);
}

.text-muted {
  color: var(--text-secondary);
}

/* Button text visibility */
button {
  color: var(--text-primary);
}

/* Form controls */
input, select, textarea {
  background-color: var(--surface-elevated);
  color: var(--text-primary);
  border: 1px solid var(--border-subtle);
}

input:focus, select:focus, textarea:focus {
  border-color: var(--accent-color);
}

#app {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}

/* Global App Header */
.app-header {
  background: var(--surface-elevated);
  color: var(--text-primary);
  text-align: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-primary);
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.1);
}

.app-header-content {
  max-width: var(--layout-fixed-width);
  margin: 0 auto;
  padding: 0 var(--layout-padding);
}

.app-title {
  margin-bottom: 4px;
  font-size: 20px;
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  color: var(--text-primary);
}

.app-icon {
  font-size: 1.1em;
  color: var(--accent-color);
}

.app-tagline {
  margin: 0;
  color: var(--text-muted);
  font-size: 12px;
  font-weight: 400;
}

/* Page Header - Made less obnoxious */
.page-header {
  background: var(--surface-secondary);
  color: var(--text-primary);
  text-align: center;
  padding: 12px 20px;
  border-bottom: 1px solid var(--border-subtle);
}

.header-content {
  max-width: var(--layout-fixed-width);
  margin: 0 auto;
  padding: 0 var(--layout-padding);
}

.header-title {
  margin-bottom: 4px;
  font-size: 18px;
  font-weight: 500;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.4rem;
  color: var(--text-primary);
}

.header-icon {
  font-size: 1em;
  color: var(--accent-color);
}

.header-subtitle {
  margin: 0;
  color: var(--text-secondary);
  font-size: 12px;
  font-weight: 400;
}

main {
  flex: 1;
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
  width: 100%;
}

footer {
  background: var(--surface-primary);
  color: var(--text-secondary);
  text-align: center;
  padding: 16px 20px;
  margin-top: auto;
  border-top: 1px solid var(--border-primary);
}

footer p {
  margin: 0;
  font-size: 13px;
}

footer code {
  background: var(--surface-secondary);
  padding: 2px 4px;
  border-radius: 2px;
  font-family: 'SF Mono', Monaco, 'Cascadia Code', monospace;
  font-size: 12px;
}

@media (max-width: 768px) {
  .app-title {
    font-size: 18px;
    flex-direction: column;
    gap: 0.25rem;
  }
  
  .app-tagline {
    font-size: 11px;
  }

  .header-title {
    font-size: 16px;
    flex-direction: column;
    gap: 0.2rem;
  }
  
  .header-subtitle {
    font-size: 11px;
  }
  
  main {
    padding: 16px;
  }
}
</style>
