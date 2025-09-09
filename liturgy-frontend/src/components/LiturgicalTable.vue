<script setup lang="ts">
import type { CalendarInfo, DayInfo } from '../services/api'

interface DateInfo {
  dateString: string
  displayDate: string
  isSelected?: boolean
}

interface Props {
  dates: DateInfo[]
  calendars: CalendarInfo[]
  dataMap: Record<string, Record<string, DayInfo>>
  loading?: boolean
  showDateColumn?: boolean
  dateColumnTitle?: string
}

const props = withDefaults(defineProps<Props>(), {
  loading: false,
  showDateColumn: true,
  dateColumnTitle: 'Date'
})

// Get commemoration interpretation for a calendar
function getCommemorationInterpretation(calendarName: string): string {
  const calendar = props.calendars.find(cal => cal.name === calendarName)
  return calendar?.commemoration_interpretation || 'Also:'
}
</script>

<template>
  <div class="liturgical-table-container">
    <table class="liturgical-table">
      <thead>
        <tr>
          <th v-if="showDateColumn" class="day-column">{{ dateColumnTitle }}</th>
          <th 
            v-for="calendar in calendars" 
            :key="calendar.name"
            class="calendar-column"
          >
            {{ calendar.display_name }}
          </th>
        </tr>
      </thead>
      <tbody>
        <tr 
          v-for="dateInfo in dates" 
          :key="dateInfo.dateString"
          class="liturgical-row"
          :class="{ 'selected-row': dateInfo.isSelected }"
        >
          <td v-if="showDateColumn" class="day-cell">
            <div class="day-info">
              <div class="day-name">{{ dateInfo.displayDate.split(',')[0] }}</div>
              <div v-if="dateInfo.displayDate.includes(',')" class="day-date">{{ dateInfo.displayDate.split(',')[1]?.trim() }}</div>
            </div>
          </td>
          <td 
            v-for="calendar in calendars" 
            :key="`${dateInfo.dateString}-${calendar.name}`"
            class="liturgy-cell"
          >
            <div v-if="dataMap[dateInfo.dateString]?.[calendar.name]" class="liturgy-content">
              <div class="liturgy-header">
                <span class="color-bar" :style="{ backgroundColor: dataMap[dateInfo.dateString][calendar.name].desc.day.color }"></span>
                <span class="feast-text">{{ dataMap[dateInfo.dateString][calendar.name].desc.day.desc }}</span>
              </div>
              <div class="rank-text">{{ dataMap[dateInfo.dateString][calendar.name].desc.day.rank }}</div>
              <div class="season-text">{{ dataMap[dateInfo.dateString][calendar.name].desc.day_in_season }}</div>
              
              <!-- Commemorations -->
              <div v-if="dataMap[dateInfo.dateString][calendar.name].desc.commemorations && dataMap[dateInfo.dateString][calendar.name].desc.commemorations.length > 0" class="commemorations">
                <div class="commemorations-header">{{ getCommemorationInterpretation(calendar.name) }}</div>
                <div 
                  v-for="commemoration in dataMap[dateInfo.dateString][calendar.name].desc.commemorations" 
                  :key="commemoration.desc"
                  class="commemoration-item"
                >
                  <span class="color-bar small" :style="{ backgroundColor: commemoration.color }"></span>
                  <span class="commemoration-desc">{{ commemoration.desc }}</span>
                  <span class="commemoration-rank">{{ commemoration.rank }}</span>
                </div>
              </div>
            </div>
            <div v-else class="no-data-cell">
              ❌ No data
            </div>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<style scoped>
.liturgical-table-container {
  overflow-x: auto;
  border-radius: 4px;
  border: 1px solid #333;
  background: #000;
  margin-bottom: 20px;
}

.liturgical-table {
  width: 100%;
  border-collapse: collapse;
  min-width: 600px;
}

.liturgical-table th {
  background: #222;
  padding: 12px 8px;
  text-align: left;
  font-weight: 600;
  color: #fff;
  border-bottom: 2px solid #444;
  border-right: 1px solid #444;
}

.liturgical-table th:last-child {
  border-right: none;
}

.day-column {
  width: 120px;
  min-width: 120px;
}

.calendar-column {
  min-width: 200px;
}

.liturgical-row {
  border-bottom: 1px solid #333;
}

.liturgical-row:hover {
  background: #222;
}

.liturgical-row.selected-row {
  background: #333;
}

.liturgical-row.selected-row:hover {
  background: #444;
}

.day-cell {
  padding: 12px 8px;
  border-right: 1px solid #333;
  background: #111;
  vertical-align: top;
}

.day-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.day-name {
  font-weight: 600;
  color: #fff;
  font-size: 14px;
}

.day-date {
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: 2px;
}

.liturgy-cell {
  padding: 12px 8px;
  border-right: 1px solid #333;
  vertical-align: top;
}

.liturgy-cell:last-child {
  border-right: none;
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

.color-bar {
  width: 6px;
  height: 20px;
  border-radius: 3px;
  flex-shrink: 0;
  border: 1px solid #444;
}

.color-bar.small {
  width: 4px;
  height: 14px;
  border-radius: 2px;
}

.feast-text {
  color: #fff;
  font-size: 14px;
  line-height: 1.3;
  font-weight: 600;
}

.rank-text {
  color: var(--text-secondary);
  font-size: 11px;
  font-weight: 500;
  margin-top: 2px;
}

.season-text {
  color: var(--text-secondary);
  font-size: 11px;
  font-style: italic;
}

.commemorations {
  margin-top: 8px;
  padding-top: 6px;
  border-top: 1px solid #333;
}

.commemorations-header {
  font-size: 10px;
  color: var(--text-secondary);
  font-weight: 600;
  margin-bottom: 4px;
  text-transform: uppercase;
}

.commemoration-item {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 3px;
  font-size: 11px;
}

.commemoration-desc {
  color: #ccc;
  font-size: 10px;
  line-height: 1.2;
  font-weight: 500;
}

.commemoration-rank {
  font-weight: normal;
  color: var(--text-secondary);
  font-size: 9px;
}

.no-data-cell {
  font-size: 11px;
  color: var(--text-secondary);
  text-align: center;
  padding: 8px 4px;
}

@media (max-width: 1024px) {
  .liturgical-table-container {
    overflow-x: auto;
    -webkit-overflow-scrolling: touch;
  }
  
  .liturgical-table {
    min-width: 800px;
  }
}

@media (max-width: 768px) {
  .liturgical-table-container {
    border-radius: 6px;
    margin-bottom: 16px;
  }
  
  .liturgical-table {
    min-width: 700px;
  }
  
  .day-column {
    width: 90px;
    min-width: 90px;
  }
  
  .calendar-column {
    min-width: 180px;
  }
  
  .liturgical-table th,
  .day-cell,
  .liturgy-cell {
    padding: 10px 6px;
  }
  
  .feast-text {
    font-size: 13px;
  }
  
  .rank-text {
    font-size: 10px;
  }
  
  .season-text {
    font-size: 10px;
  }
  
  .day-name {
    font-size: 13px;
  }
  
  .day-date {
    font-size: 11px;
  }
}

@media (max-width: 480px) {
  .liturgical-table-container {
    margin: 0 -12px 16px -12px;
    border-radius: 0;
    border-left: none;
    border-right: none;
  }
  
  .liturgical-table {
    min-width: 600px;
  }
  
  .day-column {
    width: 80px;
    min-width: 80px;
  }
  
  .calendar-column {
    min-width: 160px;
  }
  
  .liturgical-table th,
  .day-cell,
  .liturgy-cell {
    padding: 8px 4px;
  }
  
  .liturgical-table th {
    font-size: 12px;
  }
  
  .feast-text {
    font-size: 12px;
  }
  
  .rank-text {
    font-size: 9px;
  }
  
  .season-text {
    font-size: 9px;
  }
  
  .day-name {
    font-size: 12px;
  }
  
  .day-date {
    font-size: 10px;
  }
  
  .commemorations-header {
    font-size: 9px;
  }
  
  .commemoration-desc {
    font-size: 9px;
  }
  
  .commemoration-rank {
    font-size: 8px;
  }
}
</style>
