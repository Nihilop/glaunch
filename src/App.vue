<template>
  <div>
    <RouterView />

    <Dialog v-model:open="setupNicknameModal">
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Nickname</DialogTitle>
        </DialogHeader>
        <Input v-model="nicknameInput" @keydown.enter="setPseudo"/>
        <Button variant="success" @click="setPseudo">
          Ajouter
        </Button>
      </DialogContent>
    </Dialog>
  </div>
</template>

<script setup lang="ts">
import { listen } from '@tauri-apps/api/event'
import { useRouter } from 'vue-router'
import {onMounted, ref} from "vue";
import {useAppStore} from "@/stores/app.ts";
import {Dialog, DialogContent, DialogHeader, DialogTitle} from "@/components/ui/dialog";


const router = useRouter()
const appStore = useAppStore()
let previousRoute = '/'
const setupNicknameModal = ref(false)
const nicknameInput = ref(appStore.pseudo || '')

async function setPseudo() {
  if(!nicknameInput.value) return
  await appStore.setNickname(nicknameInput.value).finally(() => {
    setupNicknameModal.value = false
  })
}

onMounted(async () => {
  if(!appStore.pseudo) {
    setupNicknameModal.value = true
  }
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

<style lang="scss">
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
  --sb-track-color: #15171e;
  --sb-thumb-color: #1f2937;
  --sb-size: 15px;
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
  border: 5px solid var(--sb-track-color);
}

@supports not selector(::-webkit-scrollbar) {
  body {
    scrollbar-color: var(--sb-thumb-color)
    var(--sb-track-color);
  }
}
</style>