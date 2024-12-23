<template>
  <div class="rounded-lg shadow bg-muted p-4 mb-6 max-w-screen-lg">
    <h3 class="text-lg font-semibold mb-4 text-foreground">Paramètres système</h3>

    <!-- General Settings -->
    <div class="space-y-6">
      <div class="space-y-3">
        <h4 class="font-medium text-foreground/50">Paramètres généraux</h4>
        <div class="space-y-2">

          <div class="flex items-center space-x-2">
            <Checkbox id="start_with_windows" v-model:checked="settings.start_with_windows" @update:checked="saveSettings"/>
            <label
              for="start_with_windows"
              class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 text-foreground/50"
            >
              Démarrer avec Windows
            </label>
          </div>

          <div class="flex items-center space-x-2">
            <Checkbox id="start_with_windows" v-model:checked="settings.minimize_to_tray" @update:checked="saveSettings"/>
            <label
              for="start_with_windows"
              class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 text-foreground/50"
            >
              Réduire dans la zone de notification
            </label>
          </div>

          <div class="flex items-center space-x-2">
            <Checkbox id="start_with_windows" v-model:checked="settings.check_updates_on_startup" @update:checked="saveSettings"/>
            <label
              for="start_with_windows"
              class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 text-foreground/50"
            >
              Vérifier les mises à jour au démarrage
            </label>
          </div>
        </div>
      </div>

      <!-- Updates Section -->
      <div class="space-y-3">
        <h4 class="font-medium text-foreground/50">Mises à jour</h4>
        <div class="space-y-2">
          <div class="text-sm">
            <span class="font-medium ">Version actuelle :</span>
            <span class="ml-2">{{ updateStatus.current_version }}</span>
          </div>

          <div
            v-if="updateStatus.latest_version && updateStatus.update_available"
            class="text-sm text-green-600 font-medium"
          >
            Nouvelle version disponible : {{ updateStatus.latest_version }}
          </div>

          <div class="flex gap-2 mt-2">
            <Button
              @click="checkUpdates"
              :disabled="isCheckingUpdate"
            >
              <svg
                v-if="isCheckingUpdate"
                class="animate-spin h-4 w-4"
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
              >
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
              {{ isCheckingUpdate ? 'Vérification...' : 'Vérifier les mises à jour' }}
            </Button>

            <Button
              v-if="updateStatus.update_available"
              variant="success"
              @click="installUpdate"
              :disabled="isInstallingUpdate"
            >
              <svg
                v-if="isInstallingUpdate"
                class="animate-spin h-4 w-4"
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
              >
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
              {{ isInstallingUpdate ? 'Installation...' : 'Installer la mise à jour' }}
            </Button>
          </div>

          <div v-if="updateStatus.release_notes" class="mt-2 text-sm">
            <div class="font-medium">Notes de mise à jour :</div>
            <div class="mt-1 text-gray-600 whitespace-pre-line">{{ updateStatus.release_notes }}</div>
          </div>
        </div>
      </div>

      <div>
        <h4 class="font-medium text-foreground/50">Pseudos</h4>
        <Input v-model="appStore.nickname" />
      </div>

      <!-- Database Section -->
      <div class="space-y-3">
        <h4 class="font-medium text-foreground/50">Base de données</h4>
        <div class="flex gap-2">
          <Button
            @click="exportDatabase"
            variant="info"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
              <polyline points="17 8 12 3 7 8"/>
              <line x1="12" y1="3" x2="12" y2="15"/>
            </svg>
            Exporter
          </Button>

          <Button
            @click="importDatabase"
            variant="muted"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
              <polyline points="7 10 12 15 17 10"/>
              <line x1="12" y1="15" x2="12" y2="3"/>
            </svg>
            Importer
          </Button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { save, open } from '@tauri-apps/plugin-dialog'
import {useAppStore} from "@/stores/app.ts";

interface AppSettings {
  start_with_windows: boolean
  minimize_to_tray: boolean
  check_updates_on_startup: boolean
}

interface UpdateStatus {
  current_version: string
  latest_version: string | null
  update_available: boolean
  release_notes?: string | null
  checking: boolean
}

const settings = ref<AppSettings>({
  start_with_windows: false,
  minimize_to_tray: true,
  check_updates_on_startup: true,
})

const updateStatus = ref<UpdateStatus>({
  current_version: '',
  latest_version: null,
  update_available: false,
  release_notes: null,
  checking: false,
})

const isCheckingUpdate = ref(false)
const isInstallingUpdate = ref(false)
const appStore = useAppStore()

async function loadSettings() {
  try {
    const loaded = await invoke<AppSettings>('get_app_settings')
    settings.value = loaded
  } catch (error) {
    console.error('Erreur lors du chargement des paramètres:', error)
  }
}

async function saveSettings() {
  try {
    await invoke('save_app_settings', { settings: settings.value })
  } catch (error) {
    console.error('Erreur lors de la sauvegarde des paramètres:', error)
  }
}

async function checkUpdates() {
  isCheckingUpdate.value = true
  try {
    const status = await invoke<UpdateStatus>('check_for_updates')
    updateStatus.value = status
  } catch (error) {
    console.error('Erreur lors de la vérification des mises à jour:', error)
  } finally {
    isCheckingUpdate.value = false
  }
}

async function installUpdate() {
  isInstallingUpdate.value = true
  try {
    await invoke('install_update')
  } catch (error) {
    console.error('Erreur lors de l\'installation de la mise à jour:', error)
  } finally {
    isInstallingUpdate.value = false
  }
}

async function exportDatabase() {
  const path = await save({
    filters: [{
      name: 'Database',
      extensions: ['db']
    }]
  })

  if (path) {
    try {
      await invoke('export_db', { path })
    } catch (error) {
      console.error('Erreur lors de l\'export de la base de données:', error)
    }
  }
}

async function importDatabase() {
  const path = await open({
    filters: [{
      name: 'Database',
      extensions: ['db']
    }]
  })

  if (path) {
    try {
      await invoke('import_db', { path })
    } catch (error) {
      console.error('Erreur lors de l\'import de la base de données:', error)
    }
  }
}

onMounted(() => {
  loadSettings()
  checkUpdates()
})
</script>