import { Route, Routes } from 'react-router-dom'

import { AppLayout } from '@/components/layout/AppLayout'
import { ProductsPage } from '@/features/products/ProductsPage'
import { CheckoutPage } from '@/features/sales/CheckoutPage'
import { SalesHistoryPage } from '@/features/sales/SalesHistoryPage'
import { DashboardPage } from '@/routes/DashboardPage'

export function AppRoutes() {
  return (
    <Routes>
      <Route element={<AppLayout />}>
        <Route path="/" element={<DashboardPage />} />
        <Route path="/pos" element={<CheckoutPage />} />
        <Route path="/sales" element={<SalesHistoryPage />} />
        <Route path="/products" element={<ProductsPage />} />
      </Route>
    </Routes>
  )
}
