<template>
  <div class="fixed inset-0 bg-black/40 backdrop-blur-sm pt-24">
    <div class="container mx-auto p-4">
      <div class="flex justify-between items-center">
        <h1 class="text-2xl font-bold">GLaunch Overlay</h1>
        <button
          @click="closeOverlay"
          class="px-4 py-2 bg-red-500 hover:bg-red-600 rounded">
          Fermer
        </button>
      </div>

      <!-- Si on a un jeu actif -->
      <div v-if="activeGame" class="mt-4">
        <h2 class="text-xl">{{ activeGame.title }}</h2>
        <!-- Autres infos du jeu -->
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { onMounted, ref } from 'vue'

const activeGame = ref(null)

const closeOverlay = async () => {
  await invoke('toggle_overlay')
}

// On peut récupérer le jeu actif depuis le state
const getActiveGame = async () => {
  try {
    activeGame.value = await invoke('get_active_game')
  } catch (error) {
    console.error('Erreur lors de la récupération du jeu actif:', error)
  }
}

onMounted(getActiveGame)
</script>

<style>
body, html, #app {
  background-color: rgba(2,2,2,0.2) !important;
}
</style>