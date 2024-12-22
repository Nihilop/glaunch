import {defineStore} from 'pinia'
import {useStorage} from '@vueuse/core'
import {computed} from "vue";

export const useAppStore = defineStore('app', () => {
  // State
  const nickname = useStorage('nickname', '')

  // getter
  const pseudo = computed(() => nickname.value)

  // Action
  async function setNickname(pseudo: string) {
    nickname.value = pseudo
  }
  return {
    pseudo,
    setNickname,
    nickname
  }
})