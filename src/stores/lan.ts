import { defineStore } from 'pinia'
import { computed, reactive } from 'vue'

export const DEFAULT_LAN_PORT = 47832

export type LanStatus = 'disabled' | 'listening' | 'error'

export interface LanCursorPosition {
  x: number
  y: number
}

export interface LanInputSnapshot {
  keyboardKeys: string[]
  mouseButtons: string[]
  cursor?: LanCursorPosition
  gamepadButtons: Record<string, number>
  gamepadAxes: Record<string, number>
}

export interface LanLocalClient {
  clientId: string
  nickname: string
  port: number
}

export interface LanRemoteClient {
  clientId: string
  nickname: string
  lastSeenAt: number
  inputState: LanInputSnapshot
}

export interface LanStatePayload {
  enabled: boolean
  status: LanStatus
  error?: string
  localClient: LanLocalClient
  remoteClients: LanRemoteClient[]
}

export const useLanStore = defineStore('lan', () => {
  const settings = reactive({
    enabled: false,
    nickname: '',
    port: DEFAULT_LAN_PORT,
  })

  const local = reactive({
    clientId: '',
    nickname: '',
    port: DEFAULT_LAN_PORT,
    status: 'disabled' as LanStatus,
    error: '',
  })

  const remoteClients = reactive<Record<string, LanRemoteClient>>({})

  const remoteClientsList = computed(() => {
    return Object.values(remoteClients).sort((left, right) => {
      return right.lastSeenAt - left.lastSeenAt
    })
  })

  const remoteClientCount = computed(() => remoteClientsList.value.length)

  const init = () => {
    if (!settings.port || settings.port < 1 || settings.port > 65535) {
      settings.port = DEFAULT_LAN_PORT
    }
  }

  const applyState = (payload: LanStatePayload) => {
    local.clientId = payload.localClient.clientId
    local.nickname = payload.localClient.nickname
    local.port = payload.localClient.port
    local.status = payload.status
    local.error = payload.error ?? ''

    if (!settings.nickname) {
      settings.nickname = payload.localClient.nickname
    }

    settings.port = payload.localClient.port

    for (const clientId of Object.keys(remoteClients)) {
      delete remoteClients[clientId]
    }

    for (const client of payload.remoteClients) {
      remoteClients[client.clientId] = client
    }
  }

  const clearRemoteClients = () => {
    for (const clientId of Object.keys(remoteClients)) {
      delete remoteClients[clientId]
    }
  }

  return {
    settings,
    local,
    remoteClients,
    remoteClientsList,
    remoteClientCount,
    init,
    applyState,
    clearRemoteClients,
  }
}, {
  tauri: {
    filterKeys: ['local', 'remoteClients'],
  },
})
