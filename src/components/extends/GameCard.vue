<template>
  <Card class="relative rounded-xl bg-gray-900/20 backdrop-blur border-none overflow-hidden">
    <div
      @click="goTo"
      :class="[
        'border-4 border-transparent overflow-hidden transition-all duration-300 z-10 group cursor-pointer rounded-xl h-full opacity-50 hover:opacity-100 bg-white:50',
        viewMode === 'grid' ? 'h-48' : '',
        isActive
          ? 'p-1 !border-blue-600 !opacity-100 !blur-none'
          : 'hover:scale-[1.1]',
        loading && 'opacity-60',
        viewMode === 'list' ? 'flex items-center gap-4 !h-24' : '',
      ]"
    >
      <!-- Loading Overlay -->
      <div
        v-if="loading"
        class="absolute inset-0 flex items-center justify-center bg-black/50 backdrop-blur-sm z-10"
      >
        <LoadingSpinner class="w-6 h-6"/>
      </div>

      <!-- Game Image -->
      <GameImage
        :src="game.media?.thumbnail"
        :alt="game.metadata.title"
        :class="[
          'object-cover',
          {'w-full h-full rounded-md' : viewMode === 'grid'},
          {'h-full !w-24 rounded-md' : viewMode === 'list'},
        ]"
        :type="viewMode === 'grid' ? 'thumb' : 'default'"
      />

      <!-- List Mode Info -->
      <div v-if="viewMode === 'list'" class="flex-1 pr-4">
        <h3 class="font-medium">{{ game.metadata.title }}</h3>
        <div class="flex gap-2 mt-1">
          <span v-for="genre in game.metadata.genres?.slice(0, 2)"
                :key="genre"
                class="text-xs bg-white/10 px-2 py-1 rounded-full">
            {{ genre }}
          </span>
        </div>
      </div>

      <!-- Grid Mode Title Overlay -->
      <div
        v-if="showTitle && !loading && viewMode === 'grid'"
        :class="[
          'absolute left-0 bottom-2 right-0 px-2 transition-all duration-300 ease-out group-hover:translate-y-0 group-hover:opacity-100',
          isActive
            ? 'translate-y-0 opacity-100'
            : 'translate-y-8 opacity-0'
        ]"
      >
        <div class="bg-gradient-to-t from-black/90 to-transparent px-3 pt-8 rounded-md">
          <h3 class="font-medium truncate pb-2 select-none">{{ game.metadata.title }}</h3>
        </div>
      </div>
    </div>
  </Card>
</template>

<script setup lang="ts">
import LoadingSpinner from "@/components/LoadingSpinner.vue";
import {convertFileSrc} from "@tauri-apps/api/core";
import {useRouter} from "vue-router";
import GameImage from "@/components/GameImage.vue";

interface Props {
  showTitle?: boolean
  isActive?: boolean
  loading?: boolean
  viewMode?: 'grid' | 'list' | 'none'
  game: any
}

const router = useRouter()
const props = withDefaults(defineProps<Props>(), {
  isActive: false,
  showTitle: false,
  loading: false,
  viewMode: 'grid'
})

const debugImagePath = (path: string) => {
  const converted = convertFileSrc(path);
  console.log({
    original: path,
    converted: converted,
    exists: path.length > 0
  });
  return converted;
}

const handleImageError = (event: Event) => {
  const target = event.target as HTMLImageElement
  target.src = '/placeholder-game.png'
}

const goTo = () => {
  router.push(`/details/${props.game.id}`)
}
</script>