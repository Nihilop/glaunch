<template>
  <div class="p-6">
    <h2 class="text-2xl font-bold mb-6">Paramètres</h2>

    <!-- Steam Section -->
    <div class="rounded-lg shadow p-4 mb-6">
      <h3 class="text-lg font-semibold mb-4">Connexion Steam</h3>
      <div class="space-y-4">
        <div v-if="steam.loading" class="text-blue-600">
          <div class="animate-spin inline-block mr-2">⚪</div>
          Chargement...
        </div>

        <div v-else-if="steam.error" class="text-red-600 p-3 bg-red-50 rounded">
          {{ steam.error }}
        </div>

        <div v-else-if="steamProfile" class="space-y-4">
          <div class="flex items-center gap-4">
            <img
              :src="steamProfile.avatarfull"
              :alt="steamProfile.personaname"
              class="w-16 h-16 rounded"
            />
            <div>
              <div class="font-bold">{{ steamProfile.personaname }}</div>
              <div class="text-sm text-gray-600">
                Status: {{ getPersonaState(steamProfile.personastate) }}
              </div>
            </div>
          </div>

          <button
            @click="disconnectSteam"
            class="px-4 py-2 bg-red-500 text-white rounded hover:bg-red-600"
          >
            Déconnecter
          </button>
        </div>

        <div v-else>
          <button
            @click="startSteamAuth"
            class="px-4 py-2 bg-[#171a21] text-white rounded hover:bg-[#2a475e] flex items-center gap-2"
          >
            Se connecter avec Steam
          </button>
        </div>
      </div>
    </div>

    <!-- Battle.net Section -->
    <div class="rounded-lg shadow p-4 mb-6">
      <h3 class="text-lg font-semibold mb-4">Connexion Battle.net</h3>
      <div class="space-y-4">
        <div v-if="battlenet.loading" class="text-blue-600">
          <div class="animate-spin inline-block mr-2">⚪</div>
          Chargement...
        </div>

        <div v-else-if="battlenet.error" class="text-red-600 p-3 bg-red-50 rounded">
          {{ battlenet.error }}
        </div>

        <div v-else-if="battlenetProfile" class="space-y-4">
          <div class="flex items-center gap-4">
            <div class="bg-[#148EFF] w-16 h-16 rounded flex items-center justify-center text-white font-bold text-xl">
              {{ battlenetProfile.battletag.charAt(0) }}
            </div>
            <div>
              <div class="font-bold">{{ battlenetProfile.battletag }}</div>
            </div>
          </div>

          <button
            @click="disconnectBattleNet"
            class="px-4 py-2 bg-red-500 text-white rounded hover:bg-red-600"
          >
            Déconnecter
          </button>
        </div>

        <div v-else>
          <button
            @click="startBattleNetAuth"
            class="px-4 py-2 bg-[#148EFF] text-white rounded hover:bg-blue-600 flex items-center gap-2"
          >
            Se connecter avec Battle.net
          </button>
        </div>
      </div>
    </div>

    <!-- Epic Games Section -->
    <div class="rounded-lg shadow p-4 mb-6">
      <h3 class="text-lg font-semibold mb-4">Connexion Epic Games</h3>
      <div class="space-y-4">
        <div v-if="epic.loading" class="text-blue-600">
          <div class="animate-spin inline-block mr-2">⚪</div>
          Chargement...
        </div>

        <div v-else-if="epic.error" class="text-red-600 p-3 bg-red-50 rounded">
          {{ epic.error }}
        </div>

        <div v-else-if="epicProfile" class="space-y-4">
          <div class="flex items-center gap-4">
            <div class="bg-black w-16 h-16 rounded flex items-center justify-center text-white font-bold text-xl">
              {{ epicProfile.displayName.charAt(0) }}
            </div>
            <div>
              <div class="font-bold">{{ epicProfile.displayName }}</div>
            </div>
          </div>

          <button
            @click="disconnectEpic"
            class="px-4 py-2 bg-red-500 text-white rounded hover:bg-red-600"
          >
            Déconnecter
          </button>
        </div>

        <div v-else>
          <button
            @click="startEpicAuth"
            class="px-4 py-2 bg-black text-white rounded hover:bg-gray-800 flex items-center gap-2"
          >
            Se connecter avec Epic Games
          </button>
        </div>
      </div>
    </div>

    <!-- Debug Panel -->
    <div class="mt-6 p-4 bg-gray-900 rounded">
      <h4 class="font-semibold mb-2">Debug Info</h4>
      <div class="space-y-4 text-sm font-mono">
        <!-- Steam Debug -->
        <div>
          <div class="font-bold">Steam:</div>
          <div>ID: {{ steam.id }}</div>
          <pre class="whitespace-pre-wrap break-all">{{ steamProfile }}</pre>
        </div>

        <!-- Battle.net Debug -->
        <div>
          <div class="font-bold">Battle.net:</div>
          <pre class="whitespace-pre-wrap break-all">{{ battlenetProfile }}</pre>
        </div>

        <!-- Epic Debug -->
        <div>
          <div class="font-bold">Epic:</div>
          <pre class="whitespace-pre-wrap break-all">{{ epicProfile }}</pre>
        </div>
      </div>
    </div>
    <AppSettings />
  </div>
</template>

<script setup lang="ts">
import { useAuth } from '@/composables/useAuth'
import AppSettings from "@/components/AppSettings.vue";

const {
  steam,
  steamProfile,
  battlenet,
  battlenetProfile,
  epic,
  epicProfile,
  startSteamAuth,
  disconnectSteam,
  startBattleNetAuth,
  disconnectBattleNet,
  startEpicAuth,
  disconnectEpic
} = useAuth()

function getPersonaState(state: number): string {
  const states = {
    0: 'Hors ligne',
    1: 'En ligne',
    2: 'Occupé',
    3: 'Absent',
    4: 'Endormi',
    5: 'Prêt à échanger',
    6: 'Prêt à jouer'
  }
  return states[state as keyof typeof states] || 'Inconnu'
}
</script>