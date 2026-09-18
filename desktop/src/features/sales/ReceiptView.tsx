import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import type { Sale } from '@/features/sales/api'
import { centsToDisplay } from '@/lib/money'

/**
 * A printable receipt. `#receipt-print-area` is made the only visible
 * thing on the page by the `@media print` rules in styles/globals.css —
 * that's what makes `window.print()` produce just the receipt and not
 * the rest of the app chrome.
 */
export function ReceiptView({
  open,
  onOpenChange,
  sale,
  change,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  sale: Sale | null
  change?: number
}) {
  if (!sale) return null

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-sm">
        <DialogHeader>
          <DialogTitle>Receipt</DialogTitle>
        </DialogHeader>

        <div id="receipt-print-area" className="flex flex-col gap-2 font-mono text-xs">
          <div className="text-center">
            <p className="text-sm font-semibold">TijaraPOS</p>
            <p className="text-muted-foreground">Thank you for shopping with us</p>
          </div>

          <div className="border-t border-dashed pt-2">
            <p>Invoice: {sale.invoice_number}</p>
            <p>Date: {new Date(sale.created_at).toLocaleString()}</p>
            <p>Cashier: {sale.cashier_name}</p>
          </div>

          <table className="w-full border-t border-dashed pt-2">
            <thead>
              <tr className="text-left">
                <th className="py-1 font-normal">Item</th>
                <th className="py-1 text-right font-normal">Qty</th>
                <th className="py-1 text-right font-normal">Price</th>
                <th className="py-1 text-right font-normal">Total</th>
              </tr>
            </thead>
            <tbody>
              {sale.items.map((item) => (
                <tr key={item.product_id}>
                  <td className="py-0.5">{item.product_name}</td>
                  <td className="py-0.5 text-right">{item.quantity}</td>
                  <td className="py-0.5 text-right">{centsToDisplay(item.unit_price)}</td>
                  <td className="py-0.5 text-right">{centsToDisplay(item.line_total)}</td>
                </tr>
              ))}
            </tbody>
          </table>

          <div className="flex flex-col gap-0.5 border-t border-dashed pt-2">
            <div className="flex justify-between">
              <span>Subtotal</span>
              <span>{centsToDisplay(sale.subtotal)}</span>
            </div>
            {sale.discount_total > 0 && (
              <div className="flex justify-between">
                <span>Discount</span>
                <span>-{centsToDisplay(sale.discount_total)}</span>
              </div>
            )}
            {sale.tax_total > 0 && (
              <div className="flex justify-between">
                <span>Tax</span>
                <span>{centsToDisplay(sale.tax_total)}</span>
              </div>
            )}
            <div className="flex justify-between text-sm font-semibold">
              <span>Total</span>
              <span>{centsToDisplay(sale.total)}</span>
            </div>
          </div>

          <div className="flex flex-col gap-0.5 border-t border-dashed pt-2">
            {sale.payments.map((payment, index) => (
              <div key={index} className="flex justify-between capitalize">
                <span>{payment.method.replace('_', ' ')}</span>
                <span>{centsToDisplay(payment.amount)}</span>
              </div>
            ))}
            {change !== undefined && change > 0 && (
              <div className="flex justify-between">
                <span>Change</span>
                <span>{centsToDisplay(change)}</span>
              </div>
            )}
          </div>

          {sale.status !== 'completed' && (
            <p className="text-destructive text-center text-sm font-semibold uppercase">
              {sale.status}
            </p>
          )}

          <p className="border-t border-dashed pt-2 text-center">Thank you! Please come again.</p>
        </div>

        <Button onClick={() => window.print()}>Print</Button>
      </DialogContent>
    </Dialog>
  )
}
