<template>
  <div class="ordo-view">
    <h1>Ordo</h1>
    <div class="controls">
      <label>Calendar
        <select v-model="selectedCalendar">
          <option v-for="cal in calendars" :key="cal.name" :value="cal.name">{{ cal.display_name }}</option>
        </select>
      </label>
      <button :disabled="loading || !selectedCalendar" @click="fetchVespers">{{ loading ? 'Loading…' : 'Refresh' }}</button>
    </div>

    <p v-if="!calendars.length && !loading" class="hint">No calendars loaded yet—check backend availability.</p>

    <div v-if="loading">Loading…</div>

    <section v-if="vespers">
      <h2>Vespers</h2>
      <div class="vespers-text">{{ vespers.name }}</div>
      <div class="ordo-grid">
        <div class="ordo-card" v-for="[key, val] in ordoEntries" :key="key">
          <div class="ordo-key">{{ labelForKey(key) }}</div>
          <div class="ordo-value">{{ formatLocation(val) }}</div>
        </div>
      </div>
    </section>

    <div v-if="error" class="error">{{ error }}</div>
  </div>
 </template>

<script lang="ts">
import { ref, watch, onMounted, computed } from 'vue'
import type { Ref } from 'vue'
import { useDateNavigation } from '../composables/useDateNavigation'
import { api } from '../services/api'
import type { CalendarInfo, Vespers, OfficeComponentLocation } from '../services/api'

export default {
  name: 'OrdoView',
  setup() {
    const calendars: Ref<CalendarInfo[]> = ref([])
    const selectedCalendar = ref('ef')
    // Use shared navigation date (now visible on Ordo via AppNavigation)
    const { selectedDate } = useDateNavigation('Ordo')
    const vespers = ref<Vespers | null>(null)
    const sources = ref<string[]>([])
    const ordoKeys: Array<keyof Vespers['ordo']> = [
      'antiphons',
      'psalms',
      'chapter',
      'hymn',
      'verse',
      'magnificat_antiphon',
      'collect',
    ]

    const ordoEntries = computed<[keyof Vespers['ordo'], OfficeComponentLocation][]>(() => {
      if (!vespers.value) return []
      return ordoKeys.map((key) => [key, vespers.value!.ordo[key]])
    })

    function labelForKey(key: keyof Vespers['ordo']): string {
      switch (key) {
        case 'antiphons':
          return 'Antiphons'
        case 'psalms':
          return 'Psalms'
        case 'chapter':
          return 'Chapter'
        case 'hymn':
          return 'Hymn'
        case 'verse':
          return 'Verse'
        case 'magnificat_antiphon':
          return 'Magnificat Antiphon'
        case 'collect':
          return 'Collect'
        default:
          return String(key)
      }
    }

    function formatLocation(loc: any): string {
      // Tagged-object shape
      if (loc && typeof loc === 'object' && 'type' in loc) {
        switch (loc.type) {
          case 'Common':
            return loc.name ? `Common: ${loc.name}` : 'Common'
          case 'Proper':
            return 'Proper'
          case 'Ordinary':
            return loc.source ? `Ordinary: ${loc.source}` : 'Ordinary'
          case 'Octave':
            return loc.source ? `Octave: ${loc.source}` : 'Octave'
          case 'Psalter':
            return 'Psalter'
          case 'Sunday':
            return loc.source ? `Sunday (${loc.source})` : 'Sunday'
          default:
            return JSON.stringify(loc)
        }
      }

      // Legacy map shape from the backend enum, e.g. { "Ordinary": "Advent" }
      if (loc && typeof loc === 'object') {
        const entries = Object.entries(loc)
        if (entries.length === 1) {
          const [k, v] = entries[0]
          const val = typeof v === 'string' && v.length ? v : null
          switch (k) {
            case 'Common':
              return val ? `Common of ${val}` : 'Common'
            case 'Proper':
              return 'Proper'
            case 'Ordinary':
              return val ? `Ordinary of ${val}` : 'Ordinary'
            case 'Octave':
              return val ? `Octave of ${val}` : 'Octave'
            case 'Psalter':
              return 'Psalter'
            case 'Sunday':
              return val ? `Sunday (${val})` : 'Sunday'
            default:
              return `${k}${val ? `: ${val}` : ''}`
          }
        }
      }

      if (typeof loc === 'string') return loc
      return JSON.stringify(loc)
    }
    const loading = ref(false)
    const error = ref('')

    async function loadCalendars() {
      try {
        calendars.value = await api.getCalendars()
        // Keep ef as default, only change if ef doesn't exist
        if (!calendars.value.some((cal) => cal.name === 'ef') && calendars.value.length > 0) {
          selectedCalendar.value = calendars.value[0].name
        }
      } catch (e: any) {
        error.value = e?.message || 'Failed to load calendars'
      }
    }

    async function fetchVespers() {
      if (!selectedCalendar.value) return
      error.value = ''
      vespers.value = null
      sources.value = []
      loading.value = true

      try {
        const [year, month, day] = selectedDate.value.split('-').map(Number)
        const cal = selectedCalendar.value

        vespers.value = await api.getOrdoVespers(cal, year, month, day)
        sources.value = await api.getOrdoVespersSources(cal, year, month, day)
      } catch (e: any) {
        error.value = e?.message || 'Fetch error'
      } finally {
        loading.value = false
      }
    }

    onMounted(async () => {
      await loadCalendars()
      fetchVespers()
    })

    // Watch for calendar changes and refresh
    watch(selectedCalendar, () => {
      fetchVespers()
    })

    // Refresh when date changes via the top nav
    watch(
      () => selectedDate.value,
      () => {
        fetchVespers()
      },
    )

    return {
      calendars,
      selectedCalendar,
      selectedDate,
      vespers,
      ordoEntries,
      labelForKey,
      formatLocation,
      sources,
      loading,
      error,
      fetchVespers,
    }
  },
}
</script>

