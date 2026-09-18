import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

import * as api from '@/features/sales/api'
import type { CheckoutInput } from '@/features/sales/api'

export function useSales(status?: string) {
  return useQuery({
    queryKey: ['sales', 'list', status ?? null],
    queryFn: () => api.listSales(status ? { status } : {}),
  })
}

export function useSale(id: string | null) {
  return useQuery({
    queryKey: ['sales', 'detail', id],
    queryFn: () => api.getSale(id as string),
    enabled: id !== null,
  })
}

export function useCheckout() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (input: CheckoutInput) => api.checkout(input),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['sales'] })
      queryClient.invalidateQueries({ queryKey: ['catalog', 'products'] })
    },
  })
}

export function useCancelSale() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (id: string) => api.cancelSale(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['sales'] })
      queryClient.invalidateQueries({ queryKey: ['catalog', 'products'] })
    },
  })
}
