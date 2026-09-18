import { NavLink, Outlet } from 'react-router-dom'

import { Button } from '@/components/ui/button'
import { useAuth } from '@/features/auth/AuthProvider'
import { PERMISSIONS, type PermissionKey } from '@/lib/permissions'
import { cn } from '@/lib/utils'

const NAV_ITEMS: { to: string; label: string; end: boolean; permission?: PermissionKey }[] = [
  { to: '/', label: 'Dashboard', end: true },
  { to: '/pos', label: 'Checkout', end: false, permission: PERMISSIONS.SalesCreate },
  { to: '/sales', label: 'Sales', end: false, permission: PERMISSIONS.SalesView },
  { to: '/products', label: 'Products', end: false, permission: PERMISSIONS.ProductsView },
]

export function AppLayout() {
  const { user, logout, hasPermission } = useAuth()
  const visibleItems = NAV_ITEMS.filter(
    (item) => !item.permission || hasPermission(item.permission),
  )

  return (
    <div className="bg-background text-foreground flex min-h-svh flex-col">
      <header className="flex items-center justify-between gap-4 border-b px-6 py-3">
        <div className="flex items-center gap-6">
          <span className="text-sm font-semibold">TijaraPOS</span>
          <nav className="flex items-center gap-1">
            {visibleItems.map((item) => (
              <NavLink
                key={item.to}
                to={item.to}
                end={item.end}
                className={({ isActive }) =>
                  cn(
                    'rounded-md px-3 py-1.5 text-sm font-medium transition-colors',
                    isActive
                      ? 'bg-secondary text-secondary-foreground'
                      : 'text-muted-foreground hover:bg-secondary/50 hover:text-foreground',
                  )
                }
              >
                {item.label}
              </NavLink>
            ))}
          </nav>
        </div>
        <div className="flex items-center gap-3">
          {user && (
            <span className="text-muted-foreground text-sm">
              {user.full_name} <span className="text-xs">({user.username})</span>
            </span>
          )}
          <Button variant="outline" size="sm" onClick={() => void logout()}>
            Log out
          </Button>
        </div>
      </header>
      <main className="flex-1 overflow-y-auto p-6">
        <Outlet />
      </main>
    </div>
  )
}
