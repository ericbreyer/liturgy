<script setup lang="ts">
import { computed } from 'vue'
import { useSearch } from '../composables/useSearch'
import { useCalendarSelection } from '../composables/useCalendarSelection'
import { getColorValue } from '../utils/liturgical'

const { selectedCalendars } = useCalendarSelection()
const {
  searchQuery,
  searchResults,
  isLoading,
  error,
  hasSearched,
  hasResults,
  performSearch,
  clearSearch,
} = useSearch()

// Group results by calendar
const getResultsForCalendar = (calendarName: string) => {
  return searchResults.value.filter((result) => result.calendarName === calendarName)
}
</script>

<template>
  <div class="search-view">
    <div class="search-container">
      <div class="search-box">
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Search for feasts, saints, seasons..."
          class="search-input"
          @keyup.enter="performSearch"
        />
        <button
          @click="performSearch"
          class="search-btn"
          :disabled="!searchQuery.trim() || isLoading"
        >
          🔍 Search
        </button>
        <button @click="clearSearch" class="clear-btn" v-if="searchQuery">✖️ Clear</button>
      </div>

      <div v-if="isLoading" class="loading">⏳ Searching...</div>

      <div v-if="error" class="error">❌ {{ error }}</div>

      <div v-if="!hasResults && hasSearched && !isLoading && !error" class="no-results">
        📭 No results found for "{{ searchQuery }}"
      </div>

      <div v-if="!searchQuery && !isLoading" class="search-help">
        <h3>Search Tips</h3>
        <p v-if="selectedCalendars.length === 0" class="warning">
          ⚠️ Please select at least one calendar to search
        </p>
        <p v-else class="selected-calendars">📚 Searching in: {{ selectedCalendars.join(', ') }}</p>
        <ul>
          <li>🎉 Search for feast names (e.g., "Christmas", "Easter")</li>
          <li>👼 Look up saints (e.g., "Saint Joseph", "Mary")</li>
          <li>📅 Find liturgical seasons (e.g., "Advent", "Lent")</li>
          <li>⛪ Search by liturgical rank or color</li>
        </ul>
      </div>

      <!-- Search Results -->
      <div v-if="hasResults" class="search-results">
        <h3>Search Results ({{ searchResults.length }})</h3>

        <!-- Group results by calendar -->
        <div class="calendar-columns">
          <div
            v-for="calendarName in selectedCalendars"
            :key="calendarName"
            class="calendar-column"
          >
            <h4 class="calendar-header">{{ calendarName.toUpperCase() }}</h4>
            <div class="calendar-results">
              <div
                v-for="result in getResultsForCalendar(calendarName)"
                :key="`${result.calendarName}-${result.name}`"
                class="result-card"
              >
                <div class="result-header">
                  <h5 class="result-title">{{ result.name }}</h5>
                  <span class="result-score">{{ result.score.toFixed(2) }}</span>
                </div>

                <!-- Color bar -->
                <div
                  class="color-bar"
                  :style="{ backgroundColor: getColorValue(result.color) }"
                ></div>

                <p class="result-description">{{ result.description }}</p>
                <div class="result-meta">
                  <span v-if="result.date" class="result-date">📅 {{ result.date }}</span>
                  <span v-if="result.rank" class="result-rank">⭐ {{ result.rank }}</span>
                </div>
              </div>

              <div
                v-if="getResultsForCalendar(calendarName).length === 0"
                class="no-calendar-results"
              >
                No results in {{ calendarName }}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
@import '../styles/liturgical.css';

.search-view {
  /* width: var(--layout-fixed-width); */
  max-width: 100vw; /* Fallback for very small screens */
  margin: 0 auto;
  padding: 0 var(--layout-padding);
  box-sizing: border-box;
}

.search-container {
  background: var(--surface-secondary);
  border-radius: 8px;
  padding: 30px;
  border: 1px solid var(--border-color);
  margin-top: 20px;
}

.search-box {
  display: flex;
  gap: 12px;
  margin-bottom: 30px;
  flex-wrap: wrap;
}

.search-input {
  flex: 1;
  padding: 12px 16px;
  border: 2px solid var(--border-primary);
  border-radius: 8px;
  font-size: 16px;
  min-width: 200px;
}

.search-input:focus {
  outline: none;
  border-color: var(--success-color);
}

.search-btn {
  background: var(--success-color);
  color: var(--text-primary);
  border: none;
  padding: 12px 20px;
  border-radius: 8px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 600;
  white-space: nowrap;
}

