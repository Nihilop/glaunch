<template>
  <main class="relative min-h-screen bg-gray-900 text-white overflow-hidden flex flex-col pt-24">
    <!-- Search Input -->

    <!-- Dynamic Background -->
    <div class="fixed inset-0 transition-opacity duration-700 overflow-hidden">
      <GameBackground v-if="activeGame?.media?.background" :src="activeGame.media.background" alt="background" />
      <div class="absolute inset-0 bg-gradient-to-t from-gray-900 via-80% via-gray-900/40 to-gray-900/85 "/>
    </div>

    <!-- Content -->
    <div class="relative z-10 flex-1 flex flex-col">
      <!-- Tags Navigation -->
      <nav class="relative px-4 pt-6 overflow-hidden w-full flex flex-col">
        <input
          ref="inputRef"
          type="text"
          v-model="query"
          class="z-10 rounded-md text-xl uppercase w-full px-2 py-2 max-w-screen-md mx-auto bg-white/20 backdrop-blur -translate-y-24 transition-all duration-500 border-transparent focus-visible:border-white/10 border "
          :class="{'!translate-y-0' : query}"
        />
        <div :class="{'translate-y-24': query}" class="transition-all duration-500">
          <h1  class="text-6xl translate-y-0 mb-4">
            Bonjour, joueur #000
          </h1>
          <span class="text-normal opacity-40 pl-8">
          Taper pour rechercher...
        </span>
        </div>
      </nav>

      <!-- Games List -->
      <section class="px-4 mt-8 flex-1 flex flex-col">
        <!-- Loading State -->
        <div v-if="initialLoading" class="flex items-center justify-center min-h-[50vh]">
          <div class="flex items-center gap-3 bg-white/10 px-6 py-4 rounded-xl backdrop-blur-sm">
            <LoadingSpinner/>
            <span>Chargement de votre bibliothèque...</span>
          </div>
        </div>

        <!-- Games Grid -->
        <div v-else class="mt-8 flex-1 transition-all duration-500">
          <!-- Pattern no result-->
          <PatternBackground
            v-if="query && !filteredGames.length"
            :animate="true"
            :direction="PATTERN_BACKGROUND_DIRECTION.TopRight"
            :variant="PATTERN_BACKGROUND_VARIANT.BigDot"
            class="flex h-[50vh] w-full items-center justify-center text-center"
            size="lg"
            :speed="100000"
          >
            <h1 class="text-3xl w-full">Aucun résultat</h1>
          </PatternBackground>
          <div class="px-4 flex flex-col justify-end  h-full">
            <div
              ref="gamesListRef"
              class="grid grid-flow-col auto-cols-[15rem] gap-4 overflow-x-auto hide-scrollbar"
            >
              <GameCard
                v-for="(game, index) in filteredGames"
                :key="game.id"
                :is-active="isGameActive && gameActiveIndex === index"
                :loading="loadingMedia[game.id]"
                :game
                class="w-[15rem] h-[15rem]"
                view-mode="none"
              />
            </div>

            <transition name="slide-up">
              <div v-show="filteredGames[gameActiveIndex]" :key="filteredGames[gameActiveIndex]">
                <h1 class="text-7xl pl-8">{{ filteredGames[gameActiveIndex]?.name || '' }}</h1>
                <div
                  class="space-x-2 ml-8 mt-4">
                  <GameTag
                    v-for="tag in filteredGames[gameActiveIndex]?.metadata.genres"
                    :key="tag"
                    :is-active="false"
                    class="scale-80"
                  >
                    {{ tag }}
                  </GameTag>
                </div>
                <div class="flex justify-start my-4 pl-8">
                  <Button v-if="filteredGames[gameActiveIndex]" size="lg" @click="launchGame(filteredGames[gameActiveIndex].id)">
                    Play
                  </Button>
                </div>
              </div>
            </transition>
          </div>
        </div>

      </section>

      <div class="fixed bottom-4 right-6 z-40">
        <Button :disabled="isLoading" variant="ghost" @click="loadGames(false)" class="hover:bg-white/20 backdrop-blur hover:text-white">
          <RefreshCcw :class="{'animate-rotate' : isLoading}" />
          {{ isLoading ? 'Chargement..' : '' }}
        </Button>
      </div>
    </div>
  </main>
</template>

<script setup lang="ts">
import {ref, computed, onMounted, watch, nextTick} from 'vue'
import {invoke} from '@tauri-apps/api/core'
import {useRouter} from 'vue-router'
import {onStartTyping, useKeyModifier} from '@vueuse/core'
import {useRegion, useZone} from '@/composables/KeyboardPlugin'
import LoadingSpinner from '@/components/LoadingSpinner.vue'
import GameTag from "@/components/extends/GameTag.vue";
import {RefreshCcw} from "lucide-vue-next";
import {
  PATTERN_BACKGROUND_DIRECTION,
  PATTERN_BACKGROUND_VARIANT,
} from "@/components/extends/background";
import {useMediaPath} from "@/composables/useMediaPath.ts";
import GameBackground from "@/components/GameBackground.vue";

const router = useRouter()
// Refs
const inputRef = ref<HTMLInputElement | null>(null)
const gamesListRef = ref<HTMLElement | null>(null)

// State
const games = ref<any[]>([])
const activeTag = ref<string | null>(null)
const query = ref<string | null>(null)
const initialLoading = ref(true)
const isLoading = ref(false)
const loadingMedia = ref<Record<string, boolean>>({})
const Shifted = useKeyModifier('Shift')

// Navigation Setup - Une seule région
const {regionId} = useRegion({
  priority: 2, // Plus basse priorité que le header
  defaultZone: 'gamesList'
})


// debug overlay test dev
const toggleOverlay = async () => {
  await invoke('toggle_overlay')
}

const {isActive: isGameActive, activeIndex: gameActiveIndex, updateBounds, setActiveElement} = useZone(gamesListRef, {
  id: 'gamesList',
  type: 'horizontal',
  memory: true,
  onSelect: (index) => {
    const game = filteredGames.value[index]
    if (Shifted.value) {
      launchGame(game.id)
    } else {
      router.push(`/details/${game.id}`)
    }
  }
})

// Computed

const filteredGames = computed(() => {
  if (!activeTag.value && !query.value) return games.value

  const filterRe = new RegExp(query.value || '', 'i')
  return games.value.filter(game => {
    const gameTags = [
      ...(game.metadata.genres || []),
      ...(game.metadata.tags || [])
    ]
    return (game.metadata.title.match(filterRe) || gameTags.includes(activeTag.value))
  })
})

const activeGame = computed(() => {
  if (!isGameActive.value) return filteredGames.value[0]
  return filteredGames.value[gameActiveIndex.value] || filteredGames.value[0]
})

// Methods

const loadGames = async (useCache = true) => {
  try {
    const result = await invoke<any[]>('scan_games', {useCache})
    games.value = result.map(game => ({
      ...game,
      media: game.media || {}
    }))

    console.log(result)

    if (!useCache) {
      isLoading.value = true
      for (const game of games.value) {
        loadingMedia.value[game.id] = true
        try {
          await invoke('update_game_metadata', {gameId: game.id})
          const updatedGame = await invoke<any>('get_game', {gameId: game.id})
          if (updatedGame) {
            const index = games.value.findIndex(g => g.id === game.id)
            if (index !== -1) {
              games.value[index] = updatedGame
            }
          }
        } catch (error) {
          console.error(`Failed to update metadata for ${game.id}:`, error)
        } finally {
          loadingMedia.value[game.id] = false
        }
      }
      isLoading.value = false
      window.location.reload()
    }
  } catch (error) {
    console.error('Failed to load games:', error)
  } finally {
    initialLoading.value = false
  }
}

const launchGame = async (gameId: string) => {
  try {
    await invoke('launch_game', {gameId})
  } catch (error) {
    console.error('Failed to launch game:', error)
  }
}

// Start typing handler
onStartTyping(() => {
  if (!inputRef.value?.matches(':focus')) {
    inputRef.value?.focus()
  }
})

watch([query, filteredGames], ([newQuery, games]) => {
  if (newQuery && games.length > 0) {
    // Activer automatiquement le premier élément quand il y a des résultats
    nextTick(() => {
      setActiveElement(0)
    })
  }
})


// Séquence d'initialisation
const initializeComponent = async () => {
  // 1. Charger les données
  await loadGames(true)

  // 3. Mettre à jour les zones
  await nextTick(() => {
    if (gamesListRef.value) {
      updateBounds()
    }
  })
}

// Lifecycle
onMounted(initializeComponent)

watch(filteredGames, () => {
  nextTick(() => {
    if (gamesListRef.value) {
      updateBounds()
    }
  })
})
</script>

<style lang="scss" scoped>
.hide-scrollbar {
  scrollbar-width: none;
  -ms-overflow-style: none;
}

.hide-scrollbar::-webkit-scrollbar {
  display: none;
}

.slide-up-enter-active,
.slide-up-leave-active {
  transition: all 0.5s ease;
}

.slide-up-enter-from {
  position: absolute;
  opacity: 0;
  transform: translateY(20px);
}

.slide-up-leave-to {
  position: absolute;
  opacity: 0;
  transform: translateY(-20px);
}

.animate-rotate {
  animation: rotate 2s infinite;
}

@keyframes rotate {
  to {
    transform: rotate(360deg);
  }
}
</style>