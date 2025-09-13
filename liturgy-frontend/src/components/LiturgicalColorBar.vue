<script setup lang="ts">
import { computed } from 'vue'
import { getColorValue } from '../utils/liturgical'

interface Props {
  color: string
  size?: 'small' | 'medium' | 'large'
  orientation?: 'vertical' | 'horizontal'
}

const props = withDefaults(defineProps<Props>(), {
  size: 'medium',
  orientation: 'vertical',
})

const sizeMap = {
  small: { width: '3px', height: '2rem' },
  medium: { width: '4px', height: '3rem' },
  large: { width: '6px', height: '4rem' },
}

const style = computed(() => {
  const dimensions = sizeMap[props.size]
  const baseStyle = {
    backgroundColor: getColorValue(props.color),
    borderRadius: '2px',
    flexShrink: '0',
  }

  if (props.orientation === 'horizontal') {
    return {
      ...baseStyle,
      width: dimensions.height,
      height: dimensions.width,
    }
  }

  return {
    ...baseStyle,
    width: dimensions.width,
    height: dimensions.height,
  }
})
</script>

<template>
  <div class="liturgical-color-bar" :style="style"></div>
</template>

<style scoped>
@import '../styles/liturgical.css';
.liturgical-color-bar {
  /* any local one-off tweaks can go here; base styling lives in liturgical.css */
  border-radius: 2px;
}
</style>
