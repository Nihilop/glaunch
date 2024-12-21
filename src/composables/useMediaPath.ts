import {convertFileSrc} from '@tauri-apps/api/core'
import {appDataDir, join } from '@tauri-apps/api/path'

export function useMediaPath() {
  // Cache pour éviter de résoudre plusieurs fois le même chemin
  const pathCache = new Map<string, string>()


  const resolveMediaPath = async (path: string | null): Promise<string> => {
    if (!path) return '/placeholder-game.png'

    const appDataDirPath = await appDataDir()
    console.log("Full path:", appDataDirPath + '/' + path) // Log pour debug

    try {
      if (path.startsWith('media/')) {
        // Assurez-vous que le chemin est correctement formaté
        const fullPath = await join(appDataDirPath, path);

        const convertedPath = convertFileSrc(fullPath)
        console.log("Converted path:", convertedPath) // Log pour debug

        pathCache.set(path, convertedPath)
        return convertedPath
      }

      return '/placeholder-game.png'
    } catch (error) {
      console.error('Failed to resolve media path:', error)
      return '/placeholder-game.png'
    }
  }

  return {
    resolveMediaPath
  }
}