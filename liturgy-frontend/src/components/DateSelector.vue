<script setup lang="ts">
import { computed, ref } from 'vue'

interface Props {
  modelValue: string
  loading?: boolean
  variant?: 'day' | 'week' | 'month'
  showTitle?: boolean
  compact?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  loading: false,
  variant: 'day',
  showTitle: false,
  compact: false
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
  'go-previous': []
  'go-next': []
  'go-today': []
}>()

// Ref for the hidden date input
const dateInput = ref<HTMLInputElement>()

// Computed for formatted date display
const formattedDate = computed(() => {
  const [year, month, day] = props.modelValue.split('-').map(Number)
  const date = new Date(year, month - 1, day)
  
  if (props.compact) {
    return date.toLocaleDateString('en-US', { 
      month: 'short', 
      day: 'numeric' 
    })
  }
  
  if (props.variant === 'month') {
    return date.toLocaleDateString('en-US', { 
      year: 'numeric', 
      month: 'long' 
    })
  }
  
  return date.toLocaleDateString('en-US', { 
    weekday: 'short', 
    month: 'short', 
    day: 'numeric' 
  })
})

// For month view, use month input type
const inputType = computed(() => {
  return props.variant === 'month' ? 'month' : 'date'
})

// Convert date to month format for month input
const monthValue = computed(() => {
  if (props.variant === 'month') {
    const [year, month] = props.modelValue.split('-')
    return `${year}-${month}`
  }
  return props.modelValue
})

// Navigation button labels based on variant
const previousLabel = computed(() => {
  if (props.compact) {
    return '‹'
  }
  switch (props.variant) {
    case 'week': return '← Prev Week'
    case 'month': return '← Prev Month'
    default: return '← Prev Day'
  }
})

const nextLabel = computed(() => {
  if (props.compact) {
    return '›'
  }
  switch (props.variant) {
    case 'week': return 'Next Week →'
    case 'month': return 'Next Month →'
    default: return 'Next Day →'
  }
})

function updateDate(event: Event) {
  const target = event.target as HTMLInputElement
  let newDate = target.value
  
  // If month input, convert to full date (use 1st of month)
  if (props.variant === 'month' && newDate.match(/^\d{4}-\d{2}$/)) {
    newDate = `${newDate}-01`
  }
  
  emit('update:modelValue', newDate)
}

function goToPrevious() {
  emit('go-previous')
}

function goToNext() {
  emit('go-next')
}

function goToToday() {
  emit('go-today')
}
</script>

