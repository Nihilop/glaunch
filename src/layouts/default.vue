<template>
  <div class="min-h-screen bg-gray-900/20 text-white">
    <Titlebar />
    <div class="flex">
      <Navbar />
      <div class="flex-1 w-full pl-16">
        <RouterView />
        <GameStatusCard />
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
</script>