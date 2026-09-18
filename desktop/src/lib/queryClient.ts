import { QueryClient } from '@tanstack/react-query'

// The desktop app is offline-first and talks to a local SQLite-backed API,
// so requests are cheap and reliable: retry sparingly and keep data fresh
// rather than caching aggressively like a remote-latency web app would.
export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      staleTime: 10_000,
      refetchOnWindowFocus: false,
    },
    mutations: {
      retry: 0,
    },
  },
})
