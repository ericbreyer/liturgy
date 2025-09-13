<script setup lang="ts">
import { ref } from 'vue'
import { useCalendarSelection } from '../composables/useCalendarSelection'

const showCalendarDropdown = ref<boolean>(false)

const {
  calendars,
  selectedCalendars,
  loading,
  error,
  selectedCalendarInfos,
  toggleCalendar,
  selectAll,
  selectNone
} = useCalendarSelection()

// Props for customization
interface Props {
  title?: string
  showTitle?: boolean
  variant?: 'dropdown' | 'expanded'
}

const props = withDefaults(defineProps<Props>(), {
  title: 'Select Calendars:',
  showTitle: true,
  variant: 'dropdown'
})

function handleCalendarToggle(calendarName: string) {
  toggleCalendar(calendarName)
}

function handleSelectAll() {
  selectAll()
}

function handleSelectNone() {
  selectNone()
}
</script>

<template>
  <div class="calendar-selection">
    <h3 v-if="showTitle">{{ title }}</h3>
    
    <div v-if="error" class="error">
      ❌ {{ error }}
    </div>

    <!-- Dropdown variant -->
    <div v-if="variant === 'dropdown'" class="calendar-dropdown">
      <button @click="showCalendarDropdown = !showCalendarDropdown" class="dropdown-toggle" :disabled="loading">
        <span>{{ selectedCalendars.length }} of {{ calendars.length }} calendars selected</span>
        <span class="dropdown-arrow" :class="{ 'open': showCalendarDropdown }">▼</span>
      </button>
      <div v-if="showCalendarDropdown" class="dropdown-content">
        <div class="selection-buttons">
          <button @click="handleSelectAll" class="select-btn">Select All</button>
          <button @click="handleSelectNone" class="select-btn">Select None</button>
        </div>
        <div class="calendar-checkboxes">
          <label 
            v-for="calendar in calendars" 
            :key="calendar.name" 
            class="checkbox-label"
          >
            <input
              type="checkbox"
              :value="calendar.name"
              :checked="selectedCalendars.includes(calendar.name)"
              @change="handleCalendarToggle(calendar.name)"
              :disabled="loading"
            >
            <span class="checkbox-text">{{ calendar.display_name }}</span>
          </label>
        </div>
      </div>
    </div>

    <!-- Expanded variant -->
    <div v-else-if="variant === 'expanded'" class="calendar-expanded">
      <div class="selection-buttons">
        <button @click="handleSelectAll" class="select-btn">Select All</button>
        <button @click="handleSelectNone" class="select-btn">Select None</button>
      </div>
      <div class="calendar-checkboxes">
        <label 
          v-for="calendar in calendars" 
          :key="calendar.name" 
          class="checkbox-label"
        >
          <input
            type="checkbox"
            :value="calendar.name"
            :checked="selectedCalendars.includes(calendar.name)"
            @change="handleCalendarToggle(calendar.name)"
            :disabled="loading"
          >
          <span class="checkbox-text">{{ calendar.display_name }}</span>
        </label>
      </div>
    </div>

    <div v-if="loading" class="loading-calendars">
      ⏳ Loading calendars...
    </div>
  </div>
</template>

<style scoped>
.calendar-selection h3 {
  margin-top: 0;
  margin-bottom: 15px;
  color: #333;
}

.error {
  background: #3a1a1a;
  color: #ff6b6b;
  padding: 10px;
  border-radius: 6px;
  border: 1px solid #4a2828;
  margin-bottom: 15px;
  font-size: 14px;
}

.calendar-dropdown {
  position: relative;
}

.dropdown-toggle {
  width: 100%;
  padding: 12px 16px;
  background: #222;
  border: 1px solid #444;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  color: #fff;
}

.dropdown-toggle:hover:not(:disabled) {
  background: #333;
}

.dropdown-toggle:disabled {
  background: #111;
  cursor: not-allowed;
  color: var(--text-secondary);
}

.dropdown-arrow {
  transition: transform 0.2s ease;
  color: var(--text-secondary);
}

.dropdown-arrow.open {
  transform: rotate(180deg);
}

.dropdown-content {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  background: #222;
  border: 1px solid #444;
  border-top: none;
  border-radius: 0 0 4px 4px;
  z-index: 1000;
  padding: 12px;
}

.dropdown-content .selection-buttons {
  margin-bottom: 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid #444;
}

.calendar-expanded {
  background: #111;
  border: 1px solid #333;
  border-radius: 6px;
  padding: 16px;
}

.selection-buttons {
  display: flex;
  gap: 10px;
  margin-bottom: 15px;
}

.select-btn {
  padding: 8px 12px;
  background: #6c757d;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
}

.select-btn:hover {
  background: #5a6268;
}

.calendar-checkboxes {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  padding: 8px;
  border-radius: 4px;
  transition: background-color 0.2s;
}

.checkbox-label:hover {
  background: #222;
}

.checkbox-text {
  font-size: 14px;
  color: var(--text-primary);
}

.loading-calendars {
  text-align: center;
  padding: 20px;
  color: var(--text-secondary);
  font-size: 14px;
}

@media (max-width: 768px) {
  .calendar-selection {
    margin-bottom: 16px;
  }
  
  .dropdown-toggle {
    padding: 12px 16px;
    font-size: 14px;
  }
  
  .dropdown-content {
    padding: 12px;
  }
  
  .calendar-checkboxes {
    flex-direction: column;
    max-height: 300px;
    overflow-y: auto;
  }
  
  .checkbox-label {
    flex: 1;
    padding: 10px;
    margin: 2px 0;
  }
  
  .checkbox-text {
    font-size: 14px;
  }
  
  .selection-buttons {
    flex-direction: row;
    gap: 8px;
  }
  
  .select-btn {
    flex: 1;
    padding: 10px 12px;
    font-size: 13px;
  }
}

@media (max-width: 480px) {
  .dropdown-toggle {
    padding: 14px 16px;
    font-size: 16px;
    width: 100%;
  }
  
  .dropdown-content {
    padding: 16px;
    margin: 0 -12px;
    border-radius: 0;
  }
  
  .selection-buttons {
    flex-direction: column;
    gap: 8px;
  }
  
  .select-btn {
    width: 100%;
    padding: 12px;
    font-size: 14px;
  }
  
  .checkbox-label {
    padding: 12px;
    border: 1px solid #333;
    border-radius: 6px;
    margin: 4px 0;
  }
  
  .checkbox-text {
    font-size: 15px;
  }
  
  h3 {
    font-size: 16px;
    margin-bottom: 12px;
  }
}
</style>

<script lang="ts">
// Provide a runtime default export for environments/tools that expect one
// (keeps compatibility with older import behaviors / typecheckers)
export default {}
</script>
