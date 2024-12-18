// useSound.ts
import { ref } from 'vue'
import { soundConfig } from './config'

export type SoundType = keyof typeof soundConfig.sounds

const soundCache = new Map<string, HTMLAudioElement>()

export function useSound() {
  const isMuted = ref(!soundConfig.enabled)

  // Précharger les sons
  const preloadSounds = () => {
    Object.entries(soundConfig.sounds).forEach(([type, path]) => {
      if (!soundCache.has(type)) {
        const audio = new Audio(path)
        audio.preload = 'auto'
        soundCache.set(type, audio)
      }
    })
  }

  const playSound = async (type: SoundType) => {
    if (isMuted.value) return

    try {
      let sound = soundCache.get(type)
      if (!sound) {
        sound = new Audio(soundConfig.sounds[type])
        soundCache.set(type, sound)
      }

      sound.currentTime = 0
      await sound.play().catch(error => {
        console.warn(`[Sound] Failed to play ${type}:`, error)
      })
    } catch (error) {
      console.warn(`[Sound] Error with sound ${type}:`, error)
    }
  }

  // Précharger les sons au montage
  if (typeof window !== 'undefined') {
    preloadSounds()
  }

  return {
    playSound,
    isMuted,
    toggleMute: () => {
      isMuted.value = !isMuted.value
    }
  }
}