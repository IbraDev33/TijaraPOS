import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { useAuth } from '@/features/auth/AuthProvider'
import { RequirePermission } from '@/features/auth/RequirePermission'
import { useLowStock } from '@/features/inventory/queries'
import { PERMISSIONS } from '@/lib/permissions'

export function DashboardPage() {
  const { user } = useAuth()

  return (
    <div className="flex flex-col gap-4">
      <h1 className="text-2xl font-semibold">Dashboard</h1>
      {user && (
        <p className="text-muted-foreground max-w-2xl text-sm">
          Signed in as <span className="text-foreground font-medium">{user.full_name}</span>.
        </p>
      )}

      <RequirePermission permission={PERMISSIONS.StockView}>
        <LowStockWidget />
      </RequirePermission>

      <p className="text-muted-foreground text-sm">
        Today's sales, revenue, and top products land here in Phase 9 (Reports).
      </p>
    </div>
  )
}

function LowStockWidget() {
  const lowStock = useLowStock()
  const products = lowStock.data ?? []

  return (
    <Card className="max-w-md">
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          Low stock
          {products.length > 0 && <Badge variant="destructive">{products.length}</Badge>}
        </CardTitle>
      </CardHeader>
      <CardContent>
        {products.length === 0 ? (
          <p className="text-muted-foreground text-sm">Nothing is running low.</p>
        ) : (
          <ul className="flex flex-col gap-1 text-sm">
            {products.slice(0, 5).map((product) => (
              <li key={product.id} className="flex justify-between">
                <span>{product.name}</span>
                <span className="text-muted-foreground">
                  {product.current_stock} / {product.min_stock}
                </span>
              </li>
            ))}
            {products.length > 5 && (
              <li className="text-muted-foreground">and {products.length - 5} more…</li>
            )}
          </ul>
        )}
      </CardContent>
    </Card>
  )
}
