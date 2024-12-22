<template>
  <main class="absolute w-full h-screen bg-gray-900 text-white overflow-hidden">
    <!-- Dynamic Background -->
    <div class="absolute inset-0 transition-opacity duration-500">
      <GameBackground v-if="game?.media?.background || game?.media?.cover" :src="game.media.background || game.media.cover" alt="background" />
      <div class="absolute inset-0 bg-gradient-to-b from-gray-900/85 via-20% via-gray-900/40 to-transparent"/>
    </div>

    <!-- Back Button -->
    <button
      @click="router.back()"
      class="absolute top-4 left-4 py-2 pl-2 pr-4 rounded-lg bg-black/20 hover:bg-black/70 transition-colors z-50 flex items-center space-x-3"
    >
      <ChevronLeft /> <CommandShortcut>Echap</CommandShortcut>
    </button>

    <!-- Content Panel -->
    <div
      ref="contentRef"
      :class="[
        'absolute bottom-0 left-0 right-0 bg-gray-900/95 backdrop-blur-sm',
        'transform transition-transform duration-700 ease-out',
        'h-[70vh]',
        isLoaded ? 'translate-y-0' : 'translate-y-full'
      ]"
    >

      <!-- Panel Content -->
      <div class="p-8 space-y-8 max-h-full">
        <!-- Header -->
        <header class="flex items-end space-x-4 -mt-[150px]">
          <GameImage
            v-if="game"
            :src="game.media?.thumbnail"
            :alt="game.metadata.title"
            type="thumb"
            class="!w-auto"
          />
          <div>
            <div ref="actionsRef" class="flex gap-4 mb-4">
              <button
                v-for="(action, index) in actions"
                :key="action.label"
                :class="[
                    'px-6 py-3 rounded-lg font-medium transition-all duration-200',
                    isActionsActive && actionsActiveIndex === index
                      ? 'ring-4 ring-blue-500'
                      : '',
                    action.primary ? 'bg-blue-600 hover:bg-blue-700' : 'bg-gray-800 hover:bg-gray-700'
                  ]"
                @click="clickActions(index)"
              >
                {{ action.label }}
              </button>
              <Dialog v-model:open="modal">
                <DialogContent class="sm:max-w-[800px] grid-rows-[auto_minmax(0,1fr)_auto] p-0 max-h-[90dvh]">
                  <DialogHeader>
                    <DialogTitle>Update metadata</DialogTitle>
                  </DialogHeader>
                  <div class="grid gap-4 py-4 overflow-y-auto px-6">
                    <div class="flex flex-col justify-between h-[300dvh]">
                      <ul v-if="gameMeta.length" ref="metaUpdateModal" class="space-y-4">
                        <li v-for="(result, index) in gameMeta" :key="index">
                          <Popover>
                            <PopoverTrigger as-child>
                              <div
                                class="flex items-center space-x-4 bg-gray-400/20 shadow overflow-hidden rounded-md cursor-pointer">
                                <img :src="result.cover_url" width="100px" class="rounded-md"
                                     @error="handleImageError"/>
                                <div>
                                  <div class="flex items-center space-x-2">
                                    <p class="text-xl font-bold">{{ result.name }}</p><span
                                    class="opacity-50">ID: {{ result.id }}</span>
                                  </div>
                                  <span>{{ result.release_date }}</span>
                                  <span>{{ result.company }}</span>
                                </div>
                              </div>
                            </PopoverTrigger>
                            <PopoverContent class="w-80">
                              <div>
                                <p>Voulez-vous mettre à jour les metas avec ces informations ?</p>
                                <div class="flex items-center justify-between">
                                  <Button @click="updateMetadata(result.id)">
                                    Oui
                                  </Button>
                                </div>
                              </div>

                            </PopoverContent>
                          </Popover>
                        </li>
                      </ul>
                    </div>
                  </div>
                </DialogContent>
              </Dialog>
            </div>
            <h1 class="text-4xl font-bold mb-2">{{ game?.name }}</h1>
            <div class="flex items-center gap-4 text-gray-400">
              <span>{{ game?.metadata.developer }}</span>
              <span>·</span>
              <span>{{ game?.platform }}</span>
            </div>
          </div>
        </header>

        <div class="grid-rows-[auto_minmax(0,1fr)_auto] p-0 max-h-[90dvh]">
          <div class="grid gap-4 py-4 overflow-y-auto px-6">
            <div class="flex flex-col justify-between h-[45dvh]">
              <!-- Game Stats -->
              <div class="flex gap-8">
                <div class="flex flex-col items-center p-4 rounded-lg hover:bg-gray-800">
                  <span class="text-2xl font-bold">{{ formatGameStats(game?.stats)?.playTime }}</span>
                  <span class="text-sm text-gray-400">Play Time</span>
                </div>

                <div class="flex flex-col items-center p-4 rounded-lg hover:bg-gray-800">
                  <span class="text-2xl font-bold">{{ formatGameStats(game?.stats)?.lastPlayed }}</span>
                  <span class="text-sm text-gray-400">Last Played</span>
                </div>

                <div class="flex flex-col items-center p-4 rounded-lg hover:bg-gray-800">
                  <span class="text-2xl font-bold">{{ formatGameStats(game?.stats)?.sessionsCount }}</span>
                  <span class="text-sm text-gray-400">Times Launched</span>
                </div>
              </div>
              <!-- Game Info -->
              <div class="space-y-6">
                <!-- Description -->
                <div v-if="game?.metadata.description" class="prose prose-invert max-w-none">
                  <h2 class="text-xl font-bold mb-4">About</h2>
                  <div>
                    <p>{{game.metadata.description}}</p>
                  </div>
                </div>

                <!-- Genres -->
                <div v-if="game?.metadata.genres?.length">
                  <h2 class="text-xl font-bold mb-4">Genres</h2>
                  <div class="flex flex-wrap gap-2">
                    <span
                      v-for="genre in game.metadata.genres"
                      :key="genre"
                      class="px-3 py-1 rounded-full bg-gray-800 text-sm"
                    >
                      {{ genre }}
                    </span>
                  </div>
                </div>

                <!-- Installation Info -->
                <div class="space-y-2">
                  <h2 class="text-xl font-bold mb-4">Installation</h2>
                  <p class="font-bold text-gray-400">
                    Size: <span class="font-normal opacity-60">{{ formatSize(game?.installation.install_size || 0) }}</span>
                  </p>
                  <p class="font-bold text-gray-400">
                    Path: <span class="font-normal opacity-60">{{ game?.installation.path }}</span>
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <Dialog v-model:open="showDeleteConfirm">
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete Game</DialogTitle>
            <DialogDescription>
              Are you sure you want to delete "{{ game?.title }}" from your library? This action cannot be undone.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="ghost" @click="showDeleteConfirm = false">Cancel</Button>
            <Button
              variant="destructive"
              :disabled="isDeleting"
              @click="handleDelete"
            >
              <Loader2 v-if="isDeleting" class="mr-2 h-4 w-4 animate-spin" />
              {{ isDeleting ? 'Deleting...' : 'Delete' }}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  </main>
