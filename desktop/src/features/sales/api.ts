import { invoke } from '@tauri-apps/api/core'
import { z } from 'zod'

/**
 * Typed, Zod-validated wrappers around the sales Tauri commands
 * (`desktop/src-tauri/src/commands/sales.rs`). Mirrors the
 * `desktop/src/features/products/api.ts` pattern.
 */

export const saleItemSchema = z.object({
  product_id: z.number(),
  product_name: z.string(),
  quantity: z.number(),
  unit_price: z.number(),
  discount: z.number(),
  tax: z.number(),
  line_total: z.number(),
})
export type SaleItem = z.infer<typeof saleItemSchema>

export const paymentSchema = z.object({
  method: z.string(),
  amount: z.number(),
  received_at: z.string(),
})
export type Payment = z.infer<typeof paymentSchema>

export const saleSchema = z.object({
  id: z.string(),
  invoice_number: z.string(),
  customer_id: z.number().nullable(),
  cashier_name: z.string(),
  subtotal: z.number(),
  discount_total: z.number(),
  tax_total: z.number(),
  total: z.number(),
  status: z.enum(['completed', 'cancelled', 'refunded']),
  created_at: z.string(),
  items: z.array(saleItemSchema),
  payments: z.array(paymentSchema),
})
export type Sale = z.infer<typeof saleSchema>

export const saleSummarySchema = z.object({
  id: z.string(),
  invoice_number: z.string(),
  cashier_name: z.string(),
  total: z.number(),
  status: z.enum(['completed', 'cancelled', 'refunded']),
  created_at: z.string(),
})
export type SaleSummary = z.infer<typeof saleSummarySchema>

export const checkoutResultSchema = z.object({
  sale: saleSchema,
  change: z.number(),
})
export type CheckoutResult = z.infer<typeof checkoutResultSchema>

export interface CartLineInput {
  productId: number
  quantity: number
  discountOverride: number | null
}

export interface PaymentInput {
  method: string
  amount: number
}

export interface CheckoutInput {
  customerId: number | null
  items: CartLineInput[]
  payments: PaymentInput[]
  idempotencyKey: string | null
}

export async function checkout(input: CheckoutInput): Promise<CheckoutResult> {
  return checkoutResultSchema.parse(await invoke('sales_checkout', { input }))
}

export async function listSales(query: { status?: string } = {}): Promise<SaleSummary[]> {
  return saleSummarySchema.array().parse(await invoke('sales_list', { query }))
}

export async function getSale(id: string): Promise<Sale> {
  return saleSchema.parse(await invoke('sales_get', { id }))
}

export async function cancelSale(id: string): Promise<Sale> {
  return saleSchema.parse(await invoke('sales_cancel', { id }))
}
