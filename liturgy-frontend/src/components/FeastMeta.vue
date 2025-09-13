<script setup lang="ts">
import { computed } from 'vue'
import { getCalendarName } from '../utils/liturgical'

interface Props {
  rank: string
  calendars?: string | string[]
  commemorationCount?: number
  size?: 'small' | 'medium' | 'large'
}

const props = withDefaults(defineProps<Props>(), {
  size: 'medium',
})

const calendarList = computed(() => {
  if (!props.calendars) return []
  return Array.isArray(props.calendars) ? props.calendars : [props.calendars]
})

const sizeClasses = computed(() => ({
  [`feast-meta--${props.size}`]: true,
}))
</script>

<template>
  <div class="feast-meta" :class="sizeClasses">
    <span class="feast-rank">{{ rank }}</span>

    <span v-if="calendarList.length > 0" class="feast-calendars">
      (<span v-for="(cal, index) in calendarList" :key="cal"
        >{{ getCalendarName(cal) }}<span v-if="index < calendarList.length - 1">, </span></span
      >)
    </span>

    <span v-if="commemorationCount" class="commemoration-count"> +{{ commemorationCount }} </span>
  </div>
</template>

<style scoped>
@import '../styles/liturgical.css';
.feast-meta {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  color: var(--text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.feast-meta--small {
  font-size: 0.45rem;
}

.feast-meta--medium {
  font-size: 0.5rem;
}

.feast-meta--large {
  font-size: 0.6rem;
}

.feast-rank {
  font-weight: 500;
}

.feast-calendars {
  color: var(--text-subtle);
}

.commemoration-count {
  font-weight: 600;
  color: var(--text-muted);
  background: var(--surface-elevated);
  padding: 0.0625rem 0.1875rem;
  border-radius: 0.1875rem;
  white-space: nowrap;
  flex-shrink: 0;
}

.feast-meta--small .commemoration-count {
  font-size: 0.45rem;
  padding: 0.0625rem 0.125rem;
}

.feast-meta--large .commemoration-count {
  font-size: 0.6rem;
}
</style>
