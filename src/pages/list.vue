<template>
  <div class="flex h-screen overflow-hidden ">
    <!-- Sidebar Filters - Desktop -->
    <aside class="hidden lg:flex w-80 flex-shrink-0 flex-col gap-4 border-r border-background p-6 border-t overflow-y-auto bg-muted text-muted-foreground ">
      <div class="space-y-6">
        <!-- Search -->
        <div>
          <h3 class="text-sm mb-2">Search</h3>
          <Input
            v-model="searchQuery"
            placeholder="Search games..."
            class="bg-gray-800/50"
          >
            <template #prefix>
              <Search class="h-4 w-4 text-gray-400" />
            </template>
          </Input>
        </div>

        <!-- View Mode Toggle -->
        <div>
          <h3 class="text-sm mb-2">View Mode</h3>
          <div class="flex gap-2">
            <Button
              variant="ghost"
              @click="handleViewModeChange(viewMode === 'grid' ? 'list' : 'grid')"
            >
              <Grid v-if="viewMode === 'grid'" class="h-4 w-4 mr-2" />
              <List v-else class="h-4 w-4 mr-2" />
              {{viewMode === 'grid' ? 'Grid' : 'Liste'}}
            </Button>
          </div>
        </div>

        <Separator class="bg-background" />

        <!-- Genres Filter -->
        <div>
          <h3 class="text-sm mb-2">Genres</h3>
          <ScrollArea class="h-[250px] pr-4">
            <div v-for="genre in uniqueTags" :key="genre" class="flex items-center space-y-2">

              <Checkbox
                :id="genre"
                :checked="selectedGenres.includes(genre)"
                @update:checked="(checked) => {
                  if (checked) {
                    selectedGenres.push(genre)
                  } else {
                    selectedGenres = selectedGenres.filter(g => g !== genre)
                  }
                 }"
              />
              <label :for="genre" class="ml-2 text-sm">
                {{ genre }} ({{ getGamesCountByTag(genre) }})
              </label>
            </div>
          </ScrollArea>
        </div>

        <Separator class="bg-background" />

        <!-- Recently Played Filter -->
        <div class="flex items-center">
          <Checkbox
            id="recent"
            v-model="showRecentlyPlayed"
            class="border-gray-600"
          />
          <label for="recent" class="ml-2 text-sm">
            Recently Played
          </label>
        </div>

        <Separator class="bg-background" />

        <div class="flex items-center">
          <Button variant="info" @click="resetFilters">
            Reset
          </Button>
        </div>
      </div>
    </aside>


    <!-- Content Area -->
    <main class="flex-1 p-6 h-full border-t border-gray-800 w-full overflow-y-auto">
      <div
        ref="gamesListRef"
        class="max-w-screen-lg"
        :class="[
          viewMode === 'grid'
            ? 'grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-4'
            : 'flex flex-col gap-4'
        ]"
      >
        <GameCard
          v-for="(game, index) in filteredGames"
          :key="game.id"
          :game="game"
          :is-active="isGameActive && gameActiveIndex === index"
          :loading="loadingMedia[game.id]"
          :view-mode="viewMode"
          :show-title="viewMode === 'grid'"
        />
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import {computed, nextTick, onMounted, onUnmounted, ref, watch} from 'vue'
import {invoke} from '@tauri-apps/api/core'
import {useRouter} from 'vue-router'
import {useRegion, useZone} from '@/composables/KeyboardPlugin'
import {Grid, List, Search} from 'lucide-vue-next'
import {Button} from '@/components/ui/button'
import {Input} from '@/components/ui/input'
import {Checkbox} from '@/components/ui/checkbox'
import {ScrollArea} from '@/components/ui/scroll-area'
import {Separator} from '@/components/ui/separator'

const router = useRouter()

// Refs
const gamesListRef = ref<HTMLElement | null>(null)

// State
const games = ref<any[]>([])
const searchQuery = ref('')
const viewMode = ref<'grid' | 'list'>('grid')
const selectedGenres = ref<string[]>([])
const showRecentlyPlayed = ref(false)
const initialLoading = ref(true)
const loadingMedia = ref<Record<string, boolean>>({})

// Navigation Setup
const { regionId } = useRegion({
  priority: 2,
  defaultZone: 'gamesList'
})

// Utils
const getGridColumns = (): number => {
  if (window.innerWidth >= 1024) return 5 // lg
  if (window.innerWidth >= 768) return 4  // md
  if (window.innerWidth >= 640) return 3  // sm
  return 2                                // default
}

// Zone registration
const { isActive: isGameActive, activeIndex: gameActiveIndex, updateBounds, updateZoneState, setActiveElement } = useZone(gamesListRef, {
  id: 'gamesList',
  type: computed(() => viewMode.value === 'grid' ? 'grid' : 'vertical').value,
  columns: computed(() => viewMode.value === 'grid' ? getGridColumns() : 1).value,
  hoverable: true,
  memory: true,
  onSelect: (index) => {
    const game = filteredGames.value[index]
    if (game) {
      router.push(`/details/${game.id}`)
    }
  }
})

// Computed
const uniqueTags = computed(() => {
  const tags = new Set<string>()
  games.value.forEach(game => {
    game.metadata.genres?.forEach((genre: string) => tags.add(genre))
    game.metadata.tags?.forEach((tag: string) => tags.add(tag))
  })
  return Array.from(tags)
})

const filteredGames = computed(() => {
  let filtered = games.value

  // Filtre par recherche
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    filtered = filtered.filter(game =>
      game.name.toLowerCase().includes(query)
    )
  }

  // Filtre par genres sélectionnés
  if (selectedGenres.value.length > 0) {
    filtered = filtered.filter(game => {
      const gameTags = [
        ...(game.metadata.genres || []),
        ...(game.metadata.tags || [])
      ]
      // Retourner true si au moins un des genres sélectionnés est présent dans les tags du jeu
      return selectedGenres.value.some(genre => gameTags.includes(genre))
    })
  }

  // Filtre par derniers joués
  if (showRecentlyPlayed.value) {
    filtered = filtered.filter(game => game.last_played)
      .sort((a, b) => (b.last_played || 0) - (a.last_played || 0))
  }

  return filtered
})

// Methods
const getGamesCountByTag = (tag: string): number => {
  return games.value.filter(game => {
    const gameTags = [
      ...(game.metadata.genres || []),
      ...(game.metadata.tags || [])
    ]
    return gameTags.includes(tag)
  }).length
}

const loadGames = async (useCache = true) => {
  try {
    const result = await invoke<any[]>('scan_games', { useCache })
    games.value = result.map(game => ({
      ...game,
      media: game.media || {}
    }))

    if (!useCache) {
      for (const game of games.value) {
        loadingMedia.value[game.id] = true
        try {
          await invoke('update_game_metadata', { gameId: game.id })
          const updatedGame = await invoke<any>('get_game', { gameId: game.id })
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
    }
  } catch (error) {
    console.error('Failed to load games:', error)
  } finally {
    initialLoading.value = false
  }
}

// Watch pour le redimensionnement
const handleResize = () => {
  nextTick(() => {
    if (gamesListRef.value) {
      updateBounds()
    }
  })
}
const handleViewModeChange = (newMode: 'grid' | 'list') => {
  viewMode.value = newMode
  nextTick(() => {
    updateBounds()
    // Conserver l'élément actif si possible
    if (gameActiveIndex.value >= 0) {
      setActiveElement(gameActiveIndex.value)
    }
  })
}

// Lifecycle
onMounted(() => {
  loadGames(true)
  window.addEventListener('resize', handleResize)
  nextTick(() => {
    if (gamesListRef.value) {
      updateZoneState()
    }
  })
})

onUnmounted(() => {
  window.removeEventListener('resize', handleResize)
})

// Watchers
watch([filteredGames], () => {
  nextTick(() => {
    updateBounds()
    // Si aucun élément n'est sélectionné et qu'il y a des résultats, sélectionner le premier
    if ((!isGameActive.value || gameActiveIndex.value === -1) && filteredGames.value.length > 0) {
      setActiveElement(0)
    }
  })
})

// Reset filters
const resetFilters = () => {
  searchQuery.value = ''
  selectedGenres.value = []
  showRecentlyPlayed.value = false
}
</script>

<style>
main {
  scroll-behavior: smooth;
  height: calc(100vh - 96px); /* 24 * 4 pour pt-24 */
}

.game-card {
  transition: transform 0.3s ease;
}
</style>