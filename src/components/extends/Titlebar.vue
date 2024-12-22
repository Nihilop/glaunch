<template>
  <div data-tauri-drag-region class="select-none flex fixed top-0 left-0 right-0 justify-end h-12 items-center transition-all duration-500 space-x-4 px-5 z-40 group bg-gradient-to-b hover:from-background hover:from-50% hover:to-transparent">
    <div @click="minimize()" class="flex items-center justify-center hover:bg-white/20 p-1 rounded-md cursor-pointer opacity-0 group-hover:opacity-100 transition-opacity duration-500" id="titlebar-minimize">
      <Minus class="w-4 h-4"/>
    </div>
    <div @click="maximize()" class="flex items-center justify-center hover:bg-white/20 p-1 rounded-md cursor-pointer opacity-0 group-hover:opacity-100 transition-opacity duration-500" id="titlebar-maximize">
      <Maximize class="w-4 h-4"/>
    </div>
    <div @click="close()" class="flex items-center justify-center hover:bg-white/20 p-1 rounded-md cursor-pointer opacity-0 group-hover:opacity-100 transition-opacity duration-500" id="titlebar-close">
      <CircleX class="w-4 h-4"/>
    </div>
  </div>
</template>

<script setup lang="ts">
import { CircleX, Maximize, Minus } from 'lucide-vue-next'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { onMounted, onUnmounted } from 'vue'
import { Event as TauriEvent, listen } from '@tauri-apps/api/event'

const appWindow = getCurrentWindow()

async function minimize() {
  await appWindow.minimize()
}

async function maximize() {
  await appWindow.toggleMaximize()
}

async function close() {
  // La fenêtre sera minimisée dans le tray ou fermée selon les paramètres
  await appWindow.close()
}

// Écouter les événements de mise à jour
async function setupUpdateListener() {
  const unlisten = await listen('update-available', (event: TauriEvent<any>) => {
    //...
  })

  return unlisten
}

let unlistenUpdate: (() => void) | undefined

onMounted(async () => {
  unlistenUpdate = await setupUpdateListener()
})

onUnmounted(() => {
  if (unlistenUpdate) {
    unlistenUpdate()
  }
})
</script>