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

const weekInfoMap = ref<Record<string, Record<string, DayInfo>>>({}) // { date: { calendar: dayInfo } }
const loading = ref(false)
const backgroundLoading = ref(false)
const error = ref<string>('')
const hasLoadedOnce = ref(false)

// Keep track of what we're actually displaying vs what's being requested
const displayedDate = ref<string>('')
const displayedWeekData = ref<Record<string, Record<string, DayInfo>> | null>(null)
const displayedCalendars = ref<string[]>([])

// Abort controller for cancelling requests
const abortController = ref<AbortController | null>(null)

// Use shared calendar selection state
const {
  selectedCalendars,
  loadCalendars,
  selectedCalendarInfos
} = useCalendarSelection()

// Use shared date navigation
const {
  selectedDate,
  formattedDate,
  updateSelectedDate,
  goToToday,
  goToPrevious: goToPreviousWeek,
  goToNext: goToNextWeek,
  route
} = useDateNavigation('Week')

// Get week dates centered around displayed date
const weekDates = computed(() => {
  if (!displayedDate.value) return []
  
  const centerDateString = displayedDate.value
  const [year, month, day] = centerDateString.split('-').map(Number)
  const centerDate = new Date(year, month - 1, day)
  const dates = []
  
  for (let i = -3; i <= 3; i++) {
    const date = new Date(centerDate.getTime() + i * 24 * 60 * 60 * 1000)
    dates.push({
      dateString: date.toISOString().split('T')[0],
      displayDate: date.toLocaleDateString('en-US', { 
        weekday: 'short', 
        month: 'short', 
        day: 'numeric' 
      }),
      isSelected: date.toISOString().split('T')[0] === centerDateString
    })
  }
  return dates
})

// Get the displayed calendar infos
const displayedCalendarInfos = computed(() => {
  return selectedCalendarInfos.value.filter(cal => displayedCalendars.value.includes(cal.name))
})

async function loadWeekInfo() {
  // Prevent multiple simultaneous calls
  if (loading.value) {
    return
  }
  
  if (selectedCalendars.value.length === 0) {
    displayedWeekData.value = null
    displayedCalendars.value = []
    return
  }

  // Cancel previous request
  if (abortController.value) {
    abortController.value.abort()
  }
  abortController.value = new AbortController()
  
  try {
    loading.value = true
    const newWeekMap: Record<string, Record<string, DayInfo>> = {}
    
    // Compute dates around the NEW selectedDate, not the old displayedDate
    const [year, month, day] = selectedDate.value.split('-').map(Number)
    const centerDate = new Date(year, month - 1, day)
    
    for (let i = -3; i <= 3; i++) {
      const currentDate = new Date(centerDate.getTime() + i * 24 * 60 * 60 * 1000)
      const dateString = currentDate.toISOString().split('T')[0]
      const dayMap: Record<string, DayInfo> = {}
      
      for (const calendarName of selectedCalendars.value) {
        try {
          const [dayYear, dayMonth, dayDay] = dateString.split('-').map(Number)
          const dayInfo = await api.getDayInfo(calendarName, dayYear, dayMonth, dayDay)
          dayMap[calendarName] = dayInfo
        } catch (err: any) {
          if (err.name !== 'AbortError') {
            console.warn(`Could not load day info for ${calendarName} on ${dateString}:`, err)
          }
        }
      }
      
      if (Object.keys(dayMap).length > 0) {
        newWeekMap[dateString] = dayMap
      }
    }
    
    if (!abortController.value.signal.aborted) {
      // Only update displayed state when new data is successfully loaded
      displayedWeekData.value = newWeekMap
      displayedCalendars.value = [...selectedCalendars.value]
      displayedDate.value = selectedDate.value
      
      // Also update the original weekInfoMap for compatibility
      weekInfoMap.value = newWeekMap
    }
  } catch (err: any) {
    if (err.name !== 'AbortError') {
      error.value = err instanceof Error ? err.message : 'Could not load week info'
    }
  } finally {
    if (!abortController.value?.signal.aborted) {
      loading.value = false
    }
  }
}

// Watch for changes in selected calendars from shared state
watch(selectedCalendars, () => {
  loadWeekInfo()
}, { immediate: true, deep: true })

// Watch for route changes (date parameter changes)
watch(() => route.query.date, () => {
  loadWeekInfo()
})

// Watch for changes in selected date
watch(selectedDate, () => {
  loadWeekInfo()
})

onMounted(async () => {
  // Initialize displayed date to current selected date (like TodayView)
  displayedDate.value = selectedDate.value
  
  // Load the week info if calendars are selected
  if (selectedCalendars.value.length > 0) {
    loadWeekInfo()
  }
})
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
      <div v-if="loading" class="background-loading">
        <div class="loading-bar"></div>
      </div>

      <!-- Content with smooth transitions -->
      <div class="content-area" :class="{ 'loading-fade': loading }">
        <!-- Mobile Card View -->
        <div v-if="displayedCalendarInfos.length > 0 && displayedDate" class="mobile-view">
          <div class="day-cards">
            <LiturgicalDayCard
              v-for="dateInfo in weekDates"
              :key="dateInfo.dateString"
              :date-string="dateInfo.dateString"
              :display-date="dateInfo.displayDate"
              :day-data="displayedWeekData?.[dateInfo.dateString] || {}"
              :calendars="displayedCalendarInfos"
              :is-selected="dateInfo.isSelected"
              :show-date="true"
            />
          </div>
        </div>

        <!-- Desktop Table View -->
        <div v-if="displayedCalendarInfos.length > 0 && displayedDate" class="desktop-view">
          <LiturgicalTable 
            :dates="weekDates"
            :calendars="displayedCalendarInfos"
            :data-map="displayedWeekData || {}"
            :loading="false"
            date-column-title="Date"
          />
        </div>

        <div v-else-if="!loading && selectedCalendars.length === 0" class="no-selection">
          📋 Please select at least one calendar from the dropdown above to view the liturgy for this week
        </div>
      </div>
    </div>
  </PageLayout>
</template>

<style scoped>
.view-container {
  width: var(--layout-fixed-width);
  max-width: 100vw; /* Fallback for very small screens */
  margin: 0 auto;
  padding: 0 var(--layout-padding);
  box-sizing: border-box;
}

.view-header {
  background: var(--surface-primary);
  padding: 20px;
  border-radius: 8px;
  margin-bottom: 20px;
  border: 1px solid var(--border-primary);
}

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
  margin: 0 0 8px 0;
  font-size: 28px;
  font-weight: 400;
  display: flex;
  align-items: center;
  gap: 12px;
  color: var(--text-primary);
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

.day-cards {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.no-selection {
  text-align: center;
  padding: 40px 20px;
  color: var(--text-secondary);
  font-size: 16px;
  background: var(--surface-primary);
  border-radius: 4px;
  border: 1px solid var(--border-primary);
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
  
  .no-selection {
    padding: 32px 16px;
    font-size: 14px;
  }
}

@media (max-width: 480px) {
  .header-content {
    padding: 12px;
    border-radius: 8px;
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
  
  .day-cards {
    gap: 8px;
  }
}
</style>
