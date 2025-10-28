<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import AppNavigation from './components/AppNavigation.vue'

const route = useRoute()

// Define page header information for each route
const pageHeaders: Record<string, { title: string; subtitle: string; icon: string }> = {
  Today: {
    title: 'Daily Liturgy',
    subtitle: "Today's liturgical celebrations and readings",
    icon: '📖',
  },
  Week: {
    title: 'Weekly Calendar',
    subtitle: 'Week-by-week liturgical calendar overview',
    icon: '📅',
  },
  Month: {
    title: 'Monthly Calendar',
    subtitle: 'Month-by-month liturgical calendar with feast details',
    icon: '🗓️',
  },
  Search: {
    title: 'Search Liturgical Data',
    subtitle: 'Find specific feasts, commemorations, and liturgical information',
    icon: '🔍',
  },
  Nerd: {
    title: 'Advanced Comparison',
    subtitle: 'Detailed comparison and analysis of liturgical calendars',
    icon: '🤓',
  },
  Novena: {
    title: 'Upcoming Novenas',
    subtitle: 'Track nine-day prayer devotions and feast preparations',
    icon: '🙏',
  },
  About: {
    title: 'About Liturgical Calendar',
    subtitle: 'Information about this application and liturgical traditions',
    icon: 'ℹ️',
  },
}

const currentHeader = computed(() => {
  const routeName = route.name as string
  return (
    pageHeaders[routeName] || {
      title: 'Liturgical Calendar',
      subtitle: 'Comprehensive liturgical calendar interface',
      icon: '📅',
    }
  )
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
      <p>Built by Eric Breyer 2025 | Send feedback to <code>eric.breyer@gmail.com</code></p>
    </footer>
  </div>
</template>

<style>
@import './styles/liturgical.css';
/* App-specific overrides kept minimal; global tokens live in src/assets/global.css */
</style>
