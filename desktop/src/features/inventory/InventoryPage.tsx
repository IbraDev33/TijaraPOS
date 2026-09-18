import { type FormEvent, useState } from 'react'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
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
import { STOCK_ADJUSTMENT_REASONS, type StockAdjustmentReason } from '@/features/inventory/api'
import { useAdjustStock, useLowStock, useMovements } from '@/features/inventory/queries'
import type { Product } from '@/features/products/api'
import { useProducts } from '@/features/products/queries'
import { PERMISSIONS } from '@/lib/permissions'

export function InventoryPage() {
  const lowStock = useLowStock()
  const movements = useMovements()

  return (
    <div className="flex flex-col gap-6">
      <h1 className="text-2xl font-semibold">Inventory</h1>

      <section className="flex flex-col gap-2">
        <h2 className="text-lg font-semibold">Low stock</h2>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>SKU</TableHead>
              <TableHead>Name</TableHead>
              <TableHead>Current stock</TableHead>
              <TableHead>Minimum</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {lowStock.data?.length === 0 && (
              <TableRow>
                <TableCell colSpan={4} className="text-muted-foreground text-center">
                  Nothing is low on stock.
                </TableCell>
              </TableRow>
            )}
            {lowStock.data?.map((product) => (
              <TableRow key={product.id}>
                <TableCell className="font-mono text-xs">{product.sku}</TableCell>
                <TableCell>{product.name}</TableCell>
                <TableCell>
                  <Badge variant="destructive">{product.current_stock}</Badge>
                </TableCell>
                <TableCell>{product.min_stock}</TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </section>

      <RequirePermission permission={PERMISSIONS.StockAdjust}>
        <AdjustStockForm />
      </RequirePermission>

      <section className="flex flex-col gap-2">
        <h2 className="text-lg font-semibold">Recent movements</h2>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Date</TableHead>
              <TableHead>Product</TableHead>
              <TableHead>Change</TableHead>
              <TableHead>Reason</TableHead>
              <TableHead>User</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {movements.data?.length === 0 && (
              <TableRow>
                <TableCell colSpan={5} className="text-muted-foreground text-center">
                  No stock movements yet.
                </TableCell>
              </TableRow>
            )}
            {movements.data?.map((movement) => (
              <TableRow key={movement.id}>
                <TableCell>{new Date(movement.created_at).toLocaleString()}</TableCell>
                <TableCell>{movement.product_name}</TableCell>
                <TableCell
                  className={movement.quantity_delta < 0 ? 'text-destructive' : 'text-success'}
                >
                  {movement.quantity_delta > 0 ? '+' : ''}
                  {movement.quantity_delta}
                </TableCell>
                <TableCell className="capitalize">{movement.reason.replace('_', ' ')}</TableCell>
                <TableCell>{movement.user_name ?? '—'}</TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </section>
    </div>
  )
}

function AdjustStockForm() {
  const [search, setSearch] = useState('')
  const [selected, setSelected] = useState<Product | null>(null)
  const [quantityDelta, setQuantityDelta] = useState('')
  const [reason, setReason] = useState<StockAdjustmentReason>('adjustment')
  const [note, setNote] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)

  const products = useProducts({ search: search.trim() || undefined })
  const adjustStock = useAdjustStock()

  const parsedDelta = Number.parseInt(quantityDelta, 10)
  const canSubmit = selected !== null && Number.isInteger(parsedDelta) && parsedDelta !== 0

  async function onSubmit(e: FormEvent) {
    e.preventDefault()
    if (!canSubmit || !selected) return
    setError(null)
    setSuccess(null)
    try {
      const result = await adjustStock.mutateAsync({
        productId: selected.id,
        quantityDelta: parsedDelta,
        reason,
        note: note.trim() || null,
      })
      setSuccess(`${selected.name} is now at ${result.currentStock} in stock.`)
      setSelected(null)
      setSearch('')
      setQuantityDelta('')
      setNote('')
    } catch (err) {
      setError(commandErrorMessage(err))
    }
  }

  return (
    <section className="flex flex-col gap-3 rounded-lg border p-4">
      <h2 className="text-lg font-semibold">Adjust stock</h2>
      <form className="flex flex-col gap-3" onSubmit={(e) => void onSubmit(e)}>
        <div className="flex flex-col gap-2">
          <Label htmlFor="product-search">Product</Label>
          {selected ? (
            <div className="flex items-center justify-between rounded-md border px-3 py-2 text-sm">
              <span>
                {selected.name} <span className="text-muted-foreground">({selected.sku})</span> —
                currently {selected.current_stock}
              </span>
              <Button type="button" variant="ghost" size="sm" onClick={() => setSelected(null)}>
                Change
              </Button>
            </div>
          ) : (
            <>
              <Input
                id="product-search"
                placeholder="Search by name or SKU…"
                value={search}
                onChange={(e) => setSearch(e.target.value)}
              />
              {search.trim() && (
                <div className="flex max-h-40 flex-col gap-1 overflow-y-auto rounded-md border p-2">
                  {products.data?.map((product) => (
                    <button
                      key={product.id}
                      type="button"
                      className="hover:bg-secondary rounded-md px-2 py-1.5 text-left text-sm"
                      onClick={() => {
                        setSelected(product)
                        setSearch('')
                      }}
                    >
                      {product.name} ({product.sku}) — currently {product.current_stock}
                    </button>
                  ))}
                </div>
              )}
            </>
          )}
        </div>

        <div className="grid grid-cols-2 gap-3">
          <div className="flex flex-col gap-2">
            <Label htmlFor="delta">Quantity change</Label>
            <Input
              id="delta"
              inputMode="numeric"
              placeholder="e.g. 10 or -3"
              value={quantityDelta}
              onChange={(e) => setQuantityDelta(e.target.value)}
            />
          </div>
          <div className="flex flex-col gap-2">
            <Label>Reason</Label>
            <Select value={reason} onValueChange={(v) => setReason(v as StockAdjustmentReason)}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {STOCK_ADJUSTMENT_REASONS.map((r) => (
                  <SelectItem key={r} value={r}>
                    {r[0].toUpperCase() + r.slice(1)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </div>

        <div className="flex flex-col gap-2">
          <Label htmlFor="note">Note (optional)</Label>
          <Input id="note" value={note} onChange={(e) => setNote(e.target.value)} />
        </div>

        {error && <p className="text-destructive text-sm">{error}</p>}
        {success && <p className="text-success text-sm">{success}</p>}

        <Button type="submit" disabled={!canSubmit || adjustStock.isPending} className="w-fit">
          {adjustStock.isPending ? 'Saving…' : 'Apply adjustment'}
        </Button>
      </form>
    </section>
  )
}