<template>
  <div class="date-selector" :class="{ 'compact': compact }">
    <h3 v-if="showTitle" class="title">
      {{ variant === 'week' ? 'Select Week Center:' : 'Select Date:' }}
    </h3>
    
    <div class="date-controls" :class="{ 'compact': compact }">
      <!-- Previous Button -->
      <button 
        @click="goToPrevious" 
        :disabled="loading" 
        class="nav-btn previous"
        :class="{ 'compact': compact }"
        :title="variant === 'week' ? 'Previous Week' : variant === 'month' ? 'Previous Month' : 'Previous Day'"
      >
        {{ previousLabel }}
      </button>

      <!-- Date Input and Display -->
      <div class="date-input-container" :class="{ 'compact': compact }">
        <input
          ref="dateInput"
          :type="inputType"
          :value="monthValue"
          @change="updateDate"
          :disabled="loading"
          class="date-picker"
          :class="{ 'compact': compact }"
        >
      </div>

      <!-- Next Button -->
      <button 
        @click="goToNext" 
        :disabled="loading" 
        class="nav-btn next"
        :class="{ 'compact': compact }"
        :title="variant === 'week' ? 'Next Week' : variant === 'month' ? 'Next Month' : 'Next Day'"
      >
        {{ nextLabel }}
      </button>

      <!-- Today Button -->
      <button 
        @click="goToToday" 
        class="today-btn"
        :class="{ 'compact': compact }"
        :title="'Go to Today'"
      >
        {{ compact ? '●' : 'Today' }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.date-selector {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.date-selector.compact {
  gap: 8px;
}

.title {
  margin: 0;
  color: var(--text-primary);
  font-size: 16px;
  font-weight: 600;
}

.date-controls {
  display: flex;
  gap: 8px;
  align-items: center;
  flex-wrap: wrap;
}

.date-controls.compact {
  gap: 8px;
  flex-wrap: wrap;
}

@media (max-width: 768px) {
  .date-controls {
    gap: 0.5rem;
    padding: 12px;
    background: #222;
    border-radius: 8px;
  }
  
  .nav-btn {
    padding: 10px 14px;
    font-size: 0.875rem;
    min-width: 90px;
    flex: 1;
  }
  
  .date-picker {
    padding: 10px 12px;
    font-size: 0.875rem;
    min-width: 140px;
  }
  
  .today-btn {
    padding: 10px 14px;
    font-size: 0.875rem;
    flex: 1;
  }
  
  .date-input-container {
    flex: 2;
    min-width: 140px;
  }
}

@media (max-width: 480px) {
  .date-controls {
    flex-direction: column;
    align-items: stretch;
    gap: 8px;
    padding: 16px;
  }
  
  .nav-btn {
    width: 100%;
    margin: 0;
    padding: 12px;
    font-size: 16px;
  }
  
  .date-input-container {
    order: -1;
    margin-bottom: 8px;
    width: 100%;
  }
  
  .date-picker {
    width: 100%;
    padding: 12px;
    font-size: 16px;
  }
  
  .today-btn {
    width: 100%;
    padding: 12px;
    font-size: 16px;
  }
  
  h3 {
    font-size: 16px;
    margin-bottom: 12px;
  }
}

.nav-btn {
  padding: 10px 12px;
  background: #6c757d;
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  white-space: nowrap;
  transition: background-color 0.2s;
}

.nav-btn.compact {
  padding: 8px 10px;
  font-size: 16px;
  min-width: 36px;
  border-radius: 50%;
  font-weight: bold;
}

.nav-btn:hover:not(:disabled) {
  background: #5a6268;
}

.nav-btn:disabled {
  background: #333;
  cursor: not-allowed;
}

.date-input-container {
  display: flex;
  align-items: center;
  position: relative;
  flex: 1;
  min-width: 150px;
}

.date-input-container.compact {
  min-width: 100px;
  flex: 0 1 auto;
}

.date-picker {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid #444;
  border-radius: 6px;
  font-size: 14px;
  background: #222;
  color: var(--text-primary);
}

.date-picker.compact {
  padding: 8px 12px;
  font-size: 14px;
  border: 1px solid #444;
  border-radius: 6px;
  background: #222;
  color: var(--text-primary);
  text-align: center;
  min-width: 120px;
}

.date-picker.compact:focus {
  outline: none;
  border-color: var(--accent-color);
}

.date-picker:focus {
  outline: none;
  border-color: var(--accent-color);
}

.today-btn {
  padding: 10px 16px;
  background: var(--surface-primary);
  color: var(--text-primary);
  border: 2px solid var(--accent-color);
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  white-space: nowrap;
  transition: all 0.2s ease;
}

.today-btn:hover {
  background: var(--surface-secondary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
}

.today-btn.compact {
  padding: 8px 10px;
  min-width: 36px;
  border-radius: 4px;
  font-size: 16px;
  font-weight: 400;
}

.today-btn:hover {
  background: rgb(120, 70, 220);
}

/* Mobile optimizations */
@media (max-width: 768px) {
  .date-controls:not(.compact) {
    flex-direction: column;
    align-items: stretch;
    gap: 8px;
  }
  
  .date-controls:not(.compact) .nav-btn,
  .date-controls:not(.compact) .today-btn {
    width: 100%;
    justify-content: center;
  }
  
  .date-input-container:not(.compact) {
    min-width: unset;
  }
  
  .date-controls.compact {
    flex-wrap: nowrap;
    justify-content: space-between;
    max-width: 280px;
    margin: 0 auto;
  }
}

@media (max-width: 480px) {
  .date-controls.compact {
    max-width: 100%;
    padding: 0 10px;
  }
  
  .nav-btn.compact {
    min-width: 44px;
    padding: 10px;
    font-size: 16px;
  }
  
  .today-btn.compact {
    min-width: 44px;
    padding: 10px;
    font-size: 16px;
  }
  
  .date-picker.compact {
    font-size: 16px;
    padding: 10px 12px;
    min-width: 140px;
  }
}
</style>

<script lang="ts">
// Provide a runtime default export for environments/tools that expect one
// (keeps compatibility with older import behaviors / typecheckers)
export default {}
</script>
