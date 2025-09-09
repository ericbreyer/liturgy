<script setup lang="ts">
interface Props {
  title?: string
  showBackButton?: boolean
  backRoute?: string
  maxWidth?: string
  padding?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  maxWidth: '1200px',
  padding: true
})

const emit = defineEmits<{
  back: []
}>()

function handleBack() {
  emit('back')
}
</script>

<template>
  <div class="page-layout" :style="{ maxWidth, padding: padding ? undefined : '0' }">
    <header v-if="title || showBackButton" class="page-header">
      <button 
        v-if="showBackButton" 
        @click="handleBack"
        class="back-button"
        aria-label="Go back"
      >
        ← Back
      </button>
      <h2 v-if="title" class="page-title">{{ title }}</h2>
    </header>
    
    <div class="page-content">
      <slot />
    </div>
  </div>
</template>

<style scoped>
.page-layout {
  width: var(--layout-fixed-width);
  max-width: 100vw; /* Fallback for very small screens */
  margin: 0 auto;
  padding: 0 20px;
  min-height: 100%;
  box-sizing: border-box;
}

.page-header {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 20px;
  padding: 16px 0;
  border-bottom: 1px solid #333;
}

.back-button {
  background: none;
  border: 1px solid #444;
  padding: 8px 12px;
  border-radius: 6px;
  font-size: 14px;
  color: #6c757d;
  cursor: pointer;
  transition: all 0.2s;
  flex-shrink: 0;
}

.back-button:hover {
  background: #222;
  border-color: #adb5bd;
  color: #495057;
}

.page-title {
  font-size: 24px;
  font-weight: 600;
  color: #333;
  margin: 0;
  flex: 1;
}

.page-content {
  flex: 1;
}

@media (max-width: 768px) {
  .page-layout {
    padding: 0 16px;
  }
  
  .page-header {
    padding: 12px 0;
    margin-bottom: 16px;
  }
  
  .page-title {
    font-size: 20px;
  }
  
  .back-button {
    padding: 6px 10px;
    font-size: 13px;
  }
}

@media (max-width: 480px) {
  .page-layout {
    padding: 0 12px;
  }
  
  .page-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
  }
  
  .page-title {
    font-size: 18px;
    width: 100%;
  }
  
  .back-button {
    align-self: flex-start;
  }
}
</style>
