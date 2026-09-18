import { Route, Routes } from 'react-router-dom'

import { DashboardPage } from '@/routes/DashboardPage'

export function AppRoutes() {
  return (
    <Routes>
      <Route path="/" element={<DashboardPage />} />
    </Routes>
  )
}
