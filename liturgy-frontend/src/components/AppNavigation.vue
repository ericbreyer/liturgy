<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import CalendarSelection from './CalendarSelection.vue'
import DateSelector from './DateSelector.vue'
import { useCalendarSelection } from '../composables/useCalendarSelection'
import { useDateNavigation } from '../composables/useDateNavigation'

const route = useRoute()
const router = useRouter()

// Initialize calendar selection
const { loadCalendars, syncWithRoute } = useCalendarSelection()

interface NavItem {
  id: string
  label: string
  icon: string
  route: string
}

const navItems: NavItem[] = [
  { id: 'today', label: 'Daily View', icon: '', route: '/today' },
  { id: 'week', label: 'Week View', icon: '', route: '/week' },
  { id: 'month', label: 'Month View', icon: '', route: '/month' },
  { id: 'search', label: 'Search', icon: '', route: '/search' },
  { id: 'novena', label: 'Novenas', icon: '', route: '/novena' },
  { id: 'nerd', label: 'Advanced', icon: '', route: '/nerd' },
  { id: 'about', label: 'About', icon: '', route: '/about' }
]

// Create route object with current query params preserved
function getRouteWithQuery(path: string) {
  return {
    path,
    query: route.query
  }
}

// Load calendars when the navigation component mounts
onMounted(async () => {
  await loadCalendars(router, route)
  
  // Sync with route changes
  if (syncWithRoute) {
    syncWithRoute(route)
  }
})

// Date navigation - only show for date-based views
const showDatePicker = computed(() => {
  return ['Today', 'Week', 'Month', 'Nerd'].includes(route.name as string)
})

// Get the appropriate date navigation based on current route
const dateNavigation = computed(() => {
  const routeName = route.name as string
  if (['Today', 'Week', 'Month', 'Nerd'].includes(routeName)) {
    return useDateNavigation(routeName as 'Today' | 'Week' | 'Month' | 'Nerd')
  }
  return null
})

// Date picker variant based on route
const datePickerVariant = computed(() => {
  const routeName = route.name as string
  switch (routeName) {
    case 'Today': return 'day'
    case 'Week': return 'week'
    case 'Month': return 'month'
    case 'Nerd': return 'day'
    default: return 'day'
  }
})
</script>

<script lang="ts">
// Provide a runtime default export for environments/tools that expect one
// (keeps compatibility with older import behaviors / typecheckers)
export default {}
</script>

<template>
  <nav class="app-nav">
    <div class="nav-container">
      <!-- Top Row: Navigation Links (always stable) -->
      <div class="nav-top-row">
        <div class="nav-links">
          <router-link
            v-for="item in navItems"
            :key="item.id"
            :to="getRouteWithQuery(item.route)"
            class="nav-item"
            active-class="active"
          >
            <span class="nav-icon">{{ item.icon }}</span>
            <span class="nav-label">{{ item.label }}</span>
          </router-link>
        </div>
      </div>
      
      <!-- Bottom Row: Date Picker and Calendar Selection -->
      <div class="nav-bottom-row">
        <!-- Date Picker (always present but invisible when not needed) -->
        <div class="nav-date-picker" :class="{ 'invisible': !showDatePicker || !dateNavigation }">
          <DateSelector
            v-if="dateNavigation"
            :model-value="dateNavigation.selectedDate.value"
            @update:model-value="dateNavigation.updateSelectedDate"
            @go-previous="dateNavigation.goToPrevious"
            @go-next="dateNavigation.goToNext"
            @go-today="dateNavigation.goToToday"
            :variant="datePickerVariant"
            :show-title="false"
          />
        </div>
        
        <!-- Calendar Selection -->
        <div class="nav-calendar-selection">
          <CalendarSelection 
            :show-title="false" 
            variant="dropdown"
            title="Calendars"
          />
        </div>
      </div>
    </div>
  </nav>
</template>

<style scoped>
.app-nav {
  background: var(--surface-primary);
  border-bottom: 1px solid var(--border-color);
  padding: 0 20px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  position: relative;
  z-index: 10;
}

.nav-container {
  display: flex;
  flex-direction: column;
  gap: 12px;
  max-width: var(--layout-fixed-width-wide);
  margin: 0 auto;
  padding: 16px 0;
}

.nav-top-row {
  display: flex;
  justify-content: center;
  width: 100%;
}

.nav-bottom-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 20px;
  width: 100%;
}

.nav-links {
  display: flex;
  gap: 4px;
  background: var(--surface-secondary);
  border-radius: 12px;
  padding: 6px;
  justify-content: center;
  width: 100%;
  max-width: 800px;
}

.nav-date-picker {
  display: flex;
  align-items: center;
  justify-content: center;
  flex: 1;
}

.nav-date-picker.invisible {
  visibility: hidden;
}

.nav-calendar-selection {
  flex-shrink: 0;
  min-width: 200px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 20px;
  background: none;
  border: none;
  cursor: pointer;
  border-radius: 8px;
  transition: background-color 0.2s ease, color 0.2s ease;
  text-decoration: none;
  color: var(--text-secondary);
  font-family: inherit;
  font-size: 14px;
  font-weight: 500;
  position: relative;
  flex: 1;
  justify-content: center;
  white-space: nowrap;
  min-width: fit-content;
}

.nav-item:hover {
  background: var(--surface-primary);
  color: var(--text-primary);
}

.nav-item.active {
  background: var(--surface-primary);
  color: var(--text-primary);
  border: 2px solid var(--accent-color);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
}

.nav-icon {
  font-size: 16px;
  flex-shrink: 0;
}

.nav-label {
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

@media (max-width: 768px) {
  .nav-container {
    gap: 16px;
    padding: 12px 0;
  }
  
  .nav-bottom-row {
    flex-direction: column;
    gap: 12px;
  }
  
  .nav-date-picker {
    order: 1;
    justify-content: center;
  }
  
  .nav-calendar-selection {
    order: 2;
    width: 100%;
    max-width: 400px;
    min-width: unset;
  }
  
  .nav-item {
    flex-direction: column;
    gap: 4px;
    text-align: center;
    padding: 10px 12px;
  }
  
  .nav-label {
    font-size: 12px;
    font-weight: 400;
  }
  
  .nav-icon {
    font-size: 14px;
  }
}

@media (max-width: 480px) {
  .app-nav {
    padding: 0 12px;
  }
  
  .nav-container {
    gap: 8px;
    padding: 8px 0;
  }
  
  .nav-label {
    display: none;
  }
  
  .nav-item {
    flex-direction: row;
    gap: 0;
    justify-content: center;
    padding: 12px 8px;
    min-width: 48px;
  }
  
  .nav-icon {
    font-size: 18px;
  }
}
</style>
