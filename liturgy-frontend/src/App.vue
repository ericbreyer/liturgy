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
    subtitle: 'Overview of the coming week',
    icon: '📅',
  },
  Month: {
    title: 'Monthly View',
    subtitle: 'Monthly liturgical overview',
    icon: '🗓️',
  },
  Search: {
    title: 'Search',
    subtitle: 'Find liturgies, saints, and celebrations',
    icon: '🔍',
  },
  About: {
    title: 'About',
    subtitle: 'About this project and resources',
    icon: 'ℹ️',
  },
}

const currentHeader = computed(() => {
  const name = (route.name || 'Today') as string
  return pageHeaders[name] ?? { title: 'Liturgy', subtitle: '', icon: '📜' }
})
</script>

<template>
  <div id="app">
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

    <footer class="site-footer">
      <div class="footer-inner">
        <div class="footer-left">
          <small>&copy; 2024 — Liturgy</small>
          <nav class="footer-links">
            <a href="/about">About</a>
            <a href="/privacy">Privacy</a>
            <a href="/contact">Contact</a>
          </nav>
        </div>

        <div class="footer-right" aria-label="Social links">
          <ul class="social-list">
            <li>
              <a href="https://github.com/ericbreyer" target="_blank" rel="noopener noreferrer" aria-label="GitHub">
                <svg class="icon" aria-hidden="true" focusable="false"><use href="/icons.svg#github"/></svg>
                <span class="visually-hidden">GitHub</span>
              </a>
            </li>
            <li>
              <a href="https://www.linkedin.com/in/eric-breyer" target="_blank" rel="noopener noreferrer" aria-label="LinkedIn">
                <svg class="icon" aria-hidden="true" focusable="false"><use href="/icons.svg#linkedin"/></svg>
                <span class="visually-hidden">LinkedIn</span>
              </a>
            </li>
            <li>
              <a href="https://eric-breyer.medium.com/" target="_blank" rel="noopener noreferrer" aria-label="Medium">
                <svg class="icon" aria-hidden="true" focusable="false"><use href="/icons.svg#medium"/></svg>
                <span class="visually-hidden">Medium</span>
              </a>
            </li>
            <li>
              <a href="https://wild-mortimer.itch.io/" target="_blank" rel="noopener noreferrer" aria-label="Itch.io">
                <svg class="icon" aria-hidden="true" focusable="false"><use href="/icons.svg#itch"/></svg>
                <span class="visually-hidden">Itch.io</span>
              </a>
            </li>
            <li>
              <a href="https://www.youtube.com/channel/UCJUtMYyEcdAQNTKuja74Alg" target="_blank" rel="noopener noreferrer" aria-label="YouTube">
                <svg class="icon" aria-hidden="true" focusable="false"><use href="/icons.svg#youtube"/></svg>
                <span class="visually-hidden">YouTube</span>
              </a>
            </li>
            <li>
              <a href="https://www.instagram.com/eric_breyer/" target="_blank" rel="noopener noreferrer" aria-label="Instagram">
                <svg class="icon" aria-hidden="true" focusable="false"><use href="/icons.svg#instagram"/></svg>
                <span class="visually-hidden">Instagram</span>
              </a>
            </li>
          </ul>
        </div>
      </div>

      <div class="footer-meta">
        <p>Built by Eric Breyer 2025 | Send feedback to <code>eric.breyer@gmail.com</code></p>
      </div>
    </footer>
  </div>
</template>

<style>
@import './styles/liturgical.css';

.site-footer {
  margin-top: 32px;
  padding: 18px 20px;
  border-top: 1px solid var(--border-primary);
  background: var(--surface-primary);
}
.footer-inner {
  max-width: 1100px;
  margin: 0 auto;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  flex-wrap: wrap;
}
.footer-left { display:flex; gap:12px; align-items:center }
.footer-links a { margin-left: 12px; color: var(--text-secondary); text-decoration: none }
.social-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  gap: 12px;
  align-items: center;
}
.social-list a { color: var(--text-secondary); text-decoration: none; display:inline-flex; gap:8px; align-items:center }
.icon { width: 18px; height: 18px }
.footer-meta { color: var(--text-secondary); font-size: 13px; text-align: center; margin-top: 12px }
.visually-hidden { position: absolute; left: -10000px; top: auto; width: 1px; height: 1px; overflow: hidden }
</style>
