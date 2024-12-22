<template>
  <Transition
    enter-active-class="transition duration-300 ease-out"
    enter-from-class="transform translate-y-8 opacity-0"
    enter-to-class="transform translate-y-0 opacity-100"
    leave-active-class="transition duration-200 ease-in"
    leave-from-class="transform translate-y-0 opacity-100"
    leave-to-class="transform translate-y-8 opacity-0"
  >
    <div
      v-if="activeGame"
      class="fixed bottom-4 right-4 z-50"
    >
      <Button class="absolute top-1 right-1 z-10 rounded-full py-1 px-3" v-if="!hideCover" variant="ghost" @click="hideCover = true">
        <CircleX />
      </Button>
      <div class="bg-gray-900/95 backdrop-blur-sm border border-gray-700 rounded-lg shadow-lg overflow-hidden p-4">
        <div>
          <GameImage
            v-if="currentGame && !hideCover"
            :src="currentGame.media?.thumbnail"
            :alt="currentGame.metadata.title"
            type="default"
            className="w-36 mx-auto"
          />
        </div>
        <div class="p-4 cursor-pointer" @click="hideCover = false">
          <div class="flex items-center gap-3">
            <div class="h-2 w-2 bg-green-500 rounded-full animate-pulse"></div>
            <span class="text-white/90">{{ activeGame }}</span>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import {invoke} from '@tauri-apps/api/core'
import GameImage from "@/components/GameImage.vue";
import { CircleX } from 'lucide-vue-next'

const activeGame = ref<string | null>(null)
const currentGame = ref()
const hideCover = ref(false)
const activeGameName = ref<string>('')

// Fonction pour récupérer le nom du jeu à partir de son ID
async function getGameName(gameId: string) {
  try {
    const game = await invoke('get_game', { gameId })
    if (game) {
      currentGame.value = game
      activeGame.value = currentGame.value.name
    }
  } catch (error) {
    console.error('Failed to get game details:', error)
  }
}

// Fonction pour vérifier le statut du jeu
async function checkGameStatus() {
  try {
    const gameId = await invoke<string>('get_active_game')

    // Si l'ID du jeu a changé
    if (gameId !== activeGame.value) {
      activeGame.value = gameId
      if (gameId) {
        await getGameName(gameId)
      } else {
        activeGameName.value = ''
      }
    }
  } catch (error) {
    console.error('Failed to get game status:', error)
  }
}

// Configurer l'intervalle de vérification
let statusInterval: number | null = null

onMounted(() => {
  // Vérifier immédiatement
  checkGameStatus()

  // Puis vérifier toutes les secondes
  statusInterval = window.setInterval(checkGameStatus, 1000)
})

onUnmounted(() => {
  if (statusInterval !== null) {
    clearInterval(statusInterval)
  }
})
</script>