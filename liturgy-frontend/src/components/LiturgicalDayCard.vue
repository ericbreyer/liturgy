<script setup lang="ts">
import type { DayInfo } from '../services/api'

interface Props {
  dateString: string
  displayDate: string
  dayData: Record<string, DayInfo>
  calendars: Array<{ name: string; display_name: string; commemoration_interpretation?: string }>
  isSelected?: boolean
  showDate?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  isSelected: false,
  showDate: true,
})

// Get commemoration interpretation for a calendar
function getCommemorationInterpretation(calendarName: string): string {
  const calendar = props.calendars.find((cal) => cal.name === calendarName)
  return calendar?.commemoration_interpretation || 'Also:'
}
</script>

<template>
  <div class="liturgical-day-card" :class="{ selected: isSelected }">
    <div v-if="showDate" class="day-header">
      <h3 class="day-name">{{ displayDate.split(',')[0] }}</h3>
      <p v-if="displayDate.includes(',')" class="day-date">
        {{ displayDate.split(',')[1]?.trim() }}
      </p>
    </div>

    <div class="calendars-container">
      <div v-for="calendar in calendars" :key="calendar.name" class="calendar-section">
        <h4 class="calendar-title">{{ calendar.display_name }}</h4>

        <div v-if="dayData[calendar.name]" class="liturgy-content">
          <div class="liturgy-header">
            <span
              class="color-bar"
              :style="{ backgroundColor: dayData[calendar.name].desc.day.color }"
            ></span>
            <span class="feast-text">{{ dayData[calendar.name].desc.day.desc }}</span>
          </div>
          <div class="rank-text">{{ dayData[calendar.name].desc.day.rank }}</div>
          <div class="season-text">{{ dayData[calendar.name].desc.day_in_season }}</div>

          <!-- Commemorations -->
          <div
            v-if="
              dayData[calendar.name].desc.commemorations &&
              dayData[calendar.name].desc.commemorations.length > 0
            "
            class="commemorations"
          >
            <div class="commemorations-header">
              {{ getCommemorationInterpretation(calendar.name) }}
            </div>
            <div
              v-for="commemoration in dayData[calendar.name].desc.commemorations"
              :key="commemoration.desc"
              class="commemoration-item"
            >
              <span
                class="color-bar small"
                :style="{ backgroundColor: commemoration.color }"
              ></span>
              <span class="commemoration-desc">{{ commemoration.desc }}</span>
              <span class="commemoration-rank">{{ commemoration.rank }}</span>
            </div>
          </div>
        </div>

        <div v-else class="no-data">
          <span class="no-data-text">❌ No data</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
@import '../styles/liturgical.css';

/* Component-specific overrides only. Shared selectors (feast-title, liturgical-color-bar, commemoration-count, day-number) live in liturgical.css */
.liturgical-day-card {
  background: #111;
  border-radius: 4px;
  border: 1px solid #333;
  padding: 16px;
  margin-bottom: 16px;
  transition: all 0.2s ease;
}

.liturgical-day-card.selected {
  border-color: var(--accent-color);
}

.liturgical-day-card:hover {
  border-color: #555;
}

.day-header {
  margin-bottom: 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid #333;
}

.day-name {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0 0 4px 0;
}

.day-date {
  font-size: 14px;
  color: var(--text-secondary);
  margin: 0;
}

.calendars-container {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.calendar-section {
  padding: 12px;
  background: #222;
  border-radius: 8px;
  border: 1px solid #444;
}

.calendar-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-secondary);
  margin: 0 0 8px 0;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.liturgy-content {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.liturgy-header {
  display: flex;
  align-items: center;
  gap: 8px;
}

/* color-bar size handled by shared .liturgical-color-bar rules; keep small-size tweak here */
.color-bar.small {
  width: 4px;
  height: 14px;
  border-radius: 2px;
}

.feast-text {
  font-weight: 600;
}

.rank-text {
  color: var(--text-muted);
  font-size: 12px;
  font-weight: 500;
  margin-top: 2px;
}

.season-text {
  color: var(--text-secondary);
  font-size: 12px;
  font-style: italic;
}

.commemorations {
  margin-top: 8px;
  padding-top: 6px;
  border-top: 1px solid var(--border-primary);
}

.commemorations-header {
  font-size: 11px;
  color: var(--text-muted);
  font-weight: 600;
  margin-bottom: 4px;
  text-transform: uppercase;
}

.commemoration-item {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 3px;
  font-size: 12px;
}

.commemoration-desc {
  color: var(--text-secondary);
  font-size: 11px;
  line-height: 1.2;
  font-weight: 500;
}

.commemoration-rank {
  font-weight: normal;
  color: var(--text-secondary);
  font-size: 10px;
}

.no-data {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 12px;
}

.no-data-text {
  font-size: 12px;
  color: var(--text-secondary);
}

@media (min-width: 768px) {
  .calendars-container {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 16px;
  }
}

@media (min-width: 1024px) {
  .liturgical-day-card {
    padding: 20px;
  }

  .day-name {
    font-size: 20px;
  }

  .feast-text {
    font-size: 16px;
  }
}
</style>
