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
  compact: false,
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
  'go-previous': []
  'go-next': []
  'go-today': []
}>()

// Ref for the hidden date input
const dateInput = ref<HTMLInputElement | null>(null)

// Open/focus the native date picker when mobile date is tapped
function openDatePicker() {
  const el = dateInput.value
  if (!el) return
  // Prefer the standardized API if available
  // showPicker is supported in newer Chromium-based browsers and reliably opens the picker
  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore
  if (typeof el.showPicker === 'function') {
    try {
      // @ts-ignore
      el.showPicker()
      return
    } catch (e) {
      // fall back to other methods
    }
  }

  // Fallback: toggle classes to reveal the off-screen input until the user blurs it.
  const container = el.closest('.date-input-container') as HTMLElement | null
  const onBlur = () => {
    if (container) container.classList.remove('visible-picker')
    el.classList.remove('visible-picker-input')
    el.removeEventListener('blur', onBlur)
    window.clearTimeout(timeout)
  }

  if (container) container.classList.add('visible-picker')
  el.classList.add('visible-picker-input')
  // focus then click to trigger the native picker on mobile / allow interaction in devtools
  el.focus()
  try {
    el.click()
  } catch (e) {
    // ignore
  }

  el.addEventListener('blur', onBlur)
  // safety timeout to restore in case blur doesn't fire (e.g., devtools quirks)
  const timeout = window.setTimeout(onBlur, 3000)
}

// Computed for formatted date display
const formattedDate = computed(() => {
  const [year, month, day] = props.modelValue.split('-').map(Number)
  const date = new Date(year, month - 1, day)

  if (props.compact) {
    return date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
    })
  }

  if (props.variant === 'month') {
    return date.toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'long',
    })
  }

  return date.toLocaleDateString('en-US', {
    weekday: 'short',
    month: 'short',
    day: 'numeric',
  })
})

// Short date for mobile display (e.g., "Sep 12")
const mobileDate = computed(() => {
  const [year, month, day] = props.modelValue.split('-').map(Number)
  const date = new Date(year, month - 1, day)
  return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' })
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
    case 'week':
      return '← Prev Week'
    case 'month':
      return '← Prev Month'
    default:
      return '← Prev Day'
  }
})

const nextLabel = computed(() => {
  if (props.compact) {
    return '›'
  }
  switch (props.variant) {
    case 'week':
      return 'Next Week →'
    case 'month':
      return 'Next Month →'
    default:
      return 'Next Day →'
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
  <div class="date-selector" :class="{ compact: compact }">
    <h3 v-if="showTitle" class="title">
      {{ variant === 'week' ? 'Select Week Center:' : 'Select Date:' }}
    </h3>

    <div class="date-controls" :class="{ compact: compact }">
      <!-- Previous Button -->
      <button
        @click="goToPrevious"
        :disabled="loading"
        class="nav-btn previous"
        :class="{ compact: compact }"
        :title="
          variant === 'week'
            ? 'Previous Week'
            : variant === 'month'
              ? 'Previous Month'
              : 'Previous Day'
        "
      >
        <span class="glyph">‹</span>
        <span class="label">{{ previousLabel }}</span>
      </button>

      <!-- Date Input and Display (hidden on mobile; mobile date shown separately) -->
      <div class="date-input-container" :class="{ compact: compact }">
        <input
          ref="dateInput"
          :type="inputType"
          :value="monthValue"
          @change="updateDate"
          :disabled="loading"
          class="date-picker"
          :aria-label="variant === 'month' ? 'Select month' : 'Select date'"
          :class="{ compact: compact }"
        />
      </div>

      <!-- Native date picker will appear between prev/next on mobile; clicking it opens native UI -->
      <!-- keep the input in the DOM above; we will center it via CSS on small screens -->

      <!-- Next Button -->
      <button
        @click="goToNext"
        :disabled="loading"
        class="nav-btn next"
        :class="{ compact: compact }"
        :title="variant === 'week' ? 'Next Week' : variant === 'month' ? 'Next Month' : 'Next Day'"
      >
        <span class="glyph">›</span>
        <span class="label">{{ nextLabel }}</span>
      </button>

      <!-- Today Button -->
      <button
        @click="goToToday"
        class="today-btn"
        :class="{ compact: compact }"
        :title="'Go to Today'"
      >
        <span class="glyph">●</span>
        <span class="label">{{ compact ? '' : 'Today' }}</span>
      </button>
    </div>
  </div>
</template>

<style scoped>

.date-selector {
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-width: 0;
  max-width: 100%;
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
  min-width: 0;
  max-width: 100%;
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

.nav-btn .glyph {
  display: none; /* hide glyphs on desktop by default */
  font-size: 18px;
  line-height: 1;
}

/* Ensure today's dot glyph is hidden on desktop too */
.today-btn .glyph {
  display: none;
}

/* Labels should be visible on desktop */
.nav-btn .label,
.today-btn .label {
  display: inline-block;
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
  flex: 1 1 180px;
  min-width: 0;
  max-width: 100%;
}

.date-input-container.compact {
  flex: 1 1 160px;
}

.date-picker {
  width: 100%;
  min-width: 0;
  max-width: 100%;
  box-sizing: border-box;
  padding: 10px 12px;
  border: 1px solid #444;
  border-radius: 6px;
  font-size: 14px;
  background: #222;
  color: var(--text-primary);
}

.date-picker.compact {
  padding: 8px 12px;
  text-align: center;
}

.date-picker:focus {
  outline: none;
  border-color: var(--accent-color);
}

.date-picker::-webkit-date-and-time-value {
  min-width: 0;
  text-align: center;
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

/* Desktop / tablet adjustments */
@media (max-width: 768px) {
  .date-controls {
    padding: 12px;
    background: #222;
    border-radius: 8px;
  }

  .date-controls .date-picker {
    font-size: 16px;
  }
}

/* Mobile: glyph-only mode (apply regardless of .compact) */
@media (max-width: 480px) {
  .date-controls {
    display: grid;
    grid-template-columns: 44px minmax(0, 1fr) 44px;
    align-items: center;
    gap: 8px;
    padding: 8px 10px;
    width: 100%;
    margin: 0 auto;
  }
  /* show glyphs on mobile */
  .date-controls .glyph {
    display: inline-block;
  }

  /* hide textual labels inside buttons on mobile */
  .date-controls .label {
    display: none;
  }

  /* make buttons circular glyph-only on mobile */
  .date-controls .nav-btn,
  .date-controls .today-btn {
    width: 44px;
    height: 44px;
    min-width: 44px;
    padding: 0;
    border-radius: 50%;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    font-size: 18px;
  }

  .date-controls .date-picker {
    padding: 10px 6px;
  }

  .date-controls .today-btn {
    grid-column: 1 / -1;
    justify-self: center;
  }
}
</style>

<script lang="ts">
// Provide a runtime default export for environments/tools that expect one
// (keeps compatibility with older import behaviors / typecheckers)
export default {}
</script>