.search-btn:hover:not(:disabled) {
  background: var(--success-color);
  opacity: 0.8;
}

.search-btn:disabled {
  background: var(--surface-elevated);
  cursor: not-allowed;
}

.clear-btn {
  background: var(--error-color);
  color: var(--text-primary);
  border: none;
  padding: 12px 20px;
  border-radius: 8px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 600;
  white-space: nowrap;
}

.clear-btn:hover:not(:disabled) {
  background: var(--error-color);
  opacity: 0.8;
}

.loading {
  text-align: center;
  padding: 40px;
  color: var(--text-secondary);
  font-size: 18px;
}

.error {
  text-align: center;
  padding: 20px;
  background: var(--error-bg);
  color: var(--error-text);
  border: 1px solid var(--error-border);
  border-radius: 8px;
  margin-bottom: 20px;
}

.no-results {
  text-align: center;
  padding: 40px;
  color: var(--text-secondary);
  font-size: 16px;
}

.search-help {
  background: var(--surface-secondary);
  padding: 20px;
  border-radius: 8px;
  border: 1px solid var(--border-primary);
}

.search-help h3 {
  color: var(--text-primary);
  margin-top: 0;
  margin-bottom: 16px;
}

.search-help .warning {
  color: var(--error-color);
  font-weight: 600;
  margin-bottom: 16px;
}

.search-help .selected-calendars {
  color: var(--success-color);
  font-weight: 500;
  margin-bottom: 16px;
}

.search-help ul {
  list-style: none;
  padding: 0;
  margin: 0;
}

.search-help li {
  padding: 8px 0;
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  gap: 8px;
}

.search-results {
  margin-top: 30px;
}

.search-results h3 {
  color: var(--text-primary);
  margin-bottom: 20px;
  padding-bottom: 10px;
  border-bottom: 2px solid var(--border-primary);
}

.calendar-columns {
  display: flex;
  gap: 20px;
  align-items: flex-start;
}

.calendar-column {
  flex: 1;
  min-width: 280px;
  background: var(--surface-secondary);
  border-radius: 12px;
  overflow: hidden;
  border: 1px solid var(--border-primary);
}

.calendar-header {
  background: var(--surface-interactive);
  color: var(--text-primary);
  margin: 0;
  padding: 16px 20px;
  font-size: 14px;
  font-weight: 600;
  letter-spacing: 1px;
}

.calendar-results {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.no-calendar-results {
  text-align: center;
  color: var(--text-muted);
  font-style: italic;
  padding: 20px;
  background: var(--surface-primary);
  border-radius: 4px;
  border: 1px dashed var(--border-secondary);
}

.results-grid {
  display: grid;
  gap: 16px;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
}

.result-card {
  background: var(--surface-primary);
  border: 1px solid var(--border-primary);
  border-radius: 4px;
  padding: 16px;
  transition: border-color 0.2s ease;
}

.result-card:hover {
  border-color: var(--accent-color);
}

.result-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 8px;
}

.result-title {
  margin: 0;
  color: var(--text-primary);
  font-size: 16px;
  font-weight: 600;
  flex: 1;
  line-height: 1.3;
}

.result-score {
  background: var(--warning-color);
  color: var(--text-primary);
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 600;
  margin-left: 8px;
  opacity: 0.8;
  font-family: 'Monaco', 'Menlo', monospace;
}

.color-bar {
  height: 4px;
  width: 100%;
  border-radius: 2px;
  margin: 8px 0;
  border: 1px solid rgba(0, 0, 0, 0.1);
}

.result-calendar {
  background: var(--surface-secondary);
  color: var(--info-color);
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 500;
  margin-left: 12px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.result-description {
  color: var(--text-secondary);
  line-height: 1.5;
  margin: 0 0 12px 0;
}

.result-meta {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.result-date,
.result-rank {
  background: var(--surface-elevated);
  color: var(--text-secondary);
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 500;
}

.result-rank {
  background: var(--warning-color);
  color: var(--text-primary);
  opacity: 0.8;
}

@media (max-width: 768px) {
  .search-box {
    flex-direction: column;
  }

  .search-input {
    min-width: 100%;
  }

  .calendar-columns {
    flex-direction: column;
    gap: 16px;
  }

  .calendar-column {
    min-width: 100%;
  }

  .result-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
  }

  .result-score {
    align-self: flex-end;
  }
}
</style>
