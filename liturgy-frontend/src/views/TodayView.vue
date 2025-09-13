<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue'
import { api, type DayInfo } from '../services/api'
import LiturgicalTable from '../components/LiturgicalTable.vue'
import LiturgicalDayCard from '../components/LiturgicalDayCard.vue'
import DateSelector from '../components/DateSelector.vue'
import ErrorDisplay from '../components/ErrorDisplay.vue'
import PageLayout from '../components/PageLayout.vue'
import SkeletonLoader from '../components/SkeletonLoader.vue'
import { useCalendarSelection } from '../composables/useCalendarSelection'
import { useDateNavigation } from '../composables/useDateNavigation'

const todayInfoMap = ref<Record<string, DayInfo>>({})
const loading = ref(false)
const backgroundLoading = ref(false)
const error = ref<string>('')

// Keep track of what we're actually displaying vs what's being requested
const displayedDate = ref<string>('')
const displayedData = ref<Record<string, DayInfo>>({})
const displayedCalendars = ref<string[]>([])

// Abort controller for cancelling requests
const abortController = ref<AbortController | null>(null)

// Use shared calendar selection state
const { selectedCalendars, loadCalendars, selectedCalendarInfos } = useCalendarSelection()

// Use shared date navigation
const {
  selectedDate,
  formattedDate,
  updateSelectedDate,
  goToToday,
  goToPrevious: goToPreviousDay,
  goToNext: goToNextDay,
  route,
} = useDateNavigation('Today')

// Create a single date array for the LiturgicalTable component using displayed data
const singleDateArray = computed(() => {
  if (!displayedDate.value) return []

  const [year, month, day] = displayedDate.value.split('-').map(Number)
  const date = new Date(year, month - 1, day)

  return [
    {
      dateString: displayedDate.value,
      displayDate: date.toLocaleDateString('en-US', {
        weekday: 'long',
        month: 'short',
        day: 'numeric',
        year: 'numeric',
      }),
      isSelected: true,
    },
  ]
})

// Convert displayed data to the format expected by LiturgicalTable
const dataMapForTable = computed(() => {
  if (!displayedDate.value) return {}

  return {
    [displayedDate.value]: displayedData.value,
  }
})

// Get the displayed calendar infos
const displayedCalendarInfos = computed(() => {
  return selectedCalendarInfos.value.filter((cal) => displayedCalendars.value.includes(cal.name))
})

// Watch for changes in selected calendars from shared state
watch(
  selectedCalendars,
  () => {
    loadDayInfo()
  },
  { immediate: false, deep: true },
)

// Watch for route changes (date parameter changes)
watch(
  () => route.query.date,
  () => {
    loadDayInfo()
  },
)

// Watch for changes in selected date
watch(selectedDate, () => {
  loadDayInfo()
})

// Watch for route changes (date parameter changes)
watch(
  () => route.query.date,
  () => {
    loadDayInfo()
  },
)

onMounted(async () => {
  // Initialize displayed date to current selected date
  displayedDate.value = selectedDate.value

  // Calendar loading is now handled by AppNavigation
  // Just load the day info if calendars are already selected
  if (selectedCalendars.value.length > 0) {
    loadDayInfo()
  }
})

async function loadDayInfo() {
  if (selectedCalendars.value.length === 0) return

  // Cancel any existing request
  if (abortController.value) {
    abortController.value.abort()
  }
  abortController.value = new AbortController()

  try {
    backgroundLoading.value = true
    loading.value = true
    error.value = ''

    const [year, month, day] = selectedDate.value.split('-').map(Number)
    const newMap: Record<string, DayInfo> = {}

    await Promise.all(
      selectedCalendars.value.map(async (calendarName) => {
        try {
          const dayInfo = await api.getDayInfo(calendarName, year, month, day)
          newMap[calendarName] = dayInfo
        } catch (err) {
          if (err instanceof Error && err.name === 'AbortError') {
            return // Request was cancelled
          }
          console.warn(`Could not load day info for ${calendarName}:`, err)
        }
      }),
    )

    // Only update displayed content if this request wasn't cancelled
    if (!abortController.value?.signal.aborted) {
      todayInfoMap.value = newMap
      displayedData.value = newMap
      displayedDate.value = selectedDate.value
      displayedCalendars.value = [...selectedCalendars.value]
    }
  } catch (err) {
    if (err instanceof Error && err.name !== 'AbortError') {
      error.value = err.message || 'Could not load day info'
    }
  } finally {
    loading.value = false
    backgroundLoading.value = false
    abortController.value = null
  }
}
</script>

<template>
  <PageLayout>
    <div class="view-container">
      <ErrorDisplay
        v-if="error"
        :message="error"
        type="error"
        :dismissible="true"
        @dismiss="error = ''"
      />

      <!-- Background loading indicator -->
      <div v-if="backgroundLoading" class="background-loading">
        <div class="loading-bar"></div>
      </div>

      <!-- Content with smooth transitions -->
      <div class="content-area" :class="{ 'loading-fade': backgroundLoading }">
        <!-- Mobile Card View -->
        <div v-if="displayedCalendarInfos.length > 0 && displayedDate" class="mobile-view">
          <LiturgicalDayCard
            :date-string="displayedDate"
            :display-date="singleDateArray[0]?.displayDate || ''"
            :day-data="displayedData"
            :calendars="displayedCalendarInfos"
            :is-selected="true"
            :show-date="true"
          />
        </div>

        <!-- Desktop Table View -->
        <div v-if="displayedCalendarInfos.length > 0 && displayedDate" class="desktop-view">
          <LiturgicalTable
            :dates="singleDateArray"
            :calendars="displayedCalendarInfos"
            :data-map="dataMapForTable"
            :loading="false"
            date-column-title="Date"
          />
        </div>

        <div v-else-if="selectedCalendars.length === 0" class="no-selection">
          📋 Please select at least one calendar from the dropdown above to view the liturgy for
          this day
        </div>

        <div v-else-if="!displayedDate && !backgroundLoading" class="no-selection">
          📅 Loading liturgical information...
        </div>
      </div>
    </div>
  </PageLayout>
</template>

<style scoped>
@import '../styles/liturgical.css';

/* TodayView-specific overrides (keep truly local rules here) */
.header-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 20px;
  background: transparent;
}

.header-text {
  flex: 1;
}

.header-title {
  color: var(--text-primary);
  margin: 0 0 8px 0;
  font-size: 28px;
  font-weight: 400;
  display: flex;
  align-items: center;
  gap: 12px;
}

.header-icon {
  font-size: 32px;
}

.header-subtitle {
  margin: 0;
  opacity: 0.9;
  font-size: 16px;
  font-weight: 300;
}

.header-controls {
  flex-shrink: 0;
}

.background-loading {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  z-index: 1000;
}

.loading-bar {
  height: 2px;
  background: var(--accent-color);
  animation: loading-slide 2s ease-in-out infinite;
}

.content-area {
  transition: opacity 0.3s ease-in-out;
}

.content-area.loading-fade {
  opacity: 0.7;
}

@keyframes loading-slide {
  0% {
    transform: translateX(-100%);
  }
  50% {
    transform: translateX(0%);
  }
  100% {
    transform: translateX(100%);
  }
}

/* Show mobile view on small screens, table on large screens */
.mobile-view {
  display: block;
}
.desktop-view {
  display: none;
}

@media (min-width: 768px) {
  .mobile-view {
    display: none;
  }
  .desktop-view {
    display: block;
  }
}

@media (max-width: 768px) {
  .header-content {
    flex-direction: column;
    align-items: stretch;
    gap: 16px;
    padding: 16px;
  }
  .header-title {
    font-size: 24px;
    text-align: center;
  }
  .header-icon {
    font-size: 28px;
  }
  .header-subtitle {
    font-size: 14px;
    text-align: center;
  }
}

@media (max-width: 480px) {
  .header-content {
    padding: 12px;
    border-radius: 4px;
  }
  .header-title {
    font-size: 20px;
  }
  .header-icon {
    font-size: 24px;
  }
  .header-subtitle {
    font-size: 13px;
  }
}

@media (max-width: 768px) {
  .calendar-checkboxes {
    flex-direction: column;
  }
  .checkbox-label {
    flex: 1;
  }
  .selection-buttons {
    flex-direction: column;
  }
}
</style>