<style scoped>
.ordo-view {
  max-width: 900px;
  margin: 0 auto;
  padding: 20px;
}

.ordo-view h1 {
  font-size: 2rem;
  margin-bottom: 1.5rem;
  color: var(--text-primary);
}

.controls {
  display: flex;
  gap: 1rem;
  align-items: center;
  margin-bottom: 1.5rem;
  flex-wrap: wrap;
}

.controls label {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.controls select {
  padding: 0.5rem;
  border-radius: 6px;
  border: 1px solid var(--border-primary);
  background: var(--surface-secondary);
  color: var(--text-primary);
  font-size: 0.95rem;
}

.controls button {
  padding: 0.5rem 1rem;
  background: var(--accent-color);
  color: white;
  border: none;
  border-radius: 6px;
  font-weight: 600;
  cursor: pointer;
  transition: filter 0.2s ease;
}

.controls button:hover:not(:disabled) {
  filter: brightness(0.9);
}

.controls button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.hint {
  color: var(--text-secondary);
  font-style: italic;
  margin: 1rem 0;
}

.error {
  color: #c33;
  background: #fee;
  padding: 1rem;
  border-radius: 6px;
  margin: 1rem 0;
}

section {
  margin: 2rem 0;
  padding: 1rem;
  background: var(--surface-secondary);
  border-radius: 8px;
}

section h2 {
  font-size: 1.2rem;
  margin-top: 0;
  margin-bottom: 1rem;
  color: var(--text-primary);
}

.ordo-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 0.9rem 1.2rem;
}

.ordo-card {
  background: var(--surface-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 8px;
  padding: 0.9rem 1rem;
  display: grid;
  gap: 0.35rem;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.06);
}

.ordo-key {
  font-weight: 700;
  color: var(--text-secondary);
  letter-spacing: 0.01em;
  font-size: 0.95rem;
}

.ordo-value {
  color: var(--text-primary);
  font-size: 0.98rem;
}

.vespers-text {
  font-family: 'Georgia', serif;
  line-height: 1.8;
  white-space: pre-wrap;
  word-wrap: break-word;
  color: var(--text-primary);
  font-size: 0.95rem;
}

.sources {
  padding-left: 1.5rem;
}

.sources li {
  margin-bottom: 0.5rem;
  color: var(--text-primary);
}
</style>
