import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

import * as api from '@/features/inventory/api'
import type { StockAdjustmentInput } from '@/features/inventory/api'

export function useLowStock() {
  return useQuery({ queryKey: ['inventory', 'lowStock'], queryFn: api.lowStock })
}

export function useMovements(productId?: number) {
  return useQuery({
    queryKey: ['inventory', 'movements', productId ?? null],
    queryFn: () => api.listMovements(productId),
  })
}

export function useAdjustStock() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (input: StockAdjustmentInput) => api.adjustStock(input),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['inventory'] })
      queryClient.invalidateQueries({ queryKey: ['catalog', 'products'] })
    },
  })
}
