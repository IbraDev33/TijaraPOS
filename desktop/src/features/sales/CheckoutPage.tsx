import { MinusIcon, PlusIcon, TrashIcon } from 'lucide-react'
import { type KeyboardEvent as ReactKeyboardEvent, useEffect, useRef, useState } from 'react'

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
import type { Product } from '@/features/products/api'
import { getProductByBarcode } from '@/features/products/api'
import { useDiscounts, useProducts, useTaxes } from '@/features/products/queries'
import type { Sale } from '@/features/sales/api'
import type { CartLine } from '@/features/sales/cartMath'
import { computeCartTotals, computeLineTotals } from '@/features/sales/cartMath'
import { ReceiptView } from '@/features/sales/ReceiptView'
import { useCheckout } from '@/features/sales/queries'
import { centsToDisplay, displayToCents } from '@/lib/money'

const PAYMENT_METHODS = [
  { value: 'cash', label: 'Cash' },
  { value: 'card', label: 'Card' },
  { value: 'bank_transfer', label: 'Bank transfer' },
  { value: 'other', label: 'Other' },
]

interface PaymentRow {
  method: string
  amount: string
}

export function CheckoutPage() {
  const [search, setSearch] = useState('')
  const [cart, setCart] = useState<CartLine[]>([])
  const [payments, setPayments] = useState<PaymentRow[]>([{ method: 'cash', amount: '' }])
  const [error, setError] = useState<string | null>(null)
  const [receipt, setReceipt] = useState<{ sale: Sale; change: number } | null>(null)
  const searchRef = useRef<HTMLInputElement>(null)

  const products = useProducts({ search: search.trim() || undefined })
  const taxes = useTaxes()
  const discounts = useDiscounts()
  const checkout = useCheckout()

  const totals = computeCartTotals(cart, taxes.data ?? [], discounts.data ?? [])
  const paid = payments.reduce((sum, p) => sum + (displayToCents(p.amount) ?? 0), 0)
  const changeOrDue = paid - totals.total
  const canComplete =
    cart.length > 0 &&
    paid >= totals.total &&
    payments.every((p) => displayToCents(p.amount) !== null && displayToCents(p.amount)! > 0)

  function addProduct(product: Product) {
    setError(null)
    setCart((prev) => {
      const existing = prev.find((l) => l.product.id === product.id)
      if (existing) {
        if (existing.quantity >= product.current_stock) return prev
        return prev.map((l) =>
          l.product.id === product.id ? { ...l, quantity: l.quantity + 1 } : l,
        )
      }
      if (product.current_stock < 1) return prev
      return [...prev, { product, quantity: 1, discountOverride: null }]
    })
  }

  async function handleSearchKeyDown(e: ReactKeyboardEvent<HTMLInputElement>) {
    if (e.key !== 'Enter' || !search.trim()) return
    e.preventDefault()
    try {
      const match = await getProductByBarcode(search.trim())
      if (match) {
        addProduct(match)
        setSearch('')
        return
      }
    } catch {
      // fall through to the visible search results below
    }
  }

  function updateQuantity(productId: number, quantity: number) {
    setCart((prev) =>
      prev
        .map((l) =>
          l.product.id === productId
            ? { ...l, quantity: Math.min(Math.max(quantity, 0), l.product.current_stock) }
            : l,
        )
        .filter((l) => l.quantity > 0),
    )
  }

  function updateDiscount(productId: number, value: string) {
    const cents = value.trim() === '' ? null : displayToCents(value)
    setCart((prev) =>
      prev.map((l) => (l.product.id === productId ? { ...l, discountOverride: cents } : l)),
    )
  }

  function removeLine(productId: number) {
    setCart((prev) => prev.filter((l) => l.product.id !== productId))
  }

  function updatePayment(index: number, patch: Partial<PaymentRow>) {
    setPayments((prev) => prev.map((p, i) => (i === index ? { ...p, ...patch } : p)))
  }

  async function completeSale() {
    if (!canComplete || checkout.isPending) return
    setError(null)
    try {
      const result = await checkout.mutateAsync({
        customerId: null,
        items: cart.map((l) => ({
          productId: l.product.id,
          quantity: l.quantity,
          discountOverride: l.discountOverride,
        })),
        payments: payments.map((p) => ({ method: p.method, amount: displayToCents(p.amount)! })),
        idempotencyKey: null,
      })
      setReceipt({ sale: result.sale, change: result.change })
      setCart([])
      setPayments([{ method: 'cash', amount: '' }])
    } catch (err) {
      setError(commandErrorMessage(err))
    }
  }

  useEffect(() => {
    function onKeyDown(e: KeyboardEvent) {
      if (e.key === 'F2') {
        e.preventDefault()
        searchRef.current?.focus()
      } else if (e.key === 'F9') {
        e.preventDefault()
        void completeSale()
      } else if (e.key === 'Escape' && receipt) {
        setReceipt(null)
      }
    }
    window.addEventListener('keydown', onKeyDown)
    return () => window.removeEventListener('keydown', onKeyDown)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [canComplete, cart, payments, receipt])

  return (
    <div className="grid h-full grid-cols-3 gap-6">
      <div className="col-span-2 flex flex-col gap-4">
        <div>
          <Label htmlFor="scan">Search or scan barcode (F2)</Label>
          <Input
            id="scan"
            ref={searchRef}
            autoFocus
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            onKeyDown={(e) => void handleSearchKeyDown(e)}
            placeholder="Type a name/SKU, or scan a barcode and press Enter"
          />
        </div>

        {search.trim() && (
          <div className="flex max-h-40 flex-col gap-1 overflow-y-auto rounded-md border p-2">
            {products.data?.length === 0 && (
              <p className="text-muted-foreground text-sm">No matching products.</p>
            )}
            {products.data?.map((product) => (
              <button
                key={product.id}
                type="button"
                className="hover:bg-secondary flex items-center justify-between rounded-md px-2 py-1.5 text-left text-sm"
                onClick={() => addProduct(product)}
                disabled={product.current_stock < 1}
              >
                <span>
                  {product.name} <span className="text-muted-foreground">({product.sku})</span>
                </span>
                <span className="text-muted-foreground">
                  {centsToDisplay(product.selling_price)} · stock {product.current_stock}
                </span>
              </button>
            ))}
          </div>
        )}

        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Product</TableHead>
              <TableHead>Qty</TableHead>
              <TableHead>Price</TableHead>
              <TableHead>Discount</TableHead>
              <TableHead>Tax</TableHead>
              <TableHead>Total</TableHead>
              <TableHead />
            </TableRow>
          </TableHeader>
          <TableBody>
            {cart.length === 0 && (
              <TableRow>
                <TableCell colSpan={7} className="text-muted-foreground text-center">
                  Cart is empty. Search or scan a product to add it.
                </TableCell>
              </TableRow>
            )}
            {cart.map((line) => {
              const lineTotals = computeLineTotals(line, taxes.data ?? [], discounts.data ?? [])
              return (
                <TableRow key={line.product.id}>
                  <TableCell>{line.product.name}</TableCell>
                  <TableCell>
                    <div className="flex items-center gap-1">
                      <Button
                        variant="outline"
                        size="icon"
                        onClick={() => updateQuantity(line.product.id, line.quantity - 1)}
                      >
                        <MinusIcon />
                      </Button>
                      <span className="w-6 text-center">{line.quantity}</span>
                      <Button
                        variant="outline"
                        size="icon"
                        onClick={() => updateQuantity(line.product.id, line.quantity + 1)}
                        disabled={line.quantity >= line.product.current_stock}
                      >
                        <PlusIcon />
                      </Button>
                    </div>
                  </TableCell>
                  <TableCell>{centsToDisplay(lineTotals.unitPrice)}</TableCell>
                  <TableCell>
                    <Input
                      className="w-20"
                      inputMode="decimal"
                      placeholder={centsToDisplay(lineTotals.discount)}
                      value={
                        line.discountOverride === null ? '' : centsToDisplay(line.discountOverride)
                      }
                      onChange={(e) => updateDiscount(line.product.id, e.target.value)}
                    />
                  </TableCell>
                  <TableCell>{centsToDisplay(lineTotals.tax)}</TableCell>
                  <TableCell className="font-medium">
                    {centsToDisplay(lineTotals.lineTotal)}
                  </TableCell>
                  <TableCell>
                    <Button variant="ghost" size="icon" onClick={() => removeLine(line.product.id)}>
                      <TrashIcon />
                    </Button>
                  </TableCell>
                </TableRow>
              )
            })}
          </TableBody>
        </Table>
      </div>

      <div className="flex flex-col gap-4 rounded-lg border p-4">
        <h2 className="text-lg font-semibold">Payment</h2>

        <div className="flex flex-col gap-1 text-sm">
          <div className="flex justify-between">
            <span className="text-muted-foreground">Subtotal</span>
            <span>{centsToDisplay(totals.subtotal)}</span>
          </div>
          <div className="flex justify-between">
            <span className="text-muted-foreground">Discount</span>
            <span>-{centsToDisplay(totals.discountTotal)}</span>
          </div>
          <div className="flex justify-between">
            <span className="text-muted-foreground">Tax</span>
            <span>{centsToDisplay(totals.taxTotal)}</span>
          </div>
          <div className="flex justify-between text-base font-semibold">
            <span>Total</span>
            <span>{centsToDisplay(totals.total)}</span>
          </div>
        </div>

        <div className="flex flex-col gap-2">
          {payments.map((payment, index) => (
            <div key={index} className="flex items-center gap-2">
              <Select
                value={payment.method}
                onValueChange={(v) => updatePayment(index, { method: v })}
              >
                <SelectTrigger className="w-32">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {PAYMENT_METHODS.map((m) => (
                    <SelectItem key={m.value} value={m.value}>
                      {m.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <Input
                inputMode="decimal"
                placeholder="0.00"
                value={payment.amount}
                onChange={(e) => updatePayment(index, { amount: e.target.value })}
              />
              {payments.length > 1 && (
                <Button
                  variant="ghost"
                  size="icon"
                  onClick={() => setPayments((prev) => prev.filter((_, i) => i !== index))}
                >
                  <TrashIcon />
                </Button>
              )}
            </div>
          ))}
          <Button
            type="button"
            variant="outline"
            size="sm"
            className="w-fit"
            onClick={() => setPayments((prev) => [...prev, { method: 'cash', amount: '' }])}
          >
            <PlusIcon /> Add payment (split/mixed)
          </Button>
        </div>

        <div className="flex justify-between text-sm">
          <span className="text-muted-foreground">{changeOrDue >= 0 ? 'Change' : 'Due'}</span>
          <span className="font-medium">{centsToDisplay(Math.abs(changeOrDue))}</span>
        </div>

        {error && <p className="text-destructive text-sm">{error}</p>}

        <Button
          size="lg"
          disabled={!canComplete || checkout.isPending}
          onClick={() => void completeSale()}
        >
          {checkout.isPending ? 'Processing…' : 'Complete sale (F9)'}
        </Button>
      </div>

      <ReceiptView
        open={receipt !== null}
        onOpenChange={(open) => !open && setReceipt(null)}
        sale={receipt?.sale ?? null}
        change={receipt?.change}
      />
    </div>
  )
}
