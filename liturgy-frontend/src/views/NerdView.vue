<script setup lang="ts">
import { ref, computed, watch, onBeforeUnmount } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useCalendarSelection } from '../composables/useCalendarSelection'
import { useDateNavigation } from '../composables/useDateNavigation'
import { api, type CycleFeast } from '../services/api'
import {
  compareCycles,
  cycleStatus,
  normalizeFeastName,
  type CycleScope,
  type CycleSort,
} from '../utils/cycleComparison'
import { getColorValue } from '../utils/liturgical'

const route = useRoute()
const router = useRouter()
const { selectedCalendars } = useCalendarSelection()
const { selectedDate, formattedDate } = useDateNavigation('Nerd')
const scopeOptions: CycleScope[] = ['day', 'year']
const scope = computed<CycleScope>(() => (route.query.scope === 'year' ? 'year' : 'day'))
const year = computed(() => Number(selectedDate.value.slice(0, 4)))
const sort = ref<CycleSort>('date')
const search = ref('')
const cycles = ref<Record<string, CycleFeast[]>>({})
const errors = ref<Record<string, string>>({})
const isLoading = ref(false)
const cache = new Map<string, CycleFeast[]>()
let controller: AbortController | undefined

const feastComparisons = computed(() => {
  const orderedCycles: Record<string, CycleFeast[]> = {}
  for (const calendar of selectedCalendars.value) {
    const feasts = cycles.value[calendar]
    if (feasts) orderedCycles[calendar] = feasts
  }
  const comparisons = compareCycles(orderedCycles, selectedDate.value, scope.value, sort.value)
  const terms = search.value.trim().split(/\s+/).map(normalizeFeastName).filter(Boolean)
  if (scope.value !== 'year' || !terms.length) return comparisons
  return comparisons.filter((comparison) => {
    const text = normalizeFeastName([
      comparison.name,
      ...Object.values(comparison.calendars).map((feasts) => feasts.map((feast) =>
        [feast.name, feast.description, ...feast.titles, feast.date ?? ''].join(' ')).join(' ')),
    ].join(' '))
    return terms.every((term) => text.includes(term))
  })
})
const gridTemplateColumns = computed(
  () => `minmax(180px, 2fr) repeat(${selectedCalendars.value.length}, minmax(180px, 1fr))`,
)
const minTableWidth = computed(() => 180 * (selectedCalendars.value.length + 1))

function setScope(value: CycleScope) {
  router.push({ query: { ...route.query, cycle: undefined, scope: value } })
}

async function loadCycles() {
  controller?.abort()
  const request = new AbortController()
  controller = request
  const calendars = [...selectedCalendars.value]
  const requestYear = year.value
  cycles.value = {}
  errors.value = {}
  isLoading.value = calendars.length > 0
  await Promise.all(
    calendars.map(async (calendar) => {
      const key = `${calendar}/${requestYear}`
      try {
        const feasts =
          cache.get(key) ??
          (await api.getCalendarCycle(calendar, 'sanctoral', requestYear, request.signal))
        if (controller !== request || request.signal.aborted) return
        cache.set(key, feasts)
        cycles.value = { ...cycles.value, [calendar]: feasts }
      } catch (error) {
        if (controller !== request || request.signal.aborted) return
        errors.value = {
          ...errors.value,
          [calendar]: error instanceof Error ? error.message : 'Failed to load cycle',
        }
      }
    }),
  )
  if (controller === request && !request.signal.aborted) isLoading.value = false
}

watch([year, selectedCalendars], loadCycles, { immediate: true, deep: true })
onBeforeUnmount(() => controller?.abort())
</script>

<template>
  <div class="nerd-view">
    <div class="content-section">
      <p class="date-display">
        {{ scope === 'year' ? year : formattedDate }} ·
        Sanctoral
      </p>
      <div class="comparison-controls">
        <fieldset class="segmented">
          <legend class="sr-only">Comparison scope</legend>
          <label v-for="mode in scopeOptions" :key="mode" :class="{ active: scope === mode }">
            <input
              type="radio"
              name="cycle-scope"
              :value="mode"
              :checked="scope === mode"
              @change="setScope(mode)"
            />
            {{ mode === 'day' ? 'This date' : 'Full cycle' }}
          </label>
        </fieldset>
        <label v-if="scope === 'year'" class="search-control">
          <span class="sr-only">Search full cycle</span>
          <input v-model="search" type="search" placeholder="Search feasts" />
        </label>
        <label v-if="scope === 'year'" class="sort-control"
          >Sort
          <select v-model="sort" aria-label="Sort cycle">
            <option value="date">First date</option>
            <option value="name">Name</option>
          </select>
        </label>
      </div>
    </div>

    <div v-if="isLoading" class="loading" role="status">Loading sanctoral cycle...</div>

    <div v-for="(message, calendar) in errors" :key="calendar" class="error" role="alert">
      {{ calendar.toUpperCase() }}: {{ message }}
    </div>

    <div v-if="selectedCalendars.length === 0" class="no-calendars">
      Select at least one calendar to compare.
    </div>

    <div v-if="feastComparisons.length > 0" class="comparison-results">
      <div
        class="feast-comparison-table"
        role="table"
        aria-label="Source cycle comparison"
        :aria-busy="isLoading"
        :style="{ '--min-table-width': minTableWidth + 'px' }"
      >
        <div class="table-header" role="row" :style="{ gridTemplateColumns }">
          <div class="feast-name-col" role="columnheader">Feast</div>
          <div
            v-for="calendar in selectedCalendars"
            :key="calendar"
            class="calendar-col"
            role="columnheader"
          >
            {{ calendar.toUpperCase() }}
          </div>
        </div>

        <div
          v-for="comparison in feastComparisons"
          :key="comparison.identity"
          class="feast-row"
          role="row"
          :style="{ gridTemplateColumns }"
        >
          <div class="feast-name" role="rowheader">{{ comparison.name }}</div>

          <div v-for="calendar in selectedCalendars" :key="calendar" class="feast-cell" role="cell">
            <span v-if="errors[calendar]" class="status-label">Unavailable</span>
            <template v-else-if="comparison.calendars[calendar]">
              <div
                v-for="(feast, index) in comparison.calendars[calendar]"
                :key="index"
                class="feast-details"
              >
                <div class="status-label" :class="{ 'on-date': scope === 'day' && feast.date === selectedDate }">
                  {{ cycleStatus(feast, selectedDate, scope, comparison, calendar) }}
                </div>
                <div
                  class="color-bar"
                  :style="{ backgroundColor: getColorValue(feast.color) }"
                  :title="feast.color"
                ></div>
                <div class="feast-info">
                  <time v-if="feast.date" :datetime="feast.date">{{ feast.date }}</time>
                  <div class="rank">Rank: {{ feast.rank }}</div>
                  <div v-if="feast.titles.length" class="titles">{{ feast.titles.join(', ') }}</div>
                  <div v-if="feast.description !== comparison.name" class="description">
                    {{ feast.description }}
                  </div>
                  <div class="date-rule">Rule: {{ feast.date_rule }}</div>
                </div>
              </div>
            </template>
            <span v-else class="status-label">{{ isLoading ? 'Loading...' : 'Not in cycle' }}</span>
          </div>
        </div>
      </div>
    </div>

    <div
      v-else-if="!isLoading && selectedCalendars.length > 0 && !Object.keys(errors).length"
      class="no-feasts"
      role="status"
    >
      {{
        scope === 'year'
          ? search.trim() ? 'No matching feasts.' : `No entries in the ${year} sanctoral cycle.`
          : `No source feasts on ${selectedDate}.`
      }}
    </div>
  </div>
</template>

<style scoped>
.nerd-view {
  width: 100%;
  min-width: 0;
  max-width: 100%;
  margin: 0 auto;
  padding: 0;
  box-sizing: border-box;
}

.content-section {
  margin-bottom: 20px;
}

.date-display {
  color: var(--text-primary);
  font-weight: 600;
  margin-bottom: 8px;
  font-size: 16px;
}

.comparison-controls {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 12px;
}

.segmented {
  display: flex;
  flex-wrap: wrap;
  padding: 0;
  margin: 0;
  border: 1px solid var(--border-primary);
  border-radius: 4px;
  overflow: hidden;
}

.segmented label {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  cursor: pointer;
  color: var(--text-primary);
  background: var(--surface-primary);
}

.segmented label.active {
  background: var(--surface-elevated);
  font-weight: 600;
}

.segmented label:focus-within {
  outline: 2px solid var(--text-primary);
  outline-offset: -2px;
}

.sort-control {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--text-primary);
}

.search-control {
  flex: 1 1 220px;
  min-width: 0;
  max-width: 360px;
}

.search-control input {
  width: 100%;
  min-width: 0;
  box-sizing: border-box;
}

.sort-control select,
.search-control input {
  max-width: 100%;
  padding: 6px;
  color: var(--text-primary);
  background: var(--surface-primary);
  border: 1px solid var(--border-primary);
  border-radius: 4px;
}

.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
}

.loading,
.error,
.no-calendars,
.no-feasts {
  text-align: center;
  padding: 40px;
  color: var(--text-secondary);
  font-size: 16px;
}

.error {
  color: #ff6b6b;
  background: var(--error-bg);
  border: 1px solid var(--error-border);
  border-radius: 8px;
}

.feast-comparison-table {
  min-width: 0;
  max-width: 100%;
  background: var(--surface-primary);
  border-radius: 4px;
  overflow-x: auto;
  border: 1px solid var(--border-primary);
}

.table-header {
  display: grid;
  background: var(--surface-interactive);
  color: var(--text-primary);
  font-weight: 600;
  font-size: 14px;
}

.feast-name-col,
.calendar-col {
  min-width: 0;
  overflow-wrap: anywhere;
  padding: 16px;
  border-right: 1px solid var(--border-subtle);
}

.calendar-col:last-child {
  border-right: none;
}

.feast-row {
  display: grid;
  border-bottom: 1px solid var(--border-primary);
}

.feast-row:last-child {
  border-bottom: none;
}

.feast-name {
  min-width: 0;
  overflow-wrap: anywhere;
  padding: 16px;
  font-weight: 600;
  color: var(--text-primary);
  border-right: 1px solid var(--border-primary);
  background: var(--surface-secondary);
}

.feast-cell {
  min-width: 0;
  overflow-wrap: anywhere;
  padding: 12px;
  border-right: 1px solid var(--border-primary);
  min-height: 80px;
}

.feast-cell:last-child {
  border-right: none;
}

.feast-details + .feast-details {
  margin-top: 16px;
  padding-top: 12px;
  border-top: 1px solid var(--border-primary);
}

.status-label {
  font-size: 12px;
  color: var(--text-primary);
  font-weight: 500;
}

.on-date {
  color: var(--success-color);
}

.color-bar {
  height: 3px;
  width: 100%;
  border-radius: 2px;
  margin-bottom: 8px;
  border: 1px solid rgba(0, 0, 0, 0.1);
}

.feast-info {
  font-size: 12px;
  color: var(--text-secondary);
}

.rank {
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 4px;
}

.date-rule {
  margin-top: 4px;
}

.description {
  font-style: italic;
  color: var(--text-secondary);
  line-height: 1.3;
}

@media (max-width: 768px) {
  .feast-comparison-table {
    overflow-x: auto;
  }

  .table-header,
  .feast-row {
    min-width: var(--min-table-width, 600px);
  }
}

@media (max-width: 480px) {
  .feast-name-col,
  .calendar-col,
  .feast-name,
  .feast-cell {
    padding: 12px;
  }

  .segmented label {
    padding: 8px;
  }
}
</style>
