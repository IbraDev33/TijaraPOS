import { Button } from '@/components/ui/button'
import { useAuth } from '@/features/auth/AuthProvider'

export function DashboardPage() {
  const { user, logout } = useAuth()

  return (
    <div className="bg-background text-foreground flex min-h-svh flex-col items-center justify-center gap-4 p-8">
      <h1 className="text-2xl font-semibold">TijaraPOS Desktop</h1>
      {user && (
        <>
          <p className="text-muted-foreground">
            Signed in as <span className="font-medium">{user.full_name}</span> ({user.username})
          </p>
          <p className="text-muted-foreground max-w-md text-center text-sm">
            Permissions: {user.permissions.join(', ')}
          </p>
        </>
      )}
      <Button onClick={() => void logout()}>Log out</Button>
    </div>
  )
}
