import { invoke } from '@tauri-apps/api/core'
import { z } from 'zod'

/**
 * Typed, Zod-validated wrappers around the inventory Tauri commands
 * (`desktop/src-tauri/src/commands/inventory.rs`).
 */

export const STOCK_ADJUSTMENT_REASONS = ['adjustment', 'return', 'damage', 'transfer'] as const
export type StockAdjustmentReason = (typeof STOCK_ADJUSTMENT_REASONS)[number]

export const movementSchema = z.object({
  id: z.number(),
  product_id: z.number(),
  product_name: z.string(),
  quantity_delta: z.number(),
  reason: z.string(),
  reference_type: z.string().nullable(),
  reference_id: z.string().nullable(),
  user_name: z.string().nullable(),
  created_at: z.string(),
})
export type Movement = z.infer<typeof movementSchema>

export const lowStockProductSchema = z.object({
  id: z.number(),
  sku: z.string(),
  name: z.string(),
  current_stock: z.number(),
  min_stock: z.number(),
})
export type LowStockProduct = z.infer<typeof lowStockProductSchema>

export const adjustmentResultSchema = z.object({
  productId: z.number(),
  previousStock: z.number(),
  currentStock: z.number(),
})
export type AdjustmentResult = z.infer<typeof adjustmentResultSchema>

export interface StockAdjustmentInput {
  productId: number
  quantityDelta: number
  reason: StockAdjustmentReason
  note: string | null
}

export async function adjustStock(input: StockAdjustmentInput): Promise<AdjustmentResult> {
  return adjustmentResultSchema.parse(await invoke('inventory_adjust_stock', { input }))
}

export async function listMovements(productId?: number): Promise<Movement[]> {
  return movementSchema
    .array()
    .parse(await invoke('inventory_list_movements', { query: { productId } }))
}

export async function lowStock(): Promise<LowStockProduct[]> {
  return lowStockProductSchema.array().parse(await invoke('inventory_low_stock'))
}
