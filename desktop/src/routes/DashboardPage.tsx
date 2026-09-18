import { Button } from '@/components/ui/button'

export function DashboardPage() {
  return (
    <div className="bg-background text-foreground flex min-h-svh flex-col items-center justify-center gap-4 p-8">
      <h1 className="text-2xl font-semibold">TijaraPOS Desktop</h1>
      <p className="text-muted-foreground">
        Foundation scaffold ready — Tauri, React, Tailwind, TanStack Query.
      </p>
      <Button>Get started</Button>
    </div>
  )
}
