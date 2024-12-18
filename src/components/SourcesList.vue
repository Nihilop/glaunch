<template>
  <div class="bg-white rounded-lg shadow p-4 mb-6">
    <h2 class="text-lg font-semibold mb-4">Sources de jeux</h2>
    <div class="space-y-3">
      <div v-for="(library, index) in libraries" :key="index" class="flex items-center justify-between p-3 bg-gray-50 rounded hover:bg-gray-100">
        <div>
          <span class="font-medium">{{ getSourceDisplayName(library) }}</span>
          <span class="text-sm text-gray-500 block">{{ library }}</span>
        </div>
        <span class="text-xs px-2 py-1 bg-blue-100 text-blue-800 rounded-full">Steam</span>
      </div>

      <div class="mt-4">
        <button @click="handleAddSource" class="flex items-center px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600">
          <span class="mr-2">+</span>
          Ajouter une source
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'

const libraries = ref([])

const loadLibraries = async () => {
  try {
    libraries.value = await invoke('get_steam_libraries')
  } catch (error) {
    console.error('Erreur lors du chargement des sources:', error)
  }
}

const getSourceDisplayName = (path) => {
  const parts = path.split('\\')
  return parts[parts.length - 2] || path
}

const handleAddSource = async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: 'Sélectionner un dossier de jeux'
    })

    if (selected) {
      await invoke('add_custom_folder', { path: selected })
      await loadLibraries()
    }
  } catch (error) {
    console.error('Erreur lors de l\'ajout de la source:', error)
  }
}

onMounted(() => {
  loadLibraries()
})
</script>