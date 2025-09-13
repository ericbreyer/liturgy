<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { useCalendarSelection } from '../composables/useCalendarSelection'
import { api, type DayInfo } from '../services/api'
import { getColorValue, isFeria, getCalendarName } from '../utils/liturgical'
import { addDays, daysBetween, formatDate, getCurrentDate } from '../utils/dateUtils'
import PageLayout from '../components/PageLayout.vue'
import LoadingSpinner from '../components/LoadingSpinner.vue'
import ErrorDisplay from '../components/ErrorDisplay.vue'
import LiturgicalColorBar from '../components/LiturgicalColorBar.vue'
import FeastMeta from '../components/FeastMeta.vue'

interface NovenaFeast {
  date: string
  title: string
  rank: string
  color: string
  calendars: string[] // Changed from single calendar to array
  daysAway: number
  novenaStartDate: string
}

interface NovenaCategory {
  title: string
  description: string
  feasts: NovenaFeast[]
  cssClass: string
}

const { selectedCalendars, selectedCalendarInfos } = useCalendarSelection()

const loading = ref(false)
const error = ref<string>('')
const novenas = ref<NovenaCategory[]>([])

// Novena timing setting: 8 days (modern) or 9 days (traditional)
const novenaDays = ref(8) // Default to 8 days as currently implemented

const todayString = getCurrentDate()
console.log('Today string for novenas:', todayString)

// Calculate days away for novena start
function getNovenaStartDaysAway(novenaStartDate: string): number {
  const today = new Date().toISOString().split('T')[0]
  return daysBetween(today, novenaStartDate)
}

// Get proper tense for novena start
function getNovenaStartText(novenaStartDate: string): string {
  const daysAway = getNovenaStartDaysAway(novenaStartDate)
  if (daysAway < 0) {
    return 'Novena Started' // Already started
  } else if (daysAway === 0) {
    return 'Novena Starts Today'
  } else {
    return 'Novena Starts'
  }
}

// Check if a date is a Sunday
function isSunday(dateStr: string): boolean {
  const [year, month, day] = dateStr.split('-').map(Number)
  const date = new Date(year, month - 1, day)
  return date.getDay() === 0
}

// Check if a feast is significant enough for a novena
function isNovenaWorthy(title: string, rank: string, date: string): boolean {
  // Filter out obvious ferias
  if (isFeria(title)) {
    return false
  }

  // For Sundays, only filter out if it's explicitly just ordinary Sunday
  if (isSunday(date)) {
    if (
      rank.toLowerCase().includes('sunday') ||
      rank.toLowerCase().includes('dominica') ||
      rank.toLowerCase().includes('ordinary') ||
      title.toLowerCase().includes('ordinary time')
    ) {
      return false
    }
  }

  // Must have some meaningful content
  if (!title.trim() || !rank.trim()) {
    return false
  }

  // Accept ANY feast that's not a feria or ordinary Sunday
  return true
}

// Load feast data for the next 21 days
async function loadNovenaData() {
  if (selectedCalendars.value.length === 0) {
    novenas.value = []
    return
  }

  console.log('Selected calendars:', selectedCalendars.value)
  loading.value = true
  error.value = ''

  try {
    const feastMap = new Map<string, NovenaFeast>() // Use Map for deduplication

    // Load data for the next 21 days across all selected calendars
    for (let i = 1; i <= 21; i++) {
      const date = addDays(todayString, i)
      const daysAway = i

      console.log(`Checking date: ${date} (${daysAway} days from ${todayString})`)

      for (const calendar of selectedCalendars.value) {
        console.log(`Processing calendar: ${calendar} for date ${date}`)
        try {
          const [year, month, day] = date.split('-').map(Number)
          const dayInfo = await api.getDayInfo(calendar, year, month, day)

          if (dayInfo.desc.commemorations && dayInfo.desc.commemorations.length > 0) {
            for (const commemoration of dayInfo.desc.commemorations) {
              if (isNovenaWorthy(commemoration.desc, commemoration.rank, date)) {
                const feastKey = `${date}-${commemoration.desc}`

                if (feastMap.has(feastKey)) {
                  // Add calendar to existing feast
                  const existingFeast = feastMap.get(feastKey)!
                  if (!existingFeast.calendars.includes(calendar)) {
                    existingFeast.calendars.push(calendar)
                    // Sort calendars for consistent display order
                    existingFeast.calendars.sort()
                  }
                } else {
                  // Create new feast
                  const feast: NovenaFeast = {
                    date,
                    title: commemoration.desc,
                    rank: commemoration.rank,
                    color: commemoration.color || 'white',
                    calendars: [calendar],
                    daysAway,
                    novenaStartDate: addDays(date, -novenaDays.value),
                  }
                  feastMap.set(feastKey, feast)
                }
              }
            }
          }

          // Also check the main day observance
          if (dayInfo.desc.day) {
            if (isNovenaWorthy(dayInfo.desc.day.desc, dayInfo.desc.day.rank, date)) {
              const feastKey = `${date}-${dayInfo.desc.day.desc}`

              if (feastMap.has(feastKey)) {
                // Add calendar to existing feast
                const existingFeast = feastMap.get(feastKey)!
                if (!existingFeast.calendars.includes(calendar)) {
                  existingFeast.calendars.push(calendar)
                  // Sort calendars for consistent display order
                  existingFeast.calendars.sort()
                }
              } else {
                // Create new feast
                const feast: NovenaFeast = {
                  date,
                  title: dayInfo.desc.day.desc,
                  rank: dayInfo.desc.day.rank,
                  color: dayInfo.desc.day.color || 'white',
                  calendars: [calendar],
                  daysAway,
                  novenaStartDate: addDays(date, -novenaDays.value),
                }
                feastMap.set(feastKey, feast)
              }
            }
          }
        } catch (err) {
          console.warn(`Failed to load data for ${date} in ${calendar}:`, err)
        }
      }
    }

    // Convert map to array
    const allFeasts = Array.from(feastMap.values())

    // Categorize the feasts (no overlap between categories)
    const currentNovenas = allFeasts.filter(
      (f) => f.daysAway <= novenaDays.value - 1 && f.daysAway >= 1,
    )
    const startToday = allFeasts.filter((f) => f.daysAway === novenaDays.value)
    const upcoming = allFeasts.filter(
      (f) => f.daysAway > novenaDays.value && f.daysAway <= novenaDays.value + 6,
    )

    novenas.value = [
      {
        title: 'Start Novenas Today',
        description: `Feasts exactly ${novenaDays.value} days away - begin these novenas today`,
        feasts: startToday,
        cssClass: 'start-today',
      },
      {
        title: 'Upcoming Novenas',
        description: `Feasts ${novenaDays.value + 1}-${
          novenaDays.value + 6
        } days away - prepare for these novenas`,
        feasts: upcoming,
        cssClass: 'upcoming-novenas',
      },
      {
        title: 'Novenas in Progress',
        description: `Feasts coming up in 1-${
          novenaDays.value - 1
        } days - novenas currently underway`,
        feasts: currentNovenas,
        cssClass: 'current-novenas',
      },
    ]
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to load novena data'
  } finally {
    loading.value = false
  }
}

// Watch for calendar changes
watch(
  selectedCalendars,
  () => {
    loadNovenaData()
  },
  { immediate: true },
)

// Watch for novena days setting changes
watch(novenaDays, () => {
  loadNovenaData()
})

onMounted(() => {
  loadNovenaData()
})
</script>

<template>
  <PageLayout>
    <div class="view-container">
      <div class="novena-settings">
        <div class="setting-group">
          <label class="setting-label">Novena Timing:</label>
          <div class="radio-group">
            <label class="radio-option">
              <input type="radio" :value="8" v-model="novenaDays" />
              <span>8 Days (Modern)</span>
              <small>Start novena 8 days before feast</small>
            </label>
            <label class="radio-option">
              <input type="radio" :value="9" v-model="novenaDays" />
              <span>9 Days (Traditional)</span>
              <small>Start novena 9 days before feast</small>
            </label>
          </div>
        </div>
      </div>

      <LoadingSpinner v-if="loading" />
      <ErrorDisplay v-else-if="error" :message="error" />

      <div v-else-if="selectedCalendars.length === 0" class="no-calendars">
        <p>Please select at least one calendar to see upcoming novenas.</p>
      </div>

      <div v-else class="novena-sections">
        <div
          v-for="category in novenas"
          :key="category.title"
          :class="['novena-category', category.cssClass]"
        >
          <div class="category-header">
            <h2>{{ category.title }}</h2>
            <p class="category-description">{{ category.description }}</p>
            <div class="feast-count">
              {{ category.feasts.length }} feast{{ category.feasts.length !== 1 ? 's' : '' }}
            </div>
          </div>

          <div v-if="category.feasts.length === 0" class="no-feasts">
            <p>No feasts in this category at the moment.</p>
          </div>

          <div v-else class="feasts-grid">
            <div
              v-for="feast in category.feasts"
              :key="`${feast.date}-${feast.title}-${feast.calendars.join('-')}`"
              class="feast-card"
            >
              <div class="feast-header">
                <LiturgicalColorBar :color="feast.color" size="medium" />
                <div class="feast-info">
                  <h3 class="feast-title">{{ feast.title }}</h3>
                  <FeastMeta :rank="feast.rank" :calendars="feast.calendars" size="medium" />
                </div>
              </div>

              <div class="feast-dates">
                <div class="feast-date">
                  <strong>Feast Date:</strong>
                  <span
                    >{{ formatDate(feast.date) }}
                    <span class="days-away"
                      >({{ feast.daysAway }} day{{ feast.daysAway !== 1 ? 's' : '' }} away)</span
                    ></span
                  >
                </div>

                <div class="novena-start">
                  <strong>{{ getNovenaStartText(feast.novenaStartDate) }}:</strong>
                  <span
                    >{{ formatDate(feast.novenaStartDate) }}
                    <span class="days-away"
                      >({{ Math.abs(getNovenaStartDaysAway(feast.novenaStartDate)) }} day{{
                        Math.abs(getNovenaStartDaysAway(feast.novenaStartDate)) !== 1 ? 's' : ''
                      }}
                      {{
                        getNovenaStartDaysAway(feast.novenaStartDate) < 0
                          ? 'ago'
                          : getNovenaStartDaysAway(feast.novenaStartDate) === 0
                            ? ''
                            : 'away'
                      }})</span
                    ></span
                  >
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </PageLayout>
</template>

<style scoped>
@import '../styles/liturgical.css';

.view-container {
  padding: 1rem;
  max-width: 1200px;
  margin: 0 auto;
}

.page-header {
  text-align: center;
  margin-bottom: 1rem;
}

.page-header h1 {
  color: var(--text-primary);
  margin-bottom: 0.5rem;
  border-bottom: 2px solid var(--accent-color);
  padding-bottom: 0.5rem;
  display: inline-block;
}

.page-description {
  color: var(--text-secondary);
  font-size: 1.1rem;
}

.novena-settings {
  background: var(--surface-secondary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 1rem;
  margin-bottom: 2rem;
}

.setting-group {
  display: flex;
  align-items: center;
  gap: 1rem;
  flex-wrap: wrap;
}

.setting-label {
  font-weight: 600;
  color: var(--text-primary);
  min-width: 120px;
}

.radio-group {
  display: flex;
  gap: 1.5rem;
  flex-wrap: wrap;
}

.radio-option {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
  padding: 0.5rem;
  border-radius: 4px;
  transition: background-color 0.2s;
}

.radio-option:hover {
  background: var(--surface-tertiary);
}

.radio-option input[type='radio'] {
  margin: 0;
  accent-color: var(--accent-color);
}

.radio-option span {
  font-weight: 500;
  color: var(--text-primary);
}

.radio-option small {
  color: var(--text-secondary);
  font-size: 0.85rem;
  margin-left: 0.25rem;
}

.no-calendars {
  text-align: center;
  color: var(--text-muted);
  padding: 2rem;
  background: var(--surface-secondary);
  border-radius: 8px;
  border: 1px solid var(--border-color);
}

.novena-sections {
  display: flex;
  flex-direction: column;
  gap: 2rem;
}

.novena-category {
  background: var(--surface-secondary);
  border-radius: 12px;
  padding: 1.5rem;
  border: 1px solid var(--border-color);
}

.novena-category.current-novenas {
  border-left: 4px solid var(--success-color);
}

.novena-category.start-today {
  border-left: 4px solid var(--warning-color);
}

.novena-category.upcoming-novenas {
  border-left: 4px solid var(--info-color);
}

.category-header {
  margin-bottom: 1.5rem;
}

.category-header h2 {
  color: var(--text-primary);
  margin-bottom: 0.5rem;
  font-size: 1.5rem;
}

.category-description {
  color: var(--text-secondary);
  margin-bottom: 0.5rem;
}

.feast-count {
  color: var(--text-muted);
  font-size: 0.9rem;
  font-style: italic;
}

.no-feasts {
  text-align: center;
  color: var(--text-muted);
  padding: 1rem;
  font-style: italic;
}

.feasts-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(350px, 1fr));
  gap: 1rem;
}

.feast-card {
  background: var(--surface-elevated);
  border-radius: 8px;
  padding: 1rem;
  border: 1px solid var(--border-secondary);
  transition: all 0.2s ease;
}

.feast-card:hover {
  background: var(--surface-interactive);
  border-color: var(--accent-color);
  transform: translateY(-2px);
  box-shadow: 0 4px 12px color-mix(in srgb, var(--accent-color) 20%, transparent);
}

.feast-header {
  display: flex;
  align-items: flex-start;
  gap: 0.75rem;
  margin-bottom: 1rem;
}

/* Custom novena-specific overrides */
.novena-section .feast-dates {
  margin-top: 0.75rem;
}

.feast-calendar {
  color: var(--text-muted);
}

.feast-dates {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  font-size: 0.9rem;
}

.feast-date,
.novena-start {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.feast-date strong,
.novena-start strong {
  color: var(--text-secondary);
  font-size: 0.8rem;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.feast-date span,
.novena-start span {
  color: var(--text-primary);
}

.days-away {
  color: var(--text-muted) !important;
  font-style: italic;
}

/* Responsive adjustments */
@media (max-width: 768px) {
  .view-container {
    padding: 0.5rem;
  }

  .feasts-grid {
    grid-template-columns: 1fr;
  }

  .feast-card {
    padding: 0.75rem;
  }

  .feast-dates {
    font-size: 0.85rem;
  }
}

/* Component-scoped category / novena / control styles (moved from liturgical.css)
   These are kept here because they apply only to the Novena view. */
.category-section {
  margin-bottom: 2rem;
}

.category-header {
  margin-bottom: 1rem;
  padding-bottom: 0.5rem;
  border-bottom: 2px solid var(--border-subtle);
}

.category-title {
  color: var(--text-primary);
  font-size: 1.25rem;
  font-weight: 600;
  margin-bottom: 0.25rem;
}

.category-description {
  color: var(--text-secondary);
  font-size: 0.9rem;
}

.category-count {
  color: var(--text-muted);
  font-size: 0.8rem;
  font-weight: 500;
  margin-left: 0.5rem;
}

/* Novena-specific styles */
.feast-dates {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  font-size: 0.9rem;
}

.feast-date,
.novena-start {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.days-away {
  color: var(--text-muted);
  font-size: 0.8rem;
  font-style: italic;
}

/* Controls and settings */
.controls-section {
  background: var(--surface-primary);
  padding: 1rem;
  border-radius: 8px;
  margin-bottom: 1.5rem;
  border: 1px solid var(--border-primary);
}

.control-group {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.5rem;
}

.control-group:last-child {
  margin-bottom: 0;
}

.toggle-switch {
  background: var(--surface-interactive);
  border: 1px solid var(--border-subtle);
  border-radius: 1rem;
  padding: 0.25rem 0.75rem;
  cursor: pointer;
  transition: all 0.2s ease;
  font-size: 0.85rem;
}

.toggle-switch:hover {
  border-color: var(--accent-color);
}

.toggle-switch.active {
  background: var(--accent-color);
  color: white;
  border-color: var(--accent-color);
}
</style>
