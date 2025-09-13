<script setup lang="ts">
interface Props {
  text?: string
  size?: 'small' | 'medium' | 'large'
  inline?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  text: 'Loading...',
  size: 'medium',
  inline: false,
})
</script>

<template>
  <div class="loading-container" :class="{ inline: inline, [`size-${size}`]: true }">
    <div class="spinner"></div>
    <p class="loading-text">{{ text }}</p>
  </div>
</template>

<style scoped>
@import '../styles/liturgical.css';
.loading-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 20px;
  gap: 12px;
}

.loading-container.inline {
  flex-direction: row;
  padding: 8px;
  gap: 8px;
}

.loading-container.size-small {
  padding: 12px;
  gap: 8px;
}

.loading-container.size-large {
  padding: 40px;
  gap: 16px;
}

.spinner {
  border: 3px solid #333;
  border-top: 3px solid #007bff;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

.size-small .spinner {
  width: 20px;
  height: 20px;
  border-width: 2px;
}

.size-medium .spinner {
  width: 32px;
  height: 32px;
}

.size-large .spinner {
  width: 48px;
  height: 48px;
  border-width: 4px;
}

.inline .spinner {
  width: 16px;
  height: 16px;
  border-width: 2px;
}

.loading-text {
  color: var(--text-secondary);
  font-size: 14px;
  margin: 0;
  text-align: center;
}

.size-small .loading-text {
  font-size: 12px;
}

.size-large .loading-text {
  font-size: 16px;
}

.inline .loading-text {
  font-size: 12px;
}

@keyframes spin {
  0% {
    transform: rotate(0deg);
  }
  100% {
    transform: rotate(360deg);
  }
}

@media (max-width: 480px) {
  .loading-text {
    font-size: 12px;
  }

  .size-medium .spinner {
    width: 24px;
    height: 24px;
  }
}
</style>
