<template>
  <RouterView />
</template>

<script setup lang="ts">
import { listen } from '@tauri-apps/api/event'
import { useRouter } from 'vue-router'
import {onMounted} from "vue";

const router = useRouter()
let previousRoute = '/'

onMounted(async () => {
  await listen('toggle-overlay-view', () => {
    if (router.currentRoute.value.path === '/overlay') {
      router.push(previousRoute)
    } else {
      previousRoute = router.currentRoute.value.path
      router.push('/overlay')
    }
  })
})
</script>

<style>
html, body, #app {
  overflow-x: hidden;
}

/* Supprimer l'outline par défaut */
*:focus {
  outline: none;
}

/* Ajouter un style personnalisé pour l'accessibilité */
*:focus-visible {
  /* Personnaliser selon vos besoins, par exemple : */
  outline: none;
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.5);
  /* Ou utiliser ring de Tailwind */
  @apply ring-2 ring-blue-500 ring-opacity-50;
}

/* Pour supprimer complètement tout style de focus */
*:focus, *:focus-visible {
  outline: none !important;
  box-shadow: none !important;
  @apply ring-0 !important;
}


body {
  --sb-track-color: #111927;
  --sb-thumb-color: #4949ff;
  --sb-size: 7px;
  background: var(--sb-track-color);
}

::-webkit-scrollbar {
  width: var(--sb-size)
}

::-webkit-scrollbar-track {
  background: var(--sb-track-color);
  border-radius: 7px;
}

::-webkit-scrollbar-thumb {
  background: var(--sb-thumb-color);
  border-radius: 7px;
  border: 2px solid var(--sb-track-color);
}

@supports not selector(::-webkit-scrollbar) {
  body {
    scrollbar-color: var(--sb-thumb-color)
    var(--sb-track-color);
  }
}
</style>