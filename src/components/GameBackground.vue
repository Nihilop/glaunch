<!-- components/GameImage.vue -->
<template>
  <div class="relative w-full h-full">
    <img
      :src="resolvedSrc"
      :alt="alt"
      :class="[
        'w-full h-full object-cover opacity-100',
        loading ? 'opacity-0' : 'opacity-100',
        className
      ]"
      @error="handleError"
      @load="handleLoad"
    />
    <div
      v-if="loading"
      class="absolute inset-0 flex items-center justify-center bg-gray-900/20 backdrop-blur-sm"
    >
      <LoadingSpinner class="w-6 h-6" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { useMediaPath } from '@/composables/useMediaPath'
import LoadingSpinner from './LoadingSpinner.vue'

const props = defineProps<{
  src: string | null
  alt: string
  className?: string
}>()

const loading = ref(true)
const resolvedSrc = ref<string>('/placeholder-game.png')
const { resolveMediaPath } = useMediaPath()

const loadImage = async () => {
  loading.value = true
  resolvedSrc.value = await resolveMediaPath(props.src)
}

const handleError = () => {
  // make another placeholder background
  resolvedSrc.value = '/placeholder-game.png'
  loading.value = false
}

const handleLoad = () => {
  loading.value = false
}

watch(() => props.src, loadImage)

onMounted(loadImage)
</script>