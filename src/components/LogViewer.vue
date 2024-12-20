<template>
  <div class="fixed bottom-0 right-0 p-4 w-full bg-black/50 text-white max-h-96 overflow-y-auto z-50">
    <div v-for="log in logStore.logs" :key="log.timestamp">
      <div :class="{
        'text-red-500': log.type === 'error',
        'text-yellow-500': log.type === 'warn',
        'text-blue-500': log.type === 'info',
      }">
        [{{ log.timestamp }}] {{ log.type.toUpperCase() }}: {{ log.message }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useLogStore } from '@/stores/log';
const logStore = useLogStore();
</script>