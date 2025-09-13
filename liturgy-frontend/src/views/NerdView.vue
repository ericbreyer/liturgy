<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useCalendarSelection } from '../composables/useCalendarSelection'
import { useDateNavigation } from '../composables/useDateNavigation'
import { api, type DayInfo, type SearchResult } from '../services/api'
import { getColorValue, isFeria, getStatusIcon, getStatusLabel } from '../utils/liturgical'
import DateSelector from '../components/DateSelector.vue'

const route = useRoute()
const router = useRouter()
const { selectedCalendars } = useCalendarSelection()
const {
  selectedDate,
  formattedDate,
  updateSelectedDate,
  goToToday,
  goToPrevious: goToPreviousDay,
  goToNext: goToNextDay,
} = useDateNavigation('Nerd')

interface FeastComparison {
  name: string
  calendars: Record<
    string,
    {
      present: boolean
      description?: string
      rank?: string
      color?: string
      date?: string
      transferred?: boolean
      rankChanged?: boolean
    }
  >
  baseCalendar?: string // The calendar where this feast was first found
}

interface CalendarDayData {
  calendar: string
  dayInfo?: DayInfo
  loading: boolean
  error?: string
}

const calendarData = ref<CalendarDayData[]>([])
const feastComparisons = ref<FeastComparison[]>([])
const isLoading = ref(false)
const error = ref<string>('')

// Computed property for CSS grid columns
const gridTemplateColumns = computed(() => {
  const calendarCount = selectedCalendars.value.length
  return `2fr repeat(${calendarCount}, 1fr)`
})

// Computed property for minimum table width on mobile
const minTableWidth = computed(() => {
  const baseWidth = 300 // Base width for feast name column
  const calendarColumnWidth = 150 // Width per calendar column
  return baseWidth + selectedCalendars.value.length * calendarColumnWidth
})

// Get day data for all selected calendars
async function loadAllCalendarData() {
  if (selectedCalendars.value.length === 0) return

  isLoading.value = true
  error.value = ''

  // Parse the date string manually to avoid timezone issues
  const [year, month, day] = selectedDate.value.split('-').map(Number)

  calendarData.value = selectedCalendars.value.map((calendar) => ({
    calendar,
    loading: true,
  }))

  try {
    const promises = selectedCalendars.value.map(async (calendar, index) => {
      try {
        const dayInfo = await api.getDayInfo(calendar, year, month, day)
        calendarData.value[index] = {
          calendar,
          dayInfo,
          loading: false,
        }
      } catch (err) {
        calendarData.value[index] = {
          calendar,
          loading: false,
          error: err instanceof Error ? err.message : 'Failed to load',
        }
      }
    })

    await Promise.all(promises)
    await generateFeastComparisons()
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to load calendar data'
  } finally {
    isLoading.value = false
  }
}

// Generate feast comparisons by finding feasts across calendars
async function generateFeastComparisons() {
  const allFeasts = new Map<string, FeastComparison>()

  // Collect all feasts from all calendars
  for (const calData of calendarData.value) {
    if (!calData.dayInfo) continue

    const calendar = calData.calendar
    const dayData = calData.dayInfo

    // Add main feast (skip ferias - ordinary weekdays)
    if (dayData.desc?.day?.desc && !isFeria(dayData.desc.day.desc)) {
      const feastName = dayData.desc.day.desc
      addFeastToComparison(allFeasts, feastName, calendar, {
        present: true,
        description: dayData.desc.day.desc || '',
        rank: dayData.desc.day.rank || '',
        color: dayData.desc.day.color || '',
      })
    }

    // Add commemorations (skip ferias)
    for (const commemoration of dayData.desc?.commemorations || []) {
      if (commemoration?.desc && !isFeria(commemoration.desc)) {
        addFeastToComparison(allFeasts, commemoration.desc, calendar, {
          present: true,
          description: commemoration.desc,
          rank: commemoration.rank || '',
          color: commemoration.color || '',
        })
      }
    }
  }

  // For each feast, try to find it in other calendars using search
  for (const [feastName, comparison] of allFeasts.entries()) {
    await findFeastInOtherCalendars(feastName, comparison)
  }

  feastComparisons.value = Array.from(allFeasts.values()).sort((a, b) =>
    a.name.localeCompare(b.name),
  )
}

function addFeastToComparison(
  allFeasts: Map<string, FeastComparison>,
  feastName: string,
  calendar: string,
  data: any,
) {
  if (!allFeasts.has(feastName)) {
    allFeasts.set(feastName, {
      name: feastName,
      calendars: {},
      baseCalendar: calendar,
    })
  }

  const comparison = allFeasts.get(feastName)!
  comparison.calendars[calendar] = data
}

// Search for a feast in calendars where it wasn't found on this date
async function findFeastInOtherCalendars(feastName: string, comparison: FeastComparison) {
  const calendarsWithoutFeast = selectedCalendars.value.filter(
    (cal) => !comparison.calendars[cal]?.present,
  )

  for (const calendar of calendarsWithoutFeast) {
    try {
      const searchResults = await api.searchFeasts(calendar, feastName)

      if (searchResults.length > 0) {
        // Find the best match (highest score)
        const bestMatch = searchResults.reduce((best, current) =>
          (current.score || 0) > (best.score || 0) ? current : best,
        )

        if ((bestMatch.score || 0) > 0.9) {
          // Only if it's a good match
          comparison.calendars[calendar] = {
            present: false,
            description: bestMatch.description,
            rank: bestMatch.rank,
            color: bestMatch.color,
            date: bestMatch.date,
            transferred: comparison.baseCalendar !== calendar,
            rankChanged: comparison.calendars[comparison.baseCalendar!]?.rank !== bestMatch.rank,
          }
        }
      }
    } catch (err) {
      console.warn(`Failed to search for ${feastName} in ${calendar}:`, err)
    }
  }
}

// Get display status for a feast in a calendar
function getFeastStatus(comparison: FeastComparison, calendar: string): string {
  const calData = comparison.calendars[calendar]
  if (!calData) return 'absent'

  if (calData.present) {
    return 'present'
  } else if (calData.transferred) {
    return 'transferred'
  } else if (calData.rankChanged) {
    return 'rank-changed'
  } else {
    return 'found-elsewhere'
  }
}

// Watch for changes
watch([selectedDate, selectedCalendars], loadAllCalendarData, { deep: true })

onMounted(() => {
  loadAllCalendarData()
})
</script>

<template>
  <div class="nerd-view">
    <div class="content-section">
      <p class="date-display">{{ formattedDate }}</p>
      <p class="subtitle">
        Compare feasts and commemorations across calendars (excludes weekdays and optional BVM
        commemorations)
      </p>
    </div>

    <div v-if="isLoading" class="loading">⏳ Loading calendar data...</div>

    <div v-if="error" class="error">❌ {{ error }}</div>

    <div v-if="selectedCalendars.length === 0" class="no-calendars">
      📚 Please select at least one calendar to compare
    </div>

    <div v-if="feastComparisons.length > 0" class="comparison-results">
      <div class="legend">
        <h3>Legend</h3>
        <div class="legend-items">
          <span class="legend-item">✅ Present today</span>
          <span class="legend-item">📅 Transferred</span>
          <span class="legend-item">⭐ Rank changed</span>
          <span class="legend-item">🔍 Found elsewhere</span>
          <span class="legend-item">❌ Not found</span>
        </div>
      </div>

      <div class="feast-comparison-table" :style="{ '--min-table-width': minTableWidth + 'px' }">
        <div class="table-header" :style="{ gridTemplateColumns }">
          <div class="feast-name-col">Feast</div>
          <div v-for="calendar in selectedCalendars" :key="calendar" class="calendar-col">
            {{ calendar.toUpperCase() }}
          </div>
        </div>

        <div
          v-for="comparison in feastComparisons"
          :key="comparison.name"
          class="feast-row"
          :style="{ gridTemplateColumns }"
        >
          <div class="feast-name">{{ comparison.name }}</div>

          <div
            v-for="calendar in selectedCalendars"
            :key="calendar"
            class="feast-cell"
            :class="getFeastStatus(comparison, calendar)"
          >
            <div class="feast-status">
              <span class="status-icon">
                {{ getStatusIcon(getFeastStatus(comparison, calendar)) }}
              </span>
              <span class="status-label">
                {{ getStatusLabel(getFeastStatus(comparison, calendar)) }}
              </span>
            </div>

            <div v-if="comparison.calendars[calendar]" class="feast-details">
              <div
                v-if="comparison.calendars[calendar].color"
                class="color-bar"
                :style="{
                  backgroundColor: getColorValue(comparison.calendars[calendar].color || ''),
                }"
              ></div>

              <div class="feast-info">
                <div v-if="comparison.calendars[calendar].rank" class="rank">
                  Rank: {{ comparison.calendars[calendar].rank }}
                </div>
                <div
                  v-if="
                    comparison.calendars[calendar].date && !comparison.calendars[calendar].present
                  "
                  class="alt-date"
                >
                  Date: {{ comparison.calendars[calendar].date }}
                </div>
                <div
                  v-if="comparison.calendars[calendar].description !== comparison.name"
                  class="description"
                >
                  {{ comparison.calendars[calendar].description }}
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div v-else-if="!isLoading && selectedCalendars.length > 0" class="no-feasts">
      📅 No feasts found for comparison on {{ formattedDate }}
    </div>
  </div>
</template>

.style scoped>
<style scoped>
@import '../styles/liturgical.css';
.nerd-view {
  width: var(--layout-fixed-width-wide);
  max-width: 100vw; /* Fallback for very small screens */
  margin: 0 auto;
  padding: 0 var(--layout-padding);
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

.subtitle {
  font-size: 14px;
  font-style: italic;
  color: var(--text-secondary);
  margin-bottom: 16px;
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
  color: var(--error-color);
  background: var(--error-bg);
  border: 1px solid var(--error-border);
  border-radius: 8px;
}

.legend {
  background: var(--surface-secondary);
  padding: 16px;
  border-radius: 4px;
  margin-bottom: 24px;
  border: 1px solid var(--border-secondary);
}

.legend h3 {
  margin: 0 0 12px 0;
  color: var(--text-primary);
  font-size: 16px;
}

.legend-items {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
}

.legend-item {
  font-size: 14px;
  color: var(--text-secondary);
}

.feast-comparison-table {
  background: var(--surface-primary);
  border-radius: 4px;
  overflow: hidden;
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
  padding: 16px;
  font-weight: 600;
  color: var(--text-primary);
  border-right: 1px solid var(--border-primary);
  background: var(--surface-secondary);
}

.feast-cell {
  padding: 12px;
  border-right: 1px solid var(--border-primary);
  min-height: 80px;
}

.feast-cell:last-child {
  border-right: none;
}

.feast-cell.present {
  background: color-mix(in srgb, var(--success-color) 20%, transparent);
}

.feast-cell.transferred {
  background: color-mix(in srgb, var(--warning-color) 20%, transparent);
}

.feast-cell.rank-changed {
  background: color-mix(in srgb, var(--error-color) 20%, transparent);
}

.feast-cell.found-elsewhere {
  background: color-mix(in srgb, var(--info-color) 20%, transparent);
}

.feast-cell.absent {
  background: var(--surface-secondary);
}

.feast-status {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.status-icon {
  font-size: 16px;
}

.status-label {
  font-size: 12px;
  color: var(--text-primary);
  font-weight: 500;
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

.alt-date {
  color: var(--warning-color);
  margin-bottom: 4px;
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

  .legend-items {
    flex-direction: column;
    gap: 8px;
  }
}
</style>
