import { invoke } from '@tauri-apps/api/core'
import { z } from 'zod'

/**
 * Typed wrappers around the `auth_*` Tauri commands
 * (`desktop/src-tauri/src/commands/auth.rs`). Every response is parsed
 * through a Zod schema — cheap insurance against the Rust and TypeScript
 * shapes drifting apart, not just at compile time but at runtime too.
 */

export const authenticatedUserSchema = z.object({
  id: z.number(),
  username: z.string(),
  full_name: z.string(),
  permissions: z.array(z.string()),
})

export type AuthenticatedUser = z.infer<typeof authenticatedUserSchema>

/** Matches the `CommandError` shape every Tauri command in this app rejects with. */
export interface CommandError {
  code: string
  message: string
}

export function isCommandError(error: unknown): error is CommandError {
  return (
    typeof error === 'object' &&
    error !== null &&
    'code' in error &&
    'message' in error &&
    typeof (error as Record<string, unknown>).message === 'string'
  )
}

export function commandErrorMessage(
  error: unknown,
  fallback = 'Something went wrong. Please try again.',
) {
  return isCommandError(error) ? error.message : fallback
}

export async function needsSetup(): Promise<boolean> {
  return invoke<boolean>('auth_needs_setup')
}

export async function bootstrapAdmin(input: {
  username: string
  password: string
  fullName: string
}): Promise<AuthenticatedUser> {
  const result = await invoke('auth_bootstrap_admin', input)
  return authenticatedUserSchema.parse(result)
}

export async function login(input: {
  username: string
  password: string
}): Promise<AuthenticatedUser> {
  const result = await invoke('auth_login', input)
  return authenticatedUserSchema.parse(result)
}

export async function logout(): Promise<void> {
  await invoke('auth_logout')
}

export async function currentUser(): Promise<AuthenticatedUser | null> {
  const result = await invoke('auth_current_user')
  return result ? authenticatedUserSchema.parse(result) : null
}
