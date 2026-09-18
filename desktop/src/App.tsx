import { useAuth } from '@/features/auth/AuthProvider'
import { LoginPage } from '@/features/auth/LoginPage'
import { SetupPage } from '@/features/auth/SetupPage'
import { AppProviders } from '@/providers/AppProviders'
import { AppRoutes } from '@/routes'

function AppShell() {
  const { isLoading, needsSetup, user } = useAuth()

  if (isLoading) {
    return (
      <div className="bg-background text-muted-foreground flex min-h-svh items-center justify-center">
        Loading…
      </div>
    )
  }

  if (needsSetup) {
    return <SetupPage />
  }

  if (!user) {
    return <LoginPage />
  }

  return <AppRoutes />
}

export function App() {
  return (
    <AppProviders>
      <AppShell />
    </AppProviders>
  )
}

export default App
