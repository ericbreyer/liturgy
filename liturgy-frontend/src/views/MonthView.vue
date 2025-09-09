<script setup lang="ts">
import { ref, onMounted, computed, watch, onUnmounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { api, type DayInfo } from '../services/api'
import DateSelector from '../components/DateSelector.vue'
import LoadingSpinner from '../components/LoadingSpinner.vue'
import ErrorDisplay from '../components/ErrorDisplay.vue'
import PageLayout from '../components/PageLayout.vue'
import LiturgicalColorBar from '../components/LiturgicalColorBar.vue'
import FeastMeta from '../components/FeastMeta.vue'
import { useCalendarSelection } from '../composables/useCalendarSelection'
import { useDateNavigation } from '../composables/useDateNavigation'
import { getColorValue, getCalendarName, getRankValue } from '../utils/liturgical'

const monthInfoMap = ref<Record<string, Record<string, DayInfo>>>({}) // { date: { calendar: dayInfo } }
const loading = ref(false)
const error = ref<string>('')
const selectedDetailDate = ref<string | null>(null)
const hoveredDate = ref<string | null>(null)

// Router for URL persistence
const router = useRouter()
const currentRoute = useRoute()

// Use shared calendar selection state
const {
  selectedCalendars,
  loadCalendars,
  selectedCalendarInfos,
  syncWithRoute
} = useCalendarSelection()

// Use shared date navigation
const {
  selectedDate,
  formattedDate,
  updateSelectedDate,
  goToToday,
  goToPrevious: goToPreviousMonth,
  goToNext: goToNextMonth,
  route
} = useDateNavigation('Month')

// Get month year and days for calendar grid
const monthData = computed(() => {
  const [year, month] = selectedDate.value.split('-').map(Number)
  const firstDay = new Date(year, month - 1, 1)
  const lastDay = new Date(year, month, 0)
  const daysInMonth = lastDay.getDate()
  
  // Get the first day of the week (0 = Sunday, 1 = Monday, etc.)
  const startDay = firstDay.getDay()
  
  // Create array of all days in the calendar grid
  const days = []
  
  // Add days from previous month to fill the grid
  const prevMonth = month === 1 ? 12 : month - 1
  const prevYear = month === 1 ? year - 1 : year
  const daysInPrevMonth = new Date(prevYear, prevMonth, 0).getDate()
  
  for (let i = startDay - 1; i >= 0; i--) {
    const day = daysInPrevMonth - i
    const date = new Date(prevYear, prevMonth - 1, day)
    const dateString = date.toISOString().split('T')[0]
    days.push({
      day,
      date: dateString,
      isToday: dateString === new Date().toISOString().split('T')[0],
      isSelected: dateString === selectedDate.value,
      isOtherMonth: true,
      isPrevMonth: true
    })
  }
  
  // Add all days of the current month
  for (let day = 1; day <= daysInMonth; day++) {
    const date = new Date(year, month - 1, day)
    const dateString = date.toISOString().split('T')[0]
    days.push({
      day,
      date: dateString,
      isToday: dateString === new Date().toISOString().split('T')[0],
      isSelected: dateString === selectedDate.value,
      isOtherMonth: false
    })
  }
  
  // Add days from next month to complete the grid (ensure we have full weeks)
  const nextMonth = month === 12 ? 1 : month + 1
  const nextYear = month === 12 ? year + 1 : year
  const totalCells = Math.ceil(days.length / 7) * 7
  
  for (let day = 1; days.length < totalCells; day++) {
    const date = new Date(nextYear, nextMonth - 1, day)
    const dateString = date.toISOString().split('T')[0]
    days.push({
      day,
      date: dateString,
      isToday: dateString === new Date().toISOString().split('T')[0],
      isSelected: dateString === selectedDate.value,
      isOtherMonth: true,
      isNextMonth: true
    })
  }
  
  return {
    year,
    month,
    days,
    monthName: firstDay.toLocaleDateString('en-US', { month: 'long' })
  }
})

// Get all dates we need to load for the month (only current month)
const monthDates = computed(() => {
  return monthData.value.days
    .filter(day => day && !day.isOtherMonth)
    .map(day => day!.date)
})

// Get liturgical info for a specific date and calendar
function getDayInfo(date: string, calendar: string): DayInfo | null {
  return monthInfoMap.value[date]?.[calendar] || null
}

// Get all feasts for a day across all selected calendars
function getDayFeasts(date: string): Array<{ title: string, rank: string, calendar: string, color: string, commemorationCount?: number }> {
  const dayData = monthInfoMap.value[date]
  if (!dayData) return []
  
  const feasts = []
  
  for (const [calendar, info] of Object.entries(dayData)) {
    if (!selectedCalendars.value.includes(calendar)) continue
    
    if (info.desc?.day) {
      const commemorationCount = info.desc.commemorations?.length || 0
      feasts.push({
        title: info.desc.day.desc,
        rank: info.desc.day.rank,
        calendar: calendar,
        color: info.desc.day.color || 'green',
        commemorationCount: commemorationCount > 0 ? commemorationCount : undefined
      })
    }
  }
  
  // Sort by rank priority
  return feasts.sort((a, b) => getRankValue(b.rank) - getRankValue(a.rank))
}

// Get commemoration interpretation for a calendar
function getCommemorationInterpretation(calendarName: string): string {
  const calendar = selectedCalendarInfos.value.find(cal => cal.name === calendarName)
  return calendar?.commemoration_interpretation || 'Commemorations'
}

// Handle day selection for detailed view
function selectDay(date: string) {
  selectedDetailDate.value = selectedDetailDate.value === date ? null : date
}

// Handle day hover
function hoverDay(date: string | null) {
  hoveredDate.value = date
}

// Close detail panel
function closeDetailPanel() {
  selectedDetailDate.value = null
}

// Handle escape key to close detail panel
function handleKeyDown(event: KeyboardEvent) {
  if (event.key === 'Escape' && selectedDetailDate.value) {
    closeDetailPanel()
  }
}

// Handle click outside detail panel
function handleClickOutside(event: MouseEvent) {
  const target = event.target as HTMLElement
  const detailPanel = document.querySelector('.detail-panel')
  const dayCell = target.closest('.calendar-day')
  
  // Don't close if clicking on a day cell or inside the detail panel
  if (selectedDetailDate.value && detailPanel && !detailPanel.contains(target) && !dayCell) {
    closeDetailPanel()
  }
}

// Add event listeners when detail panel is open
watch(selectedDetailDate, (newValue, oldValue) => {
  if (newValue && !oldValue) {
    // Panel opened, add listeners with a small delay to avoid immediate closing
    document.addEventListener('keydown', handleKeyDown)
    // Use setTimeout to add click listener after current event loop
    setTimeout(() => {
      document.addEventListener('click', handleClickOutside)
    }, 0)
  } else if (!newValue && oldValue) {
    // Panel closed, remove listeners
    document.removeEventListener('keydown', handleKeyDown)
    document.removeEventListener('click', handleClickOutside)
  }
})

// Get detailed day information for the detail panel
function getDetailedDayInfo(date: string) {
  const dayData = monthInfoMap.value[date]
  if (!dayData) return null
  
  const details = []
  for (const [calendar, info] of Object.entries(dayData)) {
    if (!selectedCalendars.value.includes(calendar)) continue
    
    details.push({
      calendar,
      info
    })
  }
  
  return details
}

// Load month data for all selected calendars
async function loadMonthData() {
  if (selectedCalendars.value.length === 0 || monthDates.value.length === 0) return
  
  loading.value = true
  error.value = ''
  
  try {
    monthInfoMap.value = {}
    
    // Load all dates for the month
    for (const date of monthDates.value) {
      monthInfoMap.value[date] = {}
      
      for (const calendar of selectedCalendars.value) {
        try {
          const [year, month, day] = date.split('-').map(Number)
          const dayInfo = await api.getDayInfo(calendar, year, month, day)
          monthInfoMap.value[date][calendar] = dayInfo
        } catch (err) {
          console.warn(`Failed to load data for ${date} in ${calendar}:`, err)
        }
      }
    }
  } catch (err) {
    console.error('Error loading month data:', err)
    error.value = err instanceof Error ? err.message : 'Could not load month info'
  } finally {
    loading.value = false
  }
}

// Watch for changes in selected calendars from shared state
watch(selectedCalendars, () => {
  loadMonthData()
}, { immediate: false, deep: true })

// Watch for changes in the selected date
watch(selectedDate, () => {
  loadMonthData()
}, { immediate: true })

// Load calendars on component mount
onMounted(async () => {
  // Load calendars with URL persistence
  await loadCalendars(router, currentRoute)
  
  // Sync with route changes
  if (syncWithRoute) {
    syncWithRoute(currentRoute)
  }
  
  // Load the month data if calendars are selected
  if (selectedCalendars.value.length > 0) {
    loadMonthData()
  }
})

// Cleanup event listeners on unmount
onUnmounted(() => {
  document.removeEventListener('keydown', handleKeyDown)
  document.removeEventListener('click', handleClickOutside)
})
</script>

<template>
  <PageLayout>
    <div class="view-container">
      <!-- Loading/Error States -->
      <LoadingSpinner 
        v-if="loading" 
        text="Loading month data..."
        size="medium"
      />
      
      <ErrorDisplay 
        v-if="error" 
        :message="error"
        type="error"
        :dismissible="true"
        @dismiss="error = ''"
      />

      <!-- No calendars selected message -->
      <div v-if="!loading && !error && selectedCalendars.length === 0" class="no-selection">
        📋 Please select at least one calendar from the dropdown above to view the monthly liturgy
      </div>

      <!-- Monthly Calendar Grid -->
      <div v-if="!loading && !error && selectedCalendars.length > 0" class="month-layout">
      <div class="calendar-grid">
        <!-- Month/Year Header -->
        <div class="calendar-header">
          <h2>{{ monthData.monthName }} {{ monthData.year }}</h2>
        </div>

        <!-- Day of week headers -->
        <div class="weekday-headers">
          <div class="weekday-header">Sun</div>
          <div class="weekday-header">Mon</div>
          <div class="weekday-header">Tue</div>
          <div class="weekday-header">Wed</div>
          <div class="weekday-header">Thu</div>
          <div class="weekday-header">Fri</div>
          <div class="weekday-header">Sat</div>
        </div>

        <!-- Calendar days -->
        <div class="calendar-days">
          <div
            v-for="(day, index) in monthData.days"
            :key="index"
            :class="[
              'calendar-day',
              {
                'other-month': day?.isOtherMonth,
                'today': day?.isToday,
                'selected': day?.isSelected,
                'detail-selected': day && selectedDetailDate === day.date,
                'hovered': day && hoveredDate === day.date,
                'has-feast': day && !day.isOtherMonth && getDayFeasts(day.date).length > 0
              }
            ]"
            @click="day && !day.isOtherMonth && selectDay(day.date)"
            @mouseenter="day && !day.isOtherMonth && hoverDay(day.date)"
            @mouseleave="hoverDay(null)"
          >
            <div v-if="day" class="day-content">
              <div class="day-number">
                {{ day.day }}
                <div class="day-name mobile-only">
                  {{ new Date(day.date).toLocaleDateString('en-US', { weekday: 'short' }) }}
                </div>
              </div>
              
              <!-- Simple feast display with liturgical colors (only for current month) -->
              <div v-if="!day.isOtherMonth && getDayFeasts(day.date).length > 0" class="day-feasts">
                <div
                  v-for="(feast, idx) in getDayFeasts(day.date)"
                  :key="idx"
                  class="feast-line"
                >
                  <LiturgicalColorBar :color="feast.color" size="small" />
                  <div class="feast-content">
                    <div class="feast-title-row">
                      <div class="feast-title">
                        {{ feast.title }}
                      </div>
                      <span v-if="feast.commemorationCount" class="commemoration-count">+{{ feast.commemorationCount }}</span>
                    </div>
                    <FeastMeta 
                      :rank="feast.rank" 
                      :calendars="feast.calendar"
                      size="small"
                    />
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Detailed Day View Panel -->
    <div v-if="selectedDetailDate" class="detail-panel">
      <div class="detail-header">
        <h3>{{ new Date(selectedDetailDate).toLocaleDateString('en-US', { 
          weekday: 'long', 
          year: 'numeric', 
          month: 'long', 
          day: 'numeric' 
        }) }}</h3>
        <button @click="closeDetailPanel" class="close-button">×</button>
      </div>
      
      <div class="detail-content">
        <div v-if="getDetailedDayInfo(selectedDetailDate)" class="calendar-details">
          <div
            v-for="{ calendar, info } in getDetailedDayInfo(selectedDetailDate)"
            :key="calendar"
            class="calendar-detail"
          >
            <h4 class="calendar-name">{{ calendar.toUpperCase() }}</h4>
            
            <!-- Main feast/day info -->
            <div v-if="info.desc?.day" class="feast-detail">
              <div class="feast-name">
                {{ info.desc.day.desc }}
                <span v-if="info.desc?.commemorations && info.desc.commemorations.length > 0" class="commemoration-count">
                  +{{ info.desc.commemorations.length }}
                </span>
              </div>
              <div class="feast-rank">{{ info.desc.day.rank }}</div>
              <div v-if="info.desc.day.color" class="feast-color" :style="{ backgroundColor: info.desc.day.color }">
                {{ info.desc.day.color }}
              </div>
            </div>
            
            <!-- Season info -->
            <div v-if="info.desc?.day_in_season" class="season-info">
              <h5>Season</h5>
              <div>{{ info.desc.day_in_season }}</div>
            </div>
            
            <!-- Additional commemorations -->
            <div v-if="info.desc?.commemorations && info.desc.commemorations.length > 0" class="commemorations">
              <h5>{{ getCommemorationInterpretation(calendar) }}</h5>
              <ul>
                <li v-for="comm in info.desc.commemorations" :key="comm.desc">
                  <div class="commemoration-name">{{ comm.desc }}</div>
                  <div class="commemoration-rank">{{ comm.rank }}</div>
                </li>
              </ul>
            </div>
          </div>
        </div>
        
        <div v-else class="no-detail">
          No detailed information available for this date.
        </div>
      </div>
    </div>
    </div>
  </PageLayout>
</template>

<style scoped>
.view-header {
  background: var(--surface-primary);
  padding: 20px;
  border-radius: 8px;
  margin-bottom: 20px;
  border: 1px solid var(--border-primary);
}

.view-header .header-content {
  background: transparent;
}

.view-header h1, 
.view-header .header-title {
  color: var(--text-primary);
}

.view-header p,
.view-header .header-subtitle {
  color: var(--text-secondary);
}

.month-view {
  padding: 0.5rem;
  max-width: 100%;
  margin: 0 auto;
  overflow-x: auto;
}

@media (max-width: 768px) {
  .month-view {
    padding: 0.25rem;
  }
}

.month-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
  flex-wrap: wrap;
  gap: 1rem;
}

.month-header h1 {
  margin: 0;
  font-size: 1.5rem;
}

.calendar-controls {
  margin-bottom: 1rem;
}

.loading, .error {
  text-align: center;
  padding: 2rem;
}

.message {
  padding: 1rem;
  margin-bottom: 1rem;
  border-radius: 0.375rem;
  color: var(--error-text);
  background-color: var(--error-bg);
  border: 1px solid var(--error-border);
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

.month-layout {
  display: flex;
  gap: var(--layout-gap);
  align-items: flex-start;
  width: 100%;
  max-width: 100vw;
  margin: 0 auto;
  padding: 0 var(--layout-padding);
  box-sizing: border-box;
  overflow-x: hidden;
}

@media (max-width: 768px) {
  .month-layout {
    flex-direction: column;
  }
  
  .calendar-grid {
    width: 100%;
  }
  
  .month-header {
    flex-direction: column;
    align-items: stretch;
    gap: 0.75rem;
  }
  
  .month-header h1 {
    text-align: center;
    font-size: 1.25rem;
  }
}

.calendar-grid {
  background: var(--surface-secondary);
  border-radius: 8px;
  border: 1px solid var(--border-color);
  overflow: hidden;
  flex: 1;
  min-width: 0;
  width: 100%;
  max-width: 100%;
}

.calendar-header {
  background-color: var(--surface-secondary);
  padding: 1rem;
  text-align: center;
  border-bottom: 1px solid var(--border-secondary);
}

.calendar-header h2 {
  margin: 0;
  color: var(--text-primary);
}

.weekday-headers {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  background-color: var(--surface-interactive);
  color: var(--text-primary);
  width: 100%;
  box-sizing: border-box;
}

.weekday-header {
  padding: 0.75rem 0.5rem;
  text-align: center;
  font-weight: 600;
  font-size: 0.875rem;
  border-right: 1px solid var(--border-subtle);
  overflow: hidden;
  text-overflow: ellipsis;
}

.weekday-header:last-child {
  border-right: none;
}

/* Mobile landscape and small tablets */
@media (max-width: 767px) and (min-width: 640px) {
  .weekday-headers {
    grid-template-columns: repeat(7, minmax(120px, 1fr));
    max-width: 100%;
  }
}

/* Mobile portrait: Hide weekday headers when stacked */
@media (max-width: 639px) {
  .weekday-headers {
    display: none;
  }
}

.weekday-header {
  padding: 0.75rem;
  text-align: center;
  font-weight: 600;
  font-size: 0.875rem;
}

/* 
  RESPONSIVE CALENDAR GRID SYSTEM
  
  This calendar uses fixed dimensions with media queries to prevent "jumpiness" 
  when switching between months or views with different content amounts.
  
  Breakpoints:
  - 1200px+: Large desktop (180px columns, 160px height)
  - 768-1199px: Tablet landscape (140px+ columns, 120px height) 
  - 640-767px: Mobile landscape (120px+ columns, 100px height)
  - <640px: Mobile portrait (stacked vertically, auto height)
  
  Fixed dimensions prevent layout shifts based on content while 
  maintaining responsive behavior based on screen size.
*/

.calendar-days {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 1px;
  width: 100%;
  box-sizing: border-box;
  background-color: var(--border-primary);
}

/* Mobile portrait: Stack days vertically for better readability */
@media (max-width: 639px) {
  .calendar-days {
    grid-template-columns: 1fr;
    gap: 0.5rem;
    max-width: 100%;
    padding: 0 0.5rem;
  }
  
  .calendar-day {
    height: auto;
    min-height: 80px;
    padding: 0.75rem;
    border-radius: 0.375rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  
  .calendar-day.other-month {
    display: none;
  }
}

/* Day content styles - applies to all screen sizes */
.day-content {
  flex-direction: row;
  align-items: flex-start;
  gap: 1rem;
}

.day-number {
  font-size: 1.25rem;
  font-weight: 700;
  min-width: 2rem;
  flex-shrink: 0;
}

.day-feasts {
  flex: 1;
  overflow: hidden;
}

.feast-line {
  margin-bottom: 0.15rem;
  overflow: hidden;
  display: flex;
  align-items: flex-start;
  gap: 0.25rem;
}

.feast-content {
  overflow: hidden;
  min-width: 0;
  flex: 1;
}

.feast-title-row {
  display: flex;
  align-items: flex-start;
  gap: 0.25rem;
  margin-bottom: 0.05rem;
}

.feast-title {
  font-size: 0.625rem;
  line-height: 1.1;
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  line-clamp: 2;
  -webkit-box-orient: vertical;
  word-break: break-word;
  flex: 1;
  min-width: 0;
}

.feast-meta {
  font-size: 0.5rem;
  color: var(--text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  display: flex;
  gap: 0.25rem;
}

/* Responsive text sizing */
@media (min-width: 1200px) {
  .feast-title {
    font-size: 0.75rem;
    -webkit-line-clamp: 3;
    line-clamp: 3;
  }
  
  .feast-meta {
    font-size: 0.6rem;
  }
  
  .commemoration-count {
    font-size: 0.6rem;
  }
}

@media (max-width: 767px) {
  .feast-title {
    font-size: 0.55rem;
    -webkit-line-clamp: 1;
    line-clamp: 1;
    white-space: nowrap;
    text-overflow: ellipsis;
    display: block;
  }
  
  .feast-meta {
    font-size: 0.45rem;
  }
  
  .commemoration-count {
    font-size: 0.45rem;
    padding: 0.0625rem 0.125rem;
  }
}

.feast-meta {
  font-size: 0.6rem;
  color: var(--text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  display: flex;
  gap: 0.25rem;
}

.commemoration-count {
  font-size: 0.5rem;
  font-weight: 600;
  color: var(--text-muted);
  background: var(--bg-lighter);
  padding: 0.0625rem 0.1875rem;
  border-radius: 0.1875rem;
  white-space: nowrap;
  flex-shrink: 0;
  align-self: flex-start;
}

.calendar-day {
  min-height: 100px;
  background-color: var(--surface-secondary);
  cursor: pointer;
  transition: all 0.2s ease;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  border: none;
}

.day-content {
  padding: 0.25rem;
  height: 100%;
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
  overflow: hidden;
}

.day-number {
  font-weight: 600;
  color: var(--text-primary);
  font-size: 0.75rem;
  margin-bottom: 0.05rem;
  flex-shrink: 0;
}

/* Responsive sizing */
@media (min-width: 1200px) {
  .calendar-day {
    min-height: 120px;
  }
  
  .day-content {
    padding: 0.375rem;
  }
  
  .day-number {
    font-size: 0.875rem;
  }
}

@media (max-width: 767px) {
  .calendar-day {
    min-height: 80px;
  }
  
  .day-content {
    padding: 0.2rem;
  }
  
  .day-number {
    font-size: 0.7rem;
  }
}

.calendar-day:hover:not(.other-month) {
  background-color: var(--surface-elevated);
}

.calendar-day.hovered {
  background-color: var(--surface-interactive);
}

.calendar-day.detail-selected {
  background-color: var(--surface-interactive);
  border-color: var(--accent-color);
  border-width: 2px;
}

.calendar-day.other-month {
  cursor: default;
  background-color: var(--surface-disabled);
}

.calendar-day.other-month .day-number {
  color: var(--text-disabled);
}

.calendar-day.other-month:hover {
  background-color: var(--surface-primary);
}

.calendar-day.today {
  background-color: var(--surface-secondary);
  border-color: var(--accent-color);
}

.calendar-day.selected {
  background-color: var(--surface-elevated);
  border-color: var(--accent-color);
  border-width: 2px;
}

.calendar-day.has-feast {
  background-color: var(--surface-primary);
}

.day-content {
  padding: 0.4rem;
  height: 100%;
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
  overflow: hidden;
}

.day-number {
  font-weight: 600;
  color: var(--text-primary);
  font-size: 0.85rem;
  margin-bottom: 0.1rem;
  flex-shrink: 0;
}

.day-name {
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--text-secondary);
  margin-top: 0.125rem;
}

.mobile-only {
  display: none;
}

@media (max-width: 640px) {
  .mobile-only {
    display: block;
  }
}

.day-feasts {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}

/* Feast display with liturgical color bars */
.feast-line {
  display: flex;
  align-items: stretch;
  gap: 0.25rem;
  font-size: 0.7rem;
  line-height: 1.2;
  min-height: 1.2rem;
}

.liturgical-color-bar {
  width: 6px;
  border-radius: 2px;
  flex-shrink: 0;
  border: 1px solid var(--border-secondary);
}

.feast-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
  min-width: 0;
}

.feast-title {
  color: var(--text-primary);
  font-weight: 500;
  line-height: 1.1;
  word-break: break-word;
}

.feast-meta {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  margin-top: 0.25rem;
}

.feast-rank {
  font-size: 0.55rem;
  color: var(--text-secondary);
  font-weight: 500;
  background-color: var(--surface-secondary);
  padding: 1px 4px;
  border-radius: 2px;
  white-space: nowrap;
  line-height: 1.1;
}

.calendar-initial {
  background-color: var(--surface-interactive);
  color: var(--text-primary);
  font-size: 0.55rem;
  font-weight: 500;
  padding: 1px 4px;
  border-radius: 2px;
  line-height: 1.1;
  min-width: 20px;
  text-align: center;
}

.more-feasts {
  font-size: 0.5rem;
  color: var(--text-secondary);
  font-weight: 500;
  text-align: center;
  margin-top: 0.125rem;
  background-color: var(--surface-secondary);
  padding: 1px 4px;
  border-radius: 2px;
}

.calendar-indicator {
  background-color: var(--surface-interactive);
  color: var(--text-primary);
  font-size: 0.5rem;
  font-weight: 500;
  padding: 1px 2px;
  border-radius: 2px;
  line-height: 1;
  min-width: 12px;
  text-align: center;
}

/* Detail Panel Styles */
.detail-panel {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  background: var(--surface-primary);
  border-radius: 4px;
  border: 1px solid var(--border-primary);
  max-width: 500px;
  width: 90vw;
  max-height: 70vh;
  overflow: hidden;
  z-index: 1000;
}

.detail-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem;
  background-color: var(--surface-secondary);
  border-bottom: 1px solid var(--border-secondary);
}

.detail-header h3 {
  margin: 0;
  color: var(--text-primary);
  font-size: 1.1rem;
}

.close-button {
  background: none;
  border: none;
  font-size: 1.5rem;
  cursor: pointer;
  color: var(--text-muted);
  padding: 0;
  width: 2rem;
  height: 2rem;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  transition: background-color 0.2s ease;
}

.close-button:hover {
  background-color: var(--surface-elevated);
}

.detail-content {
  padding: 1rem;
  overflow-y: auto;
  max-height: calc(70vh - 4rem);
}

.calendar-details {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.calendar-detail {
  border: 1px solid var(--border-secondary);
  border-radius: 0.375rem;
  padding: 1rem;
  background-color: var(--surface-primary);
}

.calendar-name {
  margin: 0 0 1rem 0;
  color: var(--text-primary);
  font-size: 1rem;
  font-weight: 600;
  border-bottom: 1px solid var(--border-secondary);
  padding-bottom: 0.5rem;
}

.feast-detail {
  margin-bottom: 1rem;
}

.feast-name {
  font-size: 1.1rem;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 0.25rem;
}

.feast-rank {
  font-size: 0.55rem;
  color: var(--text-secondary);
}

.feast-color {
  display: inline-block;
  padding: 0.25rem 0.5rem;
  border-radius: 0.25rem;
  color: var(--text-primary);
  font-size: 0.8rem;
  text-shadow: 1px 1px 2px var(--background-color);
}

.season-info {
  margin-bottom: 1rem;
}

.season-info h5 {
  margin: 0 0 0.5rem 0;
  color: var(--text-secondary);
  font-size: 0.9rem;
}

.season-info div {
  color: var(--text-muted);
  font-size: 0.9rem;
}

.commemorations h5 {
  margin: 0 0 0.5rem 0;
  color: var(--text-secondary);
  font-size: 0.9rem;
}

.commemorations ul {
  margin: 0;
  padding-left: 1rem;
}

.commemorations li {
  margin-bottom: 0.5rem;
}

.commemoration-name {
  font-weight: 500;
  color: var(--text-primary);
}

.commemoration-rank {
  font-size: 0.8rem;
  color: var(--text-muted);
}

.no-detail {
  text-align: center;
  color: var(--text-muted);
  padding: 2rem;
}

@media (max-width: 768px) {
  .month-view {
    padding: 0.5rem;
  }
  
  .month-layout {
    flex-direction: column;
  }
  
  .month-header {
    flex-direction: column;
    align-items: stretch;
  }
  
  .month-header h1 {
    font-size: 1.25rem;
    text-align: center;
  }
  
  .detail-panel {
    width: 95vw;
    max-height: 80vh;
  }
  
  .calendar-days {
    min-width: 280px;
  }
  
  .calendar-day {
    min-height: 80px;
  }
  
  .day-content {
    padding: 0.25rem;
    gap: 0.125rem;
  }
  
  .day-number {
    font-size: 0.8rem;
  }
  
  .feast-title {
    font-size: 0.6rem;
  }
  
  .feast-rank {
    font-size: 0.55rem;
  }
  
  .weekday-header {
    padding: 0.25rem;
    font-size: 0.7rem;
  }
  
  .calendar-pip,
  .calendar-indicator {
    font-size: 0.45rem;
    padding: 0px 1px;
    min-width: 10px;
  }
}

@media (max-width: 480px) {
  .month-view {
    padding: 0.25rem;
  }
  
  .calendar-days {
    min-width: 260px;
  }
  
  .calendar-day {
    min-height: 70px;
  }
  
  .day-content {
    padding: 0.125rem;
  }
  
  .day-number {
    font-size: 0.75rem;
    margin-bottom: 0.125rem;
  }
  
  .feast-title {
    font-size: 0.55rem;
  }
  
  .feast-rank {
    font-size: 0.5rem;
    padding: 0px 2px;
  }
  
  .weekday-header {
    padding: 0.125rem;
    font-size: 0.65rem;
  }
  
  .calendar-pip,
  .calendar-indicator {
    font-size: 0.4rem;
    min-width: 8px;
  }
}

@media (max-width: 320px) {
  .calendar-days {
    min-width: 240px;
  }
  
  .calendar-day {
    min-height: 60px;
  }
  
  .day-number {
    font-size: 0.7rem;
  }
  
  .feast-title {
    font-size: 0.5rem;
  }
  
  .feast-rank {
    font-size: 0.45rem;
  }
  
  .weekday-header {
    font-size: 0.6rem;
  }
  
  .calendar-pip,
  .calendar-indicator {
    font-size: 0.35rem;
    min-width: 6px;
  }
}
</style>
