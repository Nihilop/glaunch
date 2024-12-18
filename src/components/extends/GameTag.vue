<template>
  <Button
    class="rounded-full transition-all duration-200 backdrop-blur bg-white/20 hover:bg-white/50"
    :class="{'ring-2 ring-secondary bg-white/30' : isActive}"
  >
    <slot />
  </Button>
</template>

<script setup lang="ts">
interface Props {
  isActive: boolean
}

defineProps<Props>()
</script>