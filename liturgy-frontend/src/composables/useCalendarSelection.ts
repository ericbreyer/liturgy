import { ref, computed, watch } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { api, type CalendarInfo } from '../services/api'

// Shared state
const calendars = ref<CalendarInfo[]>([])
const selectedCalendars = ref<string[]>([])
const loading = ref(false)
const error = ref<string>('')

// Computed properties
const selectedCalendarInfos = computed(() => {
  return calendars.value.filter((cal) => selectedCalendars.value.includes(cal.name))
})

// Helper functions for URL persistence
function getCalendarsFromUrl(route: any): string[] {
  const calendarsParam = route.query.calendars
  if (typeof calendarsParam === 'string') {
    return calendarsParam.split(',').filter((name) => name.length > 0)
  }
  return []
}

function updateUrlWithCalendars(router: any, route: any, calendarsToSelect: string[]) {
  const newQuery = { ...route.query }
  if (calendarsToSelect.length > 0) {
    newQuery.calendars = calendarsToSelect.join(',')
  } else {
    delete newQuery.calendars
  }

  // Only update if the query actually changed
  const currentCalendarsParam = route.query.calendars || ''
  const newCalendarsParam = newQuery.calendars || ''
  if (currentCalendarsParam !== newCalendarsParam) {
    router.replace({ query: newQuery })
  }
}

// Actions
async function loadCalendars(router?: any, route?: any) {
  try {
    loading.value = true
    error.value = ''
    calendars.value = await api.getCalendars()

    if (router && route) {
      // Get calendars from URL query parameters
      const urlCalendars = getCalendarsFromUrl(route)

      if (urlCalendars.length > 0) {
        // Use calendars from URL, but only include valid ones
        const validCalendars = urlCalendars.filter((name) =>
          calendars.value.some((cal) => cal.name === name),
        )
        selectedCalendars.value = validCalendars

        // If URL had invalid calendars, clean up the URL
        if (validCalendars.length !== urlCalendars.length) {
          updateUrlWithCalendars(router, route, validCalendars)
        }
      } else if (calendars.value.length > 0 && selectedCalendars.value.length === 0) {
        // No URL calendars, use default selection
        const defaultCalendars = calendars.value
          .map((cal) => cal.name)
          .filter((name) => name == 'of-us' || name == 'ef')
        selectedCalendars.value = defaultCalendars
        updateUrlWithCalendars(router, route, defaultCalendars)
      }

      // Watch for changes to selectedCalendars and update URL
      watch(
        selectedCalendars,
        (newCalendars) => {
          updateUrlWithCalendars(router, route, newCalendars)
        },
        { deep: true },
      )
    } else {
      // Fallback when no router/route provided
      if (calendars.value.length > 0 && selectedCalendars.value.length === 0) {
        selectedCalendars.value = calendars.value
          .map((cal) => cal.name)
          .filter((name) => name == 'of-us' || name == 'ef')
      }
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to load calendars'
  } finally {
    loading.value = false
  }
}

function toggleCalendar(calendarName: string) {
  if (selectedCalendars.value.includes(calendarName)) {
    selectedCalendars.value = selectedCalendars.value.filter((name) => name !== calendarName)
  } else {
    selectedCalendars.value = [...selectedCalendars.value, calendarName]
  }
}

function selectAll() {
  selectedCalendars.value = [...calendars.value.map((cal) => cal.name)]
}

function selectNone() {
  selectedCalendars.value = []
}

function syncWithRoute(route?: any) {
  if (!route) return

  const urlCalendars = getCalendarsFromUrl(route)

  if (urlCalendars.length > 0) {
    // Filter to only valid calendars
    const validCalendars = urlCalendars.filter((name) =>
      calendars.value.some((cal) => cal.name === name),
    )
    selectedCalendars.value = validCalendars
  }
}

export function useCalendarSelection() {
  return {
    // State
    calendars,
    selectedCalendars,
    loading,
    error,

    // Computed
    selectedCalendarInfos,

    // Actions
    loadCalendars,
    toggleCalendar,
    selectAll,
    selectNone,
    syncWithRoute,
  }
}
