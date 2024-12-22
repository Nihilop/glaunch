<template>
  <div class="min-h-screen">
    <Titlebar />
    <div class="flex">
      <Navbar />
      <div class="flex-1 w-full pl-16">
        <RouterView />
        <GameStatusCard />

        <LogViewer v-if="showLogs" />
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import LogViewer from "@/components/LogViewer.vue";
import {onMounted, ref} from "vue";
const showLogs = ref(false);

onMounted(() => {
  window.addEventListener('keydown', (e) => {
    if (e.ctrlKey && e.altKey && e.key === 'l') {
      showLogs.value = !showLogs.value;
    }
  });
});
</script>