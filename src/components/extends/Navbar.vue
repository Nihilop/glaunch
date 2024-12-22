<template>
  <nav class="w-16 z-50 h-screen fixed top-0 left-0">

    <div
      ref="mainNavRef"
      class="flex flex-col h-full bg-black/20 items-center gap-3 px-2 py-4 backdrop-blur-sm"
      :class="{'border-r  border-gray-800' : isList}"
    >
      <!-- Remplacer RouterLink par des div/buttons -->
      <button
        v-for="(item, index) in menuItems"
        :key="item.path"
        :class="[
          'p-3 rounded-lg transition-all duration-200',
          (isActive && activeIndex === index) && 'ring-2 ring-blue-500 bg-white/10',
          {'mt-auto' : item.path === '#'}
        ]"
        @click="() => router.push(item.path)"
        @mouseover="() => setActiveElement(index)"
      >
        <component
          :is="item.icon"
          class="w-5 h-5 hover:scale-110 transition-all duration-500"
        />
      </button>
    </div>
  </nav>
</template>

<script setup lang="ts">
import {useRegion, useZone} from '@/composables/KeyboardPlugin'
import {useRoute, useRouter} from "vue-router";
import {Home, Library, User} from "lucide-vue-next";
import {ref, watch} from "vue";
import AddGameModal from "@/components/AddGameModal.vue";

const mainNavRef = ref(null)
const isList = ref(false)
const router = useRouter()
const route = useRoute()
const menuItems = ref([
  { path: '/', icon: Home },
  { path: '/list', icon: Library },
  { path: '/settings', icon: User },
  { path: '#', icon: AddGameModal },
])


const { regionId } = useRegion({
  priority: 1,
  persistent: true,
  defaultZone: 'mainNav'
})


const { isActive, activeIndex, setActiveElement } = useZone(mainNavRef, {
  id: 'mainNav',
  type: 'vertical',
  memory: false,
  onSelect: (index) => {
    const item = menuItems.value[index]
    if (item && item.path !== '#') {
      router.push(item.path)
    }
  }
})

watch(
  () => route.fullPath,
  (newPath) => {
    isList.value = newPath.includes('/list');
  },
  { immediate: true } // Pour vérifier dès le montage du composant
);
</script>