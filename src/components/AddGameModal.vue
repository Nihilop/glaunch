<template>
  <Dialog :open="isOpen" @update:open="isOpen = $event">
    <DialogTrigger asChild>
      <CirclePlus class="w-5 h-5 hover:scale-110 transition-all duration-500 text-foreground" />
    </DialogTrigger>

    <DialogContent class="sm:max-w-[425px]">
      <DialogHeader>
        <DialogTitle>Add Custom Game</DialogTitle>
        <DialogDescription>
          Add a custom game to your library. You can update metadata later.
        </DialogDescription>
      </DialogHeader>

      <div class="grid gap-4 py-4">
        <!-- Game Title -->
        <div class="grid gap-2">
          <Label for="title">Game Title</Label>
          <Input
            id="title"
            v-model="formData.title"
            placeholder="Enter game title"
          />
        </div>

        <!-- Executable Path -->
        <div class="grid gap-2">
          <Label for="exec">Executable</Label>
          <div class="flex gap-2">
            <Input
              id="exec"
              v-model="formData.executable_path"
              readonly
              placeholder="Select game executable"
            />
            <Button
              variant="outline"
              size="icon"
              @click="selectFile('executable')"
            >
              <Folder class="h-4 w-4" />
            </Button>
          </div>
        </div>

        <!-- Installation Path -->
        <div class="grid gap-2">
          <Label for="install">Installation Directory</Label>
          <div class="flex gap-2">
            <Input
              id="install"
              v-model="formData.install_path"
              readonly
              placeholder="Select installation directory"
            />
            <Button
              variant="outline"
              size="icon"
              @click="selectFile('install')"
            >
              <Folder class="h-4 w-4" />
            </Button>
          </div>
        </div>

        <!-- Icon Path (Optional) -->
        <div class="grid gap-2">
          <Label for="icon">Icon (Optional)</Label>
          <div class="flex gap-2">
            <Input
              id="icon"
              v-model="formData.icon_path"
              readonly
              placeholder="Select game icon"
            />
            <Button
              variant="outline"
              size="icon"
              @click="selectFile('icon')"
            >
              <Folder class="h-4 w-4" />
            </Button>
          </div>
        </div>
      </div>

      <DialogFooter>
        <Button
          :disabled="!isFormValid || loading"
          @click="handleSubmit"
        >
          <template v-if="loading">
            <Loader2 class="mr-2 h-4 w-4 animate-spin" />
            Adding...
          </template>
          <template v-else>
            Add Game
          </template>
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Folder, Loader2, CirclePlus } from 'lucide-vue-next'

interface FormData {
  title: string
  executable_path: string
  install_path: string
  icon_path: string
}

const emit = defineEmits(['gameAdded'])

const isOpen = ref(false)
const loading = ref(false)
const formData = ref<FormData>({
  title: '',
  executable_path: '',
  install_path: '',
  icon_path: '',
})

const isFormValid = computed(() => {
  return formData.value.title &&
    formData.value.executable_path &&
    formData.value.install_path
})

async function selectFile(type: 'executable' | 'install' | 'icon') {
  try {
    const selected = await open({
      directory: type === 'install',
      filters: type === 'icon' ? [{
        name: 'Image',
        extensions: ['png', 'jpg', 'jpeg']
      }] : undefined
    })

    if (selected) {
      switch(type) {
        case 'executable':
          formData.value.executable_path = selected as string
          break
        case 'install':
          formData.value.install_path = selected as string
          break
        case 'icon':
          formData.value.icon_path = selected as string
          break
      }
    }
  } catch (error) {
    console.error('Failed to select file:', error)
  }
}

async function handleSubmit() {
  if (!isFormValid.value) return

  try {
    loading.value = true
    await invoke('add_custom_game', {
      title: formData.value.title,
      executablePath: formData.value.executable_path,
      installPath: formData.value.install_path,
      iconPath: formData.value.icon_path,
      platform: 'custom'
    })
    isOpen.value = false
    emit('gameAdded')

    // Reset form
    formData.value = {
      title: '',
      executable_path: '',
      install_path: '',
      icon_path: '',
    }
  } catch (error) {
    console.error('Failed to add game:', error)
  } finally {
    loading.value = false
  }
}
</script>