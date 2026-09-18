import { useAuth } from '@/features/auth/AuthProvider'

export function DashboardPage() {
  const { user } = useAuth()

  return (
    <div className="flex flex-col gap-2">
      <h1 className="text-2xl font-semibold">Dashboard</h1>
      {user && (
        <p className="text-muted-foreground max-w-2xl text-sm">
          Signed in as <span className="text-foreground font-medium">{user.full_name}</span>.
          Permissions: {user.permissions.join(', ')}
        </p>
      )}
      <p className="text-muted-foreground text-sm">
        Today's sales, low-stock alerts, and recent activity land here in Phase 9 (Reports).
      </p>
    </div>
  )
}
