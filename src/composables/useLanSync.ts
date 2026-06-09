import { invoke } from '@tauri-apps/api/core'
import { useDebounceFn } from '@vueuse/core'
import { message } from 'antdv-next'
import { onMounted, ref, watch } from 'vue'

import type { LanStatePayload } from '@/stores/lan'

import { INVOKE_KEY, LISTEN_KEY } from '@/constants'
import { useLanStore } from '@/stores/lan'

import { useTauriListen } from './useTauriListen'

export function useLanSync() {
  const lanStore = useLanStore()
  const ready = ref(false)

  const syncState = async () => {
    try {
      const state = await invoke<LanStatePayload>(INVOKE_KEY.GET_LAN_SYNC_STATE)

      lanStore.applyState(state)
    } catch (error) {
      message.error(String(error))
    }
  }

  const reconcile = async () => {
    try {
      if (lanStore.settings.enabled) {
        await invoke(INVOKE_KEY.START_LAN_SYNC, {
          settings: {
            nickname: lanStore.settings.nickname.trim(),
            port: Number(lanStore.settings.port),
          },
        })

        return
      }

      await invoke(INVOKE_KEY.STOP_LAN_SYNC)
      lanStore.clearRemoteClients()
    } catch (error) {
      message.error(String(error))
    }
  }

  const debouncedReconcile = useDebounceFn(() => {
    void reconcile()
  }, 300)

  useTauriListen<LanStatePayload>(LISTEN_KEY.LAN_STATE_CHANGED, ({ payload }) => {
    lanStore.applyState(payload)
  })

  onMounted(async () => {
    lanStore.init()
    await syncState()
    ready.value = true
    await reconcile()
  })

  watch(() => [lanStore.settings.enabled, lanStore.settings.nickname, lanStore.settings.port], () => {
    if (!ready.value) return

    debouncedReconcile()
  })
}
