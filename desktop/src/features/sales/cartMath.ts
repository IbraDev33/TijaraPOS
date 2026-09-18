import type { Discount, Product, Tax } from '@/features/products/api'

/**
 * Client-side preview only — mirrors `sales::compute_product_discount`/
 * `compute_product_tax` in `desktop/src-tauri/src/sales/mod.rs` so the
 * cart shows accurate running totals as it's built, but the backend
 * always recomputes authoritatively at checkout. The receipt renders the
 * server's response, never these numbers.
 */
export interface CartLine {
  product: Product
  quantity: number
  /** `null` means "auto-apply the product's own discount, if any". */
  discountOverride: number | null
}

export interface LineTotals {
  unitPrice: number
  lineSubtotal: number
  discount: number
  tax: number
  lineTotal: number
}

export function computeLineTotals(line: CartLine, taxes: Tax[], discounts: Discount[]): LineTotals {
  const unitPrice = line.product.selling_price
  const lineSubtotal = unitPrice * line.quantity

  let discount = 0
  if (line.discountOverride !== null) {
    discount = line.discountOverride
  } else if (line.product.discount_id !== null) {
    const matched = discounts.find((d) => d.id === line.product.discount_id)
    if (matched?.is_active) {
      discount =
        matched.kind === 'percent'
          ? Math.round((lineSubtotal * matched.value) / 100)
          : matched.value * line.quantity
    }
  }
  discount = Math.min(Math.max(discount, 0), lineSubtotal)

  let tax = 0
  if (line.product.tax_id !== null) {
    const matched = taxes.find((t) => t.id === line.product.tax_id)
    if (matched?.is_active) {
      tax = Math.round((lineSubtotal - discount) * matched.rate)
    }
  }

  return { unitPrice, lineSubtotal, discount, tax, lineTotal: lineSubtotal - discount + tax }
}

export function computeCartTotals(lines: CartLine[], taxes: Tax[], discounts: Discount[]) {
  let subtotal = 0
  let discountTotal = 0
  let taxTotal = 0
  for (const line of lines) {
    const totals = computeLineTotals(line, taxes, discounts)
    subtotal += totals.lineSubtotal
    discountTotal += totals.discount
    taxTotal += totals.tax
  }
  return { subtotal, discountTotal, taxTotal, total: subtotal - discountTotal + taxTotal }
}
