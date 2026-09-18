import type { PropsWithChildren, ReactNode } from 'react'

import { useAuth } from '@/features/auth/AuthProvider'
import type { PermissionKey } from '@/lib/permissions'

/**
 * The one place UI gets to ask "can the current user do this?" — features
 * render `<RequirePermission permission={PERMISSIONS.ProductsCreate}>`
 * instead of re-deriving the check inline, so authorization logic doesn't
 * end up duplicated (and drifting) across every screen.
 */
export function RequirePermission({
  permission,
  fallback = null,
  children,
}: PropsWithChildren<{ permission: PermissionKey; fallback?: ReactNode }>) {
  const { hasPermission } = useAuth()
  return hasPermission(permission) ? <>{children}</> : <>{fallback}</>
}
