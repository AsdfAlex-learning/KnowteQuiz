import type { ConnectionTestResult, DataBackupResult, Settings } from '../types/settings'
import { invoke, isTauri } from './tauri'

async function throwHttpError(res: Response): Promise<never> {
  const body = await res.text()
  throw new Error(body ? `HTTP ${res.status}: ${body}` : `HTTP ${res.status}`)
}

export async function getSettings(): Promise<Settings> {
  if (isTauri()) {
    return invoke<Settings>('get_settings')
  }
  const res = await fetch('/api/settings')
  if (!res.ok) await throwHttpError(res)
  return res.json()
}

export async function saveSettings(settings: Settings): Promise<boolean> {
  if (isTauri()) {
    return invoke<boolean>('save_settings', { settings })
  }
  const res = await fetch('/api/settings', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(settings),
  })
  if (!res.ok) await throwHttpError(res)
  return res.json()
}

export async function testConnection(): Promise<ConnectionTestResult> {
  if (isTauri()) {
    return invoke<ConnectionTestResult>('test_connection')
  }
  const res = await fetch('/api/test-connection', { method: 'POST' })
  if (!res.ok) await throwHttpError(res)
  return res.json()
}

export async function backupData(): Promise<DataBackupResult> {
  if (isTauri()) {
    return invoke<DataBackupResult>('backup_data')
  }
  const res = await fetch('/api/data/backup', { method: 'POST' })
  if (!res.ok) await throwHttpError(res)
  return res.json()
}
