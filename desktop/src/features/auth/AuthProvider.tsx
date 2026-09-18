import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { createContext, useContext, useMemo, type PropsWithChildren } from 'react'

import * as authApi from '@/features/auth/api'
import type { AuthenticatedUser } from '@/features/auth/api'
import type { PermissionKey } from '@/lib/permissions'

interface AuthContextValue {
  user: AuthenticatedUser | null
  needsSetup: boolean
  isLoading: boolean
  login: (username: string, password: string) => Promise<void>
  bootstrapAdmin: (input: { username: string; password: string; fullName: string }) => Promise<void>
  logout: () => Promise<void>
  hasPermission: (key: PermissionKey) => boolean
}

const AuthContext = createContext<AuthContextValue | null>(null)

const NEEDS_SETUP_KEY = ['auth', 'needsSetup'] as const
const CURRENT_USER_KEY = ['auth', 'currentUser'] as const

export function AuthProvider({ children }: PropsWithChildren) {
  const queryClient = useQueryClient()

  const needsSetupQuery = useQuery({
    queryKey: NEEDS_SETUP_KEY,
    queryFn: authApi.needsSetup,
  })

  const currentUserQuery = useQuery({
    queryKey: CURRENT_USER_KEY,
    queryFn: authApi.currentUser,
    // The backend has nothing to check a session against until setup has
    // run, so don't fire this query (and don't show a loading flicker)
    // until we know setup is done.
    enabled: needsSetupQuery.data === false,
  })

  const loginMutation = useMutation({
    mutationFn: authApi.login,
    onSuccess: (user) => queryClient.setQueryData(CURRENT_USER_KEY, user),
  })

  const bootstrapMutation = useMutation({
    mutationFn: authApi.bootstrapAdmin,
    onSuccess: (user) => {
      queryClient.setQueryData(NEEDS_SETUP_KEY, false)
      queryClient.setQueryData(CURRENT_USER_KEY, user)
    },
  })

  const logoutMutation = useMutation({
    mutationFn: authApi.logout,
    onSuccess: () => queryClient.setQueryData(CURRENT_USER_KEY, null),
  })

  const user = currentUserQuery.data ?? null

  const value = useMemo<AuthContextValue>(
    () => ({
      user,
      needsSetup: needsSetupQuery.data ?? false,
      isLoading:
        needsSetupQuery.isPending || (needsSetupQuery.data === false && currentUserQuery.isPending),
      login: async (username, password) => {
        await loginMutation.mutateAsync({ username, password })
      },
      bootstrapAdmin: async (input) => {
        await bootstrapMutation.mutateAsync(input)
      },
      logout: async () => {
        await logoutMutation.mutateAsync()
      },
      hasPermission: (key) => user?.permissions.includes(key) ?? false,
    }),
    [
      user,
      needsSetupQuery.data,
      needsSetupQuery.isPending,
      currentUserQuery.isPending,
      loginMutation,
      bootstrapMutation,
      logoutMutation,
    ],
  )

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>
}

export function useAuth() {
  const context = useContext(AuthContext)
  if (!context) {
    throw new Error('useAuth must be used within an AuthProvider')
  }
  return context
}
