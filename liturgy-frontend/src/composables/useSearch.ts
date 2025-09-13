import { ref, computed } from 'vue'
import { api, type SearchResult } from '../services/api'
import { useCalendarSelection } from './useCalendarSelection'

export interface ExtendedSearchResult extends SearchResult {
  calendarName: string
}

export interface SearchState {
  query: string
  results: ExtendedSearchResult[]
  loading: boolean
  error: string | null
  hasSearched: boolean
}

const state = ref<SearchState>({
  query: '',
  results: [],
  loading: false,
  error: null,
  hasSearched: false,
})

let abortController: AbortController | null = null

export function useSearch() {
  const { selectedCalendars } = useCalendarSelection()

  const searchQuery = computed({
    get: () => state.value.query,
    set: (value: string) => {
      state.value.query = value
      if (!value.trim()) {
        clearResults()
      }
    },
  })

  const searchResults = computed(() => state.value.results)
  const isLoading = computed(() => state.value.loading)
  const error = computed(() => state.value.error)
  const hasSearched = computed(() => state.value.hasSearched)
  const hasResults = computed(() => state.value.results.length > 0)

  async function performSearch() {
    const query = state.value.query.trim()
    if (!query) return

    // Check if any calendars are selected
    if (selectedCalendars.value.length === 0) {
      state.value.error = 'Please select at least one calendar to search'
      return
    }

    // Cancel any ongoing search
    if (abortController) {
      abortController.abort()
    }

    abortController = new AbortController()
    state.value.loading = true
    state.value.error = null
    state.value.hasSearched = true

    try {
      console.log(`Searching for "${query}" in calendars:`, selectedCalendars.value)

      // Search across all selected calendars
      const searchPromises = selectedCalendars.value.map(async (calendarName) => {
        try {
          console.log(`Searching in ${calendarName}...`)
          const results = await api.searchFeasts(calendarName, query)
          console.log(`Found ${results.length} results in ${calendarName}:`, results)
          return results.map((result) => ({
            ...result,
            calendarName,
          }))
        } catch (error) {
          console.warn(`Search failed for calendar ${calendarName}:`, error)
          return []
        }
      })

      const allResults = await Promise.all(searchPromises)

      // Check if request was aborted
      if (abortController.signal.aborted) return

      // Flatten and deduplicate results
      const flatResults = allResults.flat()
      console.log(`Total flat results:`, flatResults)
      const uniqueResults = deduplicateResults(flatResults)
      console.log(`Final unique results:`, uniqueResults)

      state.value.results = uniqueResults
      state.value.loading = false
    } catch (error) {
      if (abortController.signal.aborted) return

      state.value.error = error instanceof Error ? error.message : 'Search failed'
      state.value.loading = false
      state.value.results = []
    }
  }

  function clearResults() {
    if (abortController) {
      abortController.abort()
    }

    state.value.results = []
    state.value.error = null
    state.value.hasSearched = false
  }

  function clearSearch() {
    state.value.query = ''
    clearResults()
  }

  // Sort results by score (higher is better) within each calendar, keeping all results
  function deduplicateResults(results: ExtendedSearchResult[]): ExtendedSearchResult[] {
    // Don't deduplicate across calendars - show all results grouped by calendar
    // Just sort by score within each calendar
    return results.sort((a, b) => {
      // First sort by calendar name
      const calendarDiff = a.calendarName.localeCompare(b.calendarName)
      if (calendarDiff !== 0) return calendarDiff

      // Then by score (higher scores first)
      const scoreDiff = (b.score || 0) - (a.score || 0)
      if (scoreDiff !== 0) return scoreDiff

      // Finally by name alphabetically
      return a.name.localeCompare(b.name)
    })
  }

  return {
    searchQuery,
    searchResults,
    isLoading,
    error,
    hasSearched,
    hasResults,
    performSearch,
    clearSearch,
    clearResults,
  }
}