</template>

<script setup lang="ts">
import {onMounted, onUnmounted, ref} from 'vue'
import {useRoute, useRouter} from 'vue-router'
import {useRegion, useZone} from '@/composables/KeyboardPlugin'
import {convertFileSrc, invoke} from "@tauri-apps/api/core"
import {ChevronLeft, Loader2} from 'lucide-vue-next'
import {Dialog, DialogContent, DialogHeader, DialogTitle,} from '@/components/ui/dialog'
import GameBackground from "@/components/GameBackground.vue";
import GameImage from "@/components/GameImage.vue";

interface IgdbSearchResult {
  id: number;
  name: string;
  release_date?: string;
  cover_url?: string;
  company?: string;
}

const route = useRoute()
const router = useRouter()

// Refs
const statsRef = ref<HTMLElement | null>(null)
const actionsRef = ref<HTMLElement | null>(null)
const contentRef = ref<HTMLElement | null>(null)
const metaUpdateModalRef = ref<HTMLElement | null>(null)

// State
const game = ref<any | null>(null)
const isLoaded = ref(false)
const modal = ref(false)
const gameMeta = ref<IgdbSearchResult[]>([])
const showDeleteConfirm = ref(false)
const isDeleting = ref(false)

const formatDuration = (seconds: number): string => {
  if (!seconds || seconds < 0) return '0h 0m'

  const hours = Math.floor(seconds / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  const remainingSeconds = seconds % 60

  if (hours > 0) {
    return `${hours}h ${minutes}m` // Si on a des heures, on montre heures et minutes
  } else if (minutes > 0) {
    return `${minutes}m ${remainingSeconds}s` // Si on a que des minutes, on montre minutes et secondes
  } else {
    return `${remainingSeconds}s` // Si on a que des secondes
  }
}

const formatLastPlayed = (timestamp: number): string => {
  if (!timestamp || timestamp < 0) return 'Never'
  const date = new Date(timestamp * 1000)
  return new Intl.DateTimeFormat('fr-FR', {
    dateStyle: 'medium',
    timeStyle: 'short'
  }).format(date)
}

const formatGameStats = (stats: any) => {
  if (!stats) return null
  return {
    playTime: formatDuration(stats.total_playtime),
    lastPlayed: formatLastPlayed(stats.last_played),
    sessionsCount: stats.sessions_count || 0
  }
}

async function searchIgdbGames() {
  gameMeta.value = await invoke('search_igdb_games', {query: game.value.name});
}

async function updateGameWithIgdb(gameId: string, igdbId: number): Promise<void> {
  return await invoke('update_game_with_igdb', {gameId, igdbId});
}

async function updateMetadata(id: number) {
  await updateGameWithIgdb(game.value.id, id).finally(() => router.push('/'))
}

// Constants
const gameStats = ref([
  {label: 'Hours Played', value: game.value?.stats.total_playtime || 0 },
  {label: 'Last Played', value: game.value?.stats.last_played || 0},
  {label: 'Launched', value: game.value?.stats.sessions_count || 0}
])

const actions = [
  {label: 'Play', primary: true},
  {label: 'Metadata'},
  {label: 'Supprimer'}
]

// Navigation Setup
const {regionId} = useRegion({
  priority: 1,
  persistent: false
})

// Zone Registration
const {isActive: isStatsActive, activeIndex: statsActiveIndex} = useZone(statsRef, {
  id: 'stats',
  type: 'horizontal',
  memory: false,
  regionId,
})
// Zone Registration
const {isActive: isResultActive, activeIndex: resultActiveIndex} = useZone(metaUpdateModalRef, {
  id: 'result',
  type: 'vertical',
  memory: false,
  regionId,
})

const {isActive: isActionsActive, activeIndex: actionsActiveIndex} = useZone(actionsRef, {
  id: 'actions',
  type: 'horizontal',
  memory: false,
  regionId,
  onSelect: async (index) => {
    await clickActions(index)
  }
})

const clickActions = async (index: number ) => {
  switch (index) {
    case 0:
      if (game.value) {
        await launchGame()
      }
      break
    case 1:
      openMetadata()
      break
    case 2:
      showDeleteConfirm.value = true
      break
  }
}

async function handleDelete() {
  if (!game.value?.id) return

  try {
    isDeleting.value = true
    await invoke('delete_game', { gameId: game.value.id })
    router.push('/')
  } catch (error) {
    console.error('Failed to delete game:', error)
  } finally {
    isDeleting.value = false
    showDeleteConfirm.value = false
  }
}

async function openMetadata() {
  await searchIgdbGames().then((res) => {
    modal.value = !modal.value
  })

}

const handleImageError = (event: Event) => {
  const target = event.target as HTMLImageElement
  target.src = '/placeholder-game.png'
}

// Methods
const formatSize = (bytes: number): string => {
  const gb = bytes / 1024 / 1024 / 1024
  return `${gb.toFixed(1)} GB`
}

const loadGame = async () => {
  try {
    const gameData = await invoke<any>('get_game', {gameId: route.params.id})
    if (!gameData) {
      return router.push('/')
    }
    game.value = gameData
  } catch (error) {
    console.error('Error loading game:', error)
    router.push('/')
  } finally {
    isLoaded.value = true
  }
}

const launchGame = async () => {
  if (!game.value?.id) return

  try {
    await invoke('launch_game', {gameId: game.value.id})
  } catch (error) {
    console.error('Error launching game:', error)
  }
}

// Keyboard handler
const handleKeyDown = (e: KeyboardEvent) => {
  if (e.key === 'Escape') {
    router.back()
  }
}

// Lifecycle
onMounted(async () => {
  await loadGame()
  window.addEventListener('keydown', handleKeyDown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown)
})
</script>