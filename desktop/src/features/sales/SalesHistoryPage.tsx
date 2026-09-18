import { useState } from 'react'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { commandErrorMessage } from '@/features/auth/api'
import { RequirePermission } from '@/features/auth/RequirePermission'
import { useSale, useCancelSale, useSales } from '@/features/sales/queries'
import { ReceiptView } from '@/features/sales/ReceiptView'
import { centsToDisplay } from '@/lib/money'
import { PERMISSIONS } from '@/lib/permissions'

const STATUS_VARIANT = {
  completed: 'success',
  cancelled: 'destructive',
  refunded: 'secondary',
} as const

export function SalesHistoryPage() {
  const sales = useSales()
  const [viewingId, setViewingId] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)
  const viewingSale = useSale(viewingId)
  const cancelSale = useCancelSale()

  return (
    <div className="flex flex-col gap-4">
      <h1 className="text-2xl font-semibold">Sales</h1>
      {error && <p className="text-destructive text-sm">{error}</p>}

      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Invoice</TableHead>
            <TableHead>Date</TableHead>
            <TableHead>Cashier</TableHead>
            <TableHead>Total</TableHead>
            <TableHead>Status</TableHead>
            <TableHead />
          </TableRow>
        </TableHeader>
        <TableBody>
          {sales.isLoading && (
            <TableRow>
              <TableCell colSpan={6} className="text-muted-foreground text-center">
                Loading…
              </TableCell>
            </TableRow>
          )}
          {sales.data?.length === 0 && (
            <TableRow>
              <TableCell colSpan={6} className="text-muted-foreground text-center">
                No sales yet.
              </TableCell>
            </TableRow>
          )}
          {sales.data?.map((sale) => (
            <TableRow key={sale.id}>
              <TableCell className="font-mono text-xs">{sale.invoice_number}</TableCell>
              <TableCell>{new Date(sale.created_at).toLocaleString()}</TableCell>
              <TableCell>{sale.cashier_name}</TableCell>
              <TableCell>{centsToDisplay(sale.total)}</TableCell>
              <TableCell>
                <Badge variant={STATUS_VARIANT[sale.status]}>{sale.status}</Badge>
              </TableCell>
              <TableCell>
                <div className="flex justify-end gap-1">
                  <Button variant="outline" size="sm" onClick={() => setViewingId(sale.id)}>
                    View
                  </Button>
                  {sale.status === 'completed' && (
                    <RequirePermission permission={PERMISSIONS.SalesCancel}>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={async () => {
                          if (
                            !confirm(`Cancel sale ${sale.invoice_number}? Stock will be restored.`)
                          )
                            return
                          setError(null)
                          try {
                            await cancelSale.mutateAsync(sale.id)
                          } catch (err) {
                            setError(commandErrorMessage(err))
                          }
                        }}
                      >
                        Cancel
                      </Button>
                    </RequirePermission>
                  )}
                </div>
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>

      <ReceiptView
        open={viewingId !== null}
        onOpenChange={(open) => !open && setViewingId(null)}
        sale={viewingSale.data ?? null}
      />
    </div>
  )
}
