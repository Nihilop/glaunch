import { onMounted, onUnmounted, ref } from 'vue'
import type { Ref } from 'vue'

interface HorizontalScrollOptions {
  forceHorizontal?: boolean
  sensitivity?: number
  smooth?: boolean
}

export function useHorizontalWheelScroll(
  elementRef: Ref<HTMLElement | null>,
  options: HorizontalScrollOptions = {}
) {
  const {
    forceHorizontal = false,
    sensitivity = 1,
    smooth = true
  } = options

  const scrollAmount = ref(0)
  const isScrolling = ref(false)

  const handleWheel = (event: WheelEvent) => {
    if (!elementRef.value) return

    // Si on force le scroll horizontal ou si Shift est pressé
    if (forceHorizontal || event.shiftKey) {
      event.preventDefault()

      // On utilise deltaY pour le scroll vertical de la souris
      scrollAmount.value += event.deltaY * sensitivity

      if (!isScrolling.value) {
        isScrolling.value = true
        requestAnimationFrame(animateScroll)
      }
    }
  }

  const animateScroll = () => {
    if (!elementRef.value) return

    const currentScrollLeft = elementRef.value.scrollLeft
    const targetScrollLeft = currentScrollLeft + scrollAmount.value
    const distance = targetScrollLeft - currentScrollLeft
    const step = distance * 0.1 // Ajustez cette valeur pour contrôler la vitesse de l'animation

    if (Math.abs(distance) < 1) {
      elementRef.value.scrollLeft = targetScrollLeft
      scrollAmount.value = 0
      isScrolling.value = false
    } else {
      elementRef.value.scrollLeft += step
      scrollAmount.value -= step
      requestAnimationFrame(animateScroll)
    }
  }

  const setupEventListener = () => {
    if (!elementRef.value) return

    // S'assurer que l'élément peut scroll horizontalement
    elementRef.value.style.overflowX = 'scroll'
    elementRef.value.style.overflowY = 'hidden'

    // Ajouter le listener avec { passive: false } pour permettre preventDefault()
    elementRef.value.addEventListener('wheel', handleWheel, { passive: false })
  }

  const cleanupEventListener = () => {
    if (!elementRef.value) return
    elementRef.value.removeEventListener('wheel', handleWheel)
  }

  onMounted(() => {
    setupEventListener()
  })

  onUnmounted(() => {
    cleanupEventListener()
  })

  // Retourner une méthode pour réinitialiser manuellement si nécessaire
  return {
    resetEventListener: () => {
      cleanupEventListener()
      setupEventListener()
    }
  }
}
