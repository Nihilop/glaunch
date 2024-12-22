<template>
  <div class="bg-white rounded-lg shadow">
    <div class="p-4 border-b">
      <div class="flex justify-between items-center">
        <h2 class="text-lg font-semibold">Jeux ({{ games.length }})</h2>
        <div class="space-x-2">
          <button
            @click="refreshGames"
            class="px-3 py-1 text-sm bg-blue-500 rounded hover:bg-blue-600"
            :disabled="isLoading"
          >
            <span v-if="isLoading" class="inline-flex items-center">
              <svg class="animate-spin -ml-1 mr-2 h-4 w-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
              Chargement...
            </span>
            <span v-else>Rafraîchir</span>
          </button>
        </div>
      </div>
    </div>

    <!-- Loading State -->
    <div v-if="initialLoading" class="p-8 text-center">
      <div class="inline-flex items-center">
        <svg class="animate-spin h-8 w-8 text-blue-500 mr-2" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
        <span class="text-lg text-gray-600">Chargement de votre bibliothèque...</span>
      </div>
    </div>

    <!-- Games Grid -->
    <div v-else class="p-4">
      <div v-if="games.length === 0" class="text-center py-8 text-gray-500">
        Aucun jeu trouvé
      </div>

      <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        <div v-for="game in games" :key="game.id"
             class="border rounded-lg overflow-hidden hover:shadow-lg transition-shadow">
          <!-- Image avec état de chargement -->
          <div class="aspect-w-16 aspect-h-9 bg-gray-100 relative">
            <div v-if="loadingMedia[game.id]" class="absolute inset-0 flex items-center justify-center bg-gray-200/50">
              <svg class="animate-spin h-6 w-6 text-blue-500" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
            </div>
            <img
              v-if="game.media?.thumbnail"
              :src="convertFileSrc(game.media.thumbnail)"
              :alt="game.metadata.title"
              class="w-full h-full object-cover"
              @error="handleImageError"
            >
            <div v-else class="w-full h-full flex items-center justify-center bg-gray-200">
              <span class="text-gray-400">Pas d'image</span>
            </div>
          </div>

          <div class="p-4">
            <h3 class="font-semibold mb-2">{{ game.metadata.title }}</h3>
            <div class="text-sm text-gray-600 space-y-1">
              <p>Taille: {{ formatSize(game.installation.install_size) }}</p>
              <p>Plateforme: {{ game.platform }}</p>
              <p v-if="game.metadata.developer" class="text-sm text-gray-500">
                Par {{ game.metadata.developer }}
              </p>
              <p v-if="game.metadata.publisher" class="text-sm text-gray-500">
                Publié par {{ game.metadata.publisher }}
              </p>
              <div v-if="game.metadata.genres?.length" class="flex flex-wrap gap-1 mt-2">
                <span v-for="genre in game.metadata.genres" :key="genre"
                      class="px-2 py-1 text-xs bg-blue-100 text-blue-800 rounded-full">
                  {{ genre }}
                </span>
              </div>
            </div>
            <div class="mt-4 flex items-center justify-between space-x-2">
              <RouterLink :to="`details/${game.id}`" class="p-4 bg-blue-300">
                {{ `${game.id}` }}
              </RouterLink>
              <button
                @click="launchGame(game.id)"
                class="w-full px-4 py-2 bg-green-500 rounded hover:bg-green-600"
              >
                Lancer
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { invoke, convertFileSrc } from '@tauri-apps/api/core'

const games = ref([])
const isLoading = ref(false)
const initialLoading = ref(true)
const loadingMedia = ref({})

const handleImageError = (event) => {
  event.target.src = '/placeholder-game.png'
}

const formatSize = (bytes) => {
  if (!bytes) return '0 GB'
  const gb = bytes / 1024 / 1024 / 1024
  return `${gb.toFixed(2)} GB`
}

const loadGames = async (useCache = true) => {
  try {
    // Charger d'abord la liste basique des jeux
    const result = await invoke('scan_games', { useCache })
    games.value = result.map(game => ({
      ...game,
      media: game.media || {}
    }))

    // Si on n'utilise pas le cache, mettre à jour les métadonnées pour chaque jeu
    if (!useCache) {
      for (const game of games.value) {
        loadingMedia.value[game.id] = true
        try {
          await invoke('update_game_metadata', { gameId: game.id })
          // Recharger les métadonnées mises à jour
          const updatedGame = await invoke('get_game', { gameId: game.id })
          const index = games.value.findIndex(g => g.id === game.id)
          if (index !== -1) {
            games.value[index] = updatedGame
          }
        } catch (error) {
          console.error(`Failed to update metadata for ${game.id}:`, error)
        } finally {
          loadingMedia.value[game.id] = false
        }
      }
    }
  } catch (error) {
    console.error('Erreur lors du chargement des jeux:', error)
  } finally {
    initialLoading.value = false
    isLoading.value = false
  }
}

const refreshGames = async () => {
  isLoading.value = true
  await loadGames(false) // Force le rechargement sans cache
}

const launchGame = async (gameId) => {
  try {
    await invoke('launch_game', { gameId })
  } catch (error) {
    console.error('Erreur lors du lancement du jeu:', error)
  }
}

onMounted(() => {
  loadGames(true) // Utiliser le cache au chargement initial
})
</script>