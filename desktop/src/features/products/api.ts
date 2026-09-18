import { invoke } from '@tauri-apps/api/core'
import { z } from 'zod'

/**
 * Typed, Zod-validated wrappers around the catalog Tauri commands
 * (`desktop/src-tauri/src/commands/catalog.rs`). Mirrors the
 * `desktop/src/features/auth/api.ts` pattern.
 */

export const categorySchema = z.object({
  id: z.number(),
  name: z.string(),
  parent_id: z.number().nullable(),
  is_active: z.boolean(),
})
export type Category = z.infer<typeof categorySchema>

export const brandSchema = z.object({ id: z.number(), name: z.string() })
export type Brand = z.infer<typeof brandSchema>

export const unitSchema = z.object({ id: z.number(), name: z.string(), abbreviation: z.string() })
export type Unit = z.infer<typeof unitSchema>

export const taxSchema = z.object({
  id: z.number(),
  name: z.string(),
  rate: z.number(),
  is_active: z.boolean(),
})
export type Tax = z.infer<typeof taxSchema>

export const discountSchema = z.object({
  id: z.number(),
  name: z.string(),
  kind: z.enum(['percent', 'fixed']),
  value: z.number(),
  is_active: z.boolean(),
})
export type Discount = z.infer<typeof discountSchema>

export const productSchema = z.object({
  id: z.number(),
  sku: z.string(),
  name: z.string(),
  description: z.string().nullable(),
  category_id: z.number().nullable(),
  category_name: z.string().nullable(),
  brand_id: z.number().nullable(),
  brand_name: z.string().nullable(),
  unit_id: z.number(),
  unit_name: z.string(),
  unit_abbreviation: z.string(),
  purchase_price: z.number(),
  selling_price: z.number(),
  tax_id: z.number().nullable(),
  discount_id: z.number().nullable(),
  min_stock: z.number(),
  current_stock: z.number(),
  is_active: z.boolean(),
  image_path: z.string().nullable(),
  barcodes: z.array(z.string()),
  created_at: z.string(),
  updated_at: z.string(),
})
export type Product = z.infer<typeof productSchema>

export interface ProductInput {
  sku: string
  name: string
  description: string | null
  categoryId: number | null
  brandId: number | null
  unitId: number
  purchasePrice: number
  sellingPrice: number
  taxId: number | null
  discountId: number | null
  minStock: number
  imagePath: string | null
  barcodes: string[]
}

export interface ProductListQuery {
  search?: string
  categoryId?: number
  includeInactive?: boolean
}

// Categories

export async function listCategories(): Promise<Category[]> {
  return categorySchema.array().parse(await invoke('categories_list'))
}
export async function createCategory(input: { name: string; parentId: number | null }) {
  return categorySchema.parse(await invoke('categories_create', { input }))
}
export async function updateCategory(id: number, input: { name: string; parentId: number | null }) {
  return categorySchema.parse(await invoke('categories_update', { id, input }))
}
export async function deleteCategory(id: number) {
  await invoke('categories_delete', { id })
}

// Brands

export async function listBrands(): Promise<Brand[]> {
  return brandSchema.array().parse(await invoke('brands_list'))
}
export async function createBrand(input: { name: string }) {
  return brandSchema.parse(await invoke('brands_create', { input }))
}
export async function updateBrand(id: number, input: { name: string }) {
  return brandSchema.parse(await invoke('brands_update', { id, input }))
}
export async function deleteBrand(id: number) {
  await invoke('brands_delete', { id })
}

// Units

export async function listUnits(): Promise<Unit[]> {
  return unitSchema.array().parse(await invoke('units_list'))
}
export async function createUnit(input: { name: string; abbreviation: string }) {
  return unitSchema.parse(await invoke('units_create', { input }))
}
export async function updateUnit(id: number, input: { name: string; abbreviation: string }) {
  return unitSchema.parse(await invoke('units_update', { id, input }))
}
export async function deleteUnit(id: number) {
  await invoke('units_delete', { id })
}

// Taxes

export async function listTaxes(): Promise<Tax[]> {
  return taxSchema.array().parse(await invoke('taxes_list'))
}
export async function createTax(input: { name: string; rate: number }) {
  return taxSchema.parse(await invoke('taxes_create', { input }))
}
export async function updateTax(id: number, input: { name: string; rate: number }) {
  return taxSchema.parse(await invoke('taxes_update', { id, input }))
}
export async function setTaxActive(id: number, isActive: boolean) {
  await invoke('taxes_set_active', { id, isActive })
}

// Discounts

export async function listDiscounts(): Promise<Discount[]> {
  return discountSchema.array().parse(await invoke('discounts_list'))
}
export async function createDiscount(input: { name: string; kind: string; value: number }) {
  return discountSchema.parse(await invoke('discounts_create', { input }))
}
export async function updateDiscount(
  id: number,
  input: { name: string; kind: string; value: number },
) {
  return discountSchema.parse(await invoke('discounts_update', { id, input }))
}
export async function setDiscountActive(id: number, isActive: boolean) {
  await invoke('discounts_set_active', { id, isActive })
}

// Products

export async function listProducts(query: ProductListQuery = {}): Promise<Product[]> {
  return productSchema.array().parse(await invoke('products_list', { query }))
}
export async function getProduct(id: number): Promise<Product> {
  return productSchema.parse(await invoke('products_get', { id }))
}
export async function getProductByBarcode(barcode: string): Promise<Product | null> {
  const result = await invoke('products_get_by_barcode', { barcode })
  return result ? productSchema.parse(result) : null
}
export async function createProduct(input: ProductInput): Promise<Product> {
  return productSchema.parse(await invoke('products_create', { input }))
}
export async function updateProduct(id: number, input: ProductInput): Promise<Product> {
  return productSchema.parse(await invoke('products_update', { id, input }))
}
export async function setProductActive(id: number, isActive: boolean) {
  await invoke('products_set_active', { id, isActive })
}
export async function deleteProduct(id: number) {
  await invoke('products_delete', { id })
}
