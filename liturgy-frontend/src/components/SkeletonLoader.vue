<script setup lang="ts">
interface Props {
  variant?: 'card' | 'table' | 'compact'
  count?: number
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'card',
  count: 1
})
</script>

<template>
  <!-- Card Skeleton -->
  <div v-if="variant === 'card'" class="skeleton-container">
    <div v-for="n in count" :key="n" class="card-skeleton">
      <div class="skeleton-header">
        <div class="skeleton-line skeleton-title"></div>
        <div class="skeleton-line skeleton-subtitle"></div>
      </div>
      <div class="skeleton-content">
        <div class="skeleton-section">
          <div class="skeleton-feast">
            <div class="skeleton-color-bar"></div>
            <div class="skeleton-line skeleton-feast-text"></div>
          </div>
          <div class="skeleton-line skeleton-rank"></div>
          <div class="skeleton-line skeleton-season"></div>
        </div>
      </div>
    </div>
  </div>

  <!-- Table Skeleton -->
  <div v-else-if="variant === 'table'" class="table-skeleton">
    <div class="skeleton-table-header">
      <div class="skeleton-cell skeleton-date-header"></div>
      <div class="skeleton-cell skeleton-calendar-header"></div>
      <div class="skeleton-cell skeleton-calendar-header"></div>
    </div>
    <div v-for="n in count" :key="n" class="skeleton-table-row">
      <div class="skeleton-cell skeleton-date-cell">
        <div class="skeleton-line skeleton-day-name"></div>
        <div class="skeleton-line skeleton-day-date"></div>
      </div>
      <div class="skeleton-cell skeleton-liturgy-cell">
        <div class="skeleton-feast">
          <div class="skeleton-color-bar"></div>
          <div class="skeleton-line skeleton-feast-text"></div>
        </div>
        <div class="skeleton-line skeleton-rank"></div>
      </div>
      <div class="skeleton-cell skeleton-liturgy-cell">
        <div class="skeleton-feast">
          <div class="skeleton-color-bar"></div>
          <div class="skeleton-line skeleton-feast-text"></div>
        </div>
        <div class="skeleton-line skeleton-rank"></div>
      </div>
    </div>
  </div>

  <!-- Compact Skeleton -->
  <div v-else-if="variant === 'compact'" class="compact-skeleton">
    <div v-for="n in count" :key="n" class="compact-skeleton-item">
      <div class="skeleton-line skeleton-compact-text"></div>
    </div>
  </div>
</template>

<style scoped>
.skeleton-container {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.card-skeleton {
  background: #111;
  border-radius: 4px;
  border: 1px solid #333;
  padding: 16px;
  animation: pulse 1.5s ease-in-out infinite;
}

.skeleton-header {
  margin-bottom: 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid #333;
}

.skeleton-content {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.skeleton-section {
  padding: 12px;
  background: #222;
  border-radius: 8px;
  border: 1px solid #333;
}

.skeleton-line {
  height: 16px;
  background: #333;
  border-radius: 4px;
  margin-bottom: 8px;
}

.skeleton-line:last-child {
  margin-bottom: 0;
}

.skeleton-title {
  height: 20px;
  width: 60%;
  margin-bottom: 8px;
}

.skeleton-subtitle {
  height: 14px;
  width: 40%;
}

.skeleton-feast {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.skeleton-color-bar {
  width: 6px;
  height: 20px;
  background: #333;
  border-radius: 3px;
  flex-shrink: 0;
}

.skeleton-feast-text {
  height: 16px;
  width: 70%;
  margin-bottom: 0;
}

.skeleton-rank {
  height: 12px;
  width: 50%;
}

.skeleton-season {
  height: 12px;
  width: 45%;
}

/* Table Skeleton */
.table-skeleton {
  background: #111;
  border-radius: 4px;
  border: 1px solid #333;
  overflow: hidden;
}

.skeleton-table-header {
  display: flex;
  background: #222;
  border-bottom: 2px solid #333;
}

.skeleton-table-row {
  display: flex;
  border-bottom: 1px solid #333;
}

.skeleton-cell {
  padding: 12px 8px;
  border-right: 1px solid #333;
  flex: 1;
}

.skeleton-cell:last-child {
  border-right: none;
}

.skeleton-date-header {
  width: 120px;
  min-width: 120px;
  flex: none;
}

.skeleton-calendar-header {
  min-width: 200px;
}

.skeleton-date-cell {
  width: 120px;
  min-width: 120px;
  flex: none;
  background: #111;
}

.skeleton-liturgy-cell {
  min-width: 200px;
}

.skeleton-day-name {
  height: 14px;
  width: 80%;
  margin-bottom: 4px;
}

.skeleton-day-date {
  height: 12px;
  width: 60%;
}

/* Compact Skeleton */
.compact-skeleton {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.compact-skeleton-item {
  padding: 8px;
}

.skeleton-compact-text {
  height: 14px;
  width: 100%;
}

/* Animations */
@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.7;
  }
}

@media (max-width: 768px) {
  .card-skeleton {
    padding: 12px;
  }
  
  .skeleton-section {
    padding: 8px;
  }
  
  .skeleton-cell {
    padding: 8px 4px;
  }
  
  .skeleton-date-cell {
    width: 90px;
    min-width: 90px;
  }
  
  .skeleton-calendar-header {
    min-width: 180px;
  }
  
  .skeleton-liturgy-cell {
    min-width: 180px;
  }
}
</style>
