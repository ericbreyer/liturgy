<script setup lang="ts">
interface Props {
  message: string
  type?: 'error' | 'warning' | 'info'
  dismissible?: boolean
  compact?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  type: 'error',
  dismissible: false,
  compact: false,
})

const emit = defineEmits<{
  dismiss: []
}>()

function handleDismiss() {
  emit('dismiss')
}

const iconMap = {
  error: '❌',
  warning: '⚠️',
  info: 'ℹ️',
}
</script>

<template>
  <div class="error-display" :class="[`type-${type}`, { compact }]">
    <div class="error-content">
      <span class="error-icon">{{ iconMap[type] }}</span>
      <span class="error-message">{{ message }}</span>
    </div>
    <button v-if="dismissible" @click="handleDismiss" class="dismiss-button" aria-label="Dismiss">
      ✕
    </button>
  </div>
</template>

<style scoped>
@import '../styles/liturgical.css';
.error-display {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-radius: 8px;
  border: 1px solid;
  margin: 12px 0;
  font-size: 14px;
}

.error-display.compact {
  padding: 8px 12px;
  margin: 8px 0;
  font-size: 12px;
}

.error-display.type-error {
  background-color: var(--surface-primary);
  border-color: var(--error-color);
  color: var(--error-color);
}

.error-display.type-warning {
  background-color: var(--surface-primary);
  border-color: var(--warning-color);
  color: var(--warning-color);
}

.error-display.type-info {
  background-color: var(--surface-primary);
  border-color: var(--info-color);
  color: var(--info-color);
}

.error-content {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
}

.error-icon {
  font-size: 16px;
  flex-shrink: 0;
}

.compact .error-icon {
  font-size: 14px;
}

.error-message {
  line-height: 1.4;
  word-break: break-word;
}

.dismiss-button {
  background: none;
  border: none;
  font-size: 16px;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  color: inherit;
  opacity: 0.7;
  transition: opacity 0.2s;
  flex-shrink: 0;
  margin-left: 8px;
}

.dismiss-button:hover {
  opacity: 1;
  background-color: rgba(0, 0, 0, 0.1);
}

.compact .dismiss-button {
  font-size: 14px;
  padding: 2px;
}

@media (max-width: 480px) {
  .error-display {
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
  }

  .dismiss-button {
    align-self: flex-end;
    margin-left: 0;
  }
}
</style>
