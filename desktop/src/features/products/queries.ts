import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

import * as api from '@/features/products/api'
import type { ProductInput, ProductListQuery } from '@/features/products/api'

export const catalogKeys = {
  categories: ['catalog', 'categories'] as const,
  brands: ['catalog', 'brands'] as const,
  units: ['catalog', 'units'] as const,
  taxes: ['catalog', 'taxes'] as const,
  discounts: ['catalog', 'discounts'] as const,
  products: (query: ProductListQuery) => ['catalog', 'products', query] as const,
}

export function useCategories() {
  return useQuery({ queryKey: catalogKeys.categories, queryFn: api.listCategories })
}
export function useBrands() {
  return useQuery({ queryKey: catalogKeys.brands, queryFn: api.listBrands })
}
export function useUnits() {
  return useQuery({ queryKey: catalogKeys.units, queryFn: api.listUnits })
}
export function useTaxes() {
  return useQuery({ queryKey: catalogKeys.taxes, queryFn: api.listTaxes })
}
export function useDiscounts() {
  return useQuery({ queryKey: catalogKeys.discounts, queryFn: api.listDiscounts })
}

export function useProducts(query: ProductListQuery) {
  return useQuery({ queryKey: catalogKeys.products(query), queryFn: () => api.listProducts(query) })
}

function useInvalidateProducts() {
  const queryClient = useQueryClient()
  return () => queryClient.invalidateQueries({ queryKey: ['catalog', 'products'] })
}

export function useCreateProduct() {
  const invalidate = useInvalidateProducts()
  return useMutation({
    mutationFn: (input: ProductInput) => api.createProduct(input),
    onSuccess: invalidate,
  })
}

export function useUpdateProduct() {
  const invalidate = useInvalidateProducts()
  return useMutation({
    mutationFn: ({ id, input }: { id: number; input: ProductInput }) =>
      api.updateProduct(id, input),
    onSuccess: invalidate,
  })
}

export function useSetProductActive() {
  const invalidate = useInvalidateProducts()
  return useMutation({
    mutationFn: ({ id, isActive }: { id: number; isActive: boolean }) =>
      api.setProductActive(id, isActive),
    onSuccess: invalidate,
  })
}

export function useDeleteProduct() {
  const invalidate = useInvalidateProducts()
  return useMutation({
    mutationFn: (id: number) => api.deleteProduct(id),
    onSuccess: invalidate,
  })
}

// Catalog settings (categories/brands/units/taxes/discounts) mutations
// all follow the same shape: mutate, then invalidate that entity's list
// (and products, since product rows denormalize category/brand/unit names).

function useInvalidate(key: readonly unknown[]) {
  const queryClient = useQueryClient()
  return () => {
    queryClient.invalidateQueries({ queryKey: key })
    queryClient.invalidateQueries({ queryKey: ['catalog', 'products'] })
  }
}

export function useCreateCategory() {
  const invalidate = useInvalidate(catalogKeys.categories)
  return useMutation({
    mutationFn: (input: { name: string; parentId: number | null }) => api.createCategory(input),
    onSuccess: invalidate,
  })
}
export function useUpdateCategory() {
  const invalidate = useInvalidate(catalogKeys.categories)
  return useMutation({
    mutationFn: ({ id, input }: { id: number; input: { name: string; parentId: number | null } }) =>
      api.updateCategory(id, input),
    onSuccess: invalidate,
  })
}
export function useDeleteCategory() {
  const invalidate = useInvalidate(catalogKeys.categories)
  return useMutation({ mutationFn: (id: number) => api.deleteCategory(id), onSuccess: invalidate })
}

export function useCreateBrand() {
  const invalidate = useInvalidate(catalogKeys.brands)
  return useMutation({
    mutationFn: (input: { name: string }) => api.createBrand(input),
    onSuccess: invalidate,
  })
}
export function useUpdateBrand() {
  const invalidate = useInvalidate(catalogKeys.brands)
  return useMutation({
    mutationFn: ({ id, input }: { id: number; input: { name: string } }) =>
      api.updateBrand(id, input),
    onSuccess: invalidate,
  })
}
export function useDeleteBrand() {
  const invalidate = useInvalidate(catalogKeys.brands)
  return useMutation({ mutationFn: (id: number) => api.deleteBrand(id), onSuccess: invalidate })
}

export function useCreateUnit() {
  const invalidate = useInvalidate(catalogKeys.units)
  return useMutation({
    mutationFn: (input: { name: string; abbreviation: string }) => api.createUnit(input),
    onSuccess: invalidate,
  })
}
export function useUpdateUnit() {
  const invalidate = useInvalidate(catalogKeys.units)
  return useMutation({
    mutationFn: ({ id, input }: { id: number; input: { name: string; abbreviation: string } }) =>
      api.updateUnit(id, input),
    onSuccess: invalidate,
  })
}
export function useDeleteUnit() {
  const invalidate = useInvalidate(catalogKeys.units)
  return useMutation({ mutationFn: (id: number) => api.deleteUnit(id), onSuccess: invalidate })
}

export function useCreateTax() {
  const invalidate = useInvalidate(catalogKeys.taxes)
  return useMutation({
    mutationFn: (input: { name: string; rate: number }) => api.createTax(input),
    onSuccess: invalidate,
  })
}
export function useSetTaxActive() {
  const invalidate = useInvalidate(catalogKeys.taxes)
  return useMutation({
    mutationFn: ({ id, isActive }: { id: number; isActive: boolean }) =>
      api.setTaxActive(id, isActive),
    onSuccess: invalidate,
  })
}

export function useCreateDiscount() {
  const invalidate = useInvalidate(catalogKeys.discounts)
  return useMutation({
    mutationFn: (input: { name: string; kind: string; value: number }) => api.createDiscount(input),
    onSuccess: invalidate,
  })
}
export function useSetDiscountActive() {
  const invalidate = useInvalidate(catalogKeys.discounts)
  return useMutation({
    mutationFn: ({ id, isActive }: { id: number; isActive: boolean }) =>
      api.setDiscountActive(id, isActive),
    onSuccess: invalidate,
  })
}
