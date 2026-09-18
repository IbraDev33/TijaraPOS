import { zodResolver } from '@hookform/resolvers/zod'
import { PlusIcon, TrashIcon } from 'lucide-react'
import { useEffect } from 'react'
import { Controller, useFieldArray, useForm } from 'react-hook-form'
import { z } from 'zod'

import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { commandErrorMessage } from '@/features/auth/api'
import type { Product, ProductInput } from '@/features/products/api'
import {
  useBrands,
  useCategories,
  useCreateProduct,
  useDiscounts,
  useTaxes,
  useUnits,
  useUpdateProduct,
} from '@/features/products/queries'
import { centsToDisplay, displayToCents } from '@/lib/money'

const NONE = 'none'
const decimalAmount = z.string().regex(/^\d+(\.\d{1,2})?$/, 'Enter a valid amount, e.g. 12.50')

const productFormSchema = z.object({
  sku: z.string().trim().min(1, 'SKU is required').max(64),
  name: z.string().trim().min(1, 'Name is required').max(200),
  description: z.string().max(2000).optional(),
  categoryId: z.string(),
  brandId: z.string(),
  unitId: z.string().min(1, 'Unit is required'),
  purchasePrice: decimalAmount,
  sellingPrice: decimalAmount,
  taxId: z.string(),
  discountId: z.string(),
  minStock: z.string().regex(/^\d+$/, 'Must be a whole number, 0 or more'),
  barcodes: z.array(z.object({ value: z.string() })),
})

type ProductFormValues = z.infer<typeof productFormSchema>

function toFormValues(product: Product | null): ProductFormValues {
  return {
    sku: product?.sku ?? '',
    name: product?.name ?? '',
    description: product?.description ?? '',
    categoryId: product?.category_id ? String(product.category_id) : NONE,
    brandId: product?.brand_id ? String(product.brand_id) : NONE,
    unitId: product?.unit_id ? String(product.unit_id) : '',
    purchasePrice: product ? centsToDisplay(product.purchase_price) : '0.00',
    sellingPrice: product ? centsToDisplay(product.selling_price) : '0.00',
    taxId: product?.tax_id ? String(product.tax_id) : NONE,
    discountId: product?.discount_id ? String(product.discount_id) : NONE,
    minStock: String(product?.min_stock ?? 0),
    barcodes: (product?.barcodes.length ? product.barcodes : ['']).map((value) => ({ value })),
  }
}

export function ProductFormDialog({
  open,
  onOpenChange,
  product,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  /** null creates a new product; otherwise edits this one. */
  product: Product | null
}) {
  const categories = useCategories()
  const brands = useBrands()
  const units = useUnits()
  const taxes = useTaxes()
  const discounts = useDiscounts()
  const createProduct = useCreateProduct()
  const updateProduct = useUpdateProduct()

  const form = useForm<ProductFormValues>({
    resolver: zodResolver(productFormSchema),
    defaultValues: toFormValues(product),
  })
  const barcodeFields = useFieldArray({ control: form.control, name: 'barcodes' })

  useEffect(() => {
    if (open) {
      form.reset(toFormValues(product))
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, product])

  const isSaving = createProduct.isPending || updateProduct.isPending

  const onSubmit = form.handleSubmit(async (values) => {
    const purchasePrice = displayToCents(values.purchasePrice)
    const sellingPrice = displayToCents(values.sellingPrice)
    if (purchasePrice === null || sellingPrice === null) {
      return
    }

    const input: ProductInput = {
      sku: values.sku,
      name: values.name,
      description: values.description?.trim() ? values.description.trim() : null,
      categoryId: values.categoryId === NONE ? null : Number(values.categoryId),
      brandId: values.brandId === NONE ? null : Number(values.brandId),
      unitId: Number(values.unitId),
      purchasePrice,
      sellingPrice,
      taxId: values.taxId === NONE ? null : Number(values.taxId),
      discountId: values.discountId === NONE ? null : Number(values.discountId),
      minStock: Number.parseInt(values.minStock, 10),
      imagePath: product?.image_path ?? null,
      barcodes: values.barcodes.map((b) => b.value.trim()).filter(Boolean),
    }

    try {
      if (product) {
        await updateProduct.mutateAsync({ id: product.id, input })
      } else {
        await createProduct.mutateAsync(input)
      }
      onOpenChange(false)
    } catch (error) {
      form.setError('root', { message: commandErrorMessage(error) })
    }
  })

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl">
        <DialogHeader>
          <DialogTitle>{product ? 'Edit product' : 'New product'}</DialogTitle>
          <DialogDescription>
            {product ? `Editing ${product.sku}` : 'Add a product to the catalog.'}
          </DialogDescription>
        </DialogHeader>

        <form className="flex flex-col gap-4" onSubmit={onSubmit} noValidate>
          <div className="grid grid-cols-2 gap-4">
            <div className="flex flex-col gap-2">
              <Label htmlFor="sku">SKU</Label>
              <Input id="sku" {...form.register('sku')} />
              {form.formState.errors.sku && (
                <p className="text-destructive text-sm">{form.formState.errors.sku.message}</p>
              )}
            </div>
            <div className="flex flex-col gap-2">
              <Label htmlFor="name">Name</Label>
              <Input id="name" {...form.register('name')} />
              {form.formState.errors.name && (
                <p className="text-destructive text-sm">{form.formState.errors.name.message}</p>
              )}
            </div>
          </div>

          <div className="flex flex-col gap-2">
            <Label htmlFor="description">Description</Label>
            <Input id="description" {...form.register('description')} />
          </div>

          <div className="grid grid-cols-3 gap-4">
            <div className="flex flex-col gap-2">
              <Label>Category</Label>
              <Controller
                control={form.control}
                name="categoryId"
                render={({ field }) => (
                  <Select value={field.value} onValueChange={field.onChange}>
                    <SelectTrigger>
                      <SelectValue placeholder="None" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value={NONE}>None</SelectItem>
                      {categories.data?.map((c) => (
                        <SelectItem key={c.id} value={String(c.id)}>
                          {c.name}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                )}
              />
            </div>
            <div className="flex flex-col gap-2">
              <Label>Brand</Label>
              <Controller
                control={form.control}
                name="brandId"
                render={({ field }) => (
                  <Select value={field.value} onValueChange={field.onChange}>
                    <SelectTrigger>
                      <SelectValue placeholder="None" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value={NONE}>None</SelectItem>
                      {brands.data?.map((b) => (
                        <SelectItem key={b.id} value={String(b.id)}>
                          {b.name}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                )}
              />
            </div>
            <div className="flex flex-col gap-2">
              <Label>Unit</Label>
              <Controller
                control={form.control}
                name="unitId"
                render={({ field }) => (
                  <Select value={field.value} onValueChange={field.onChange}>
                    <SelectTrigger>
                      <SelectValue placeholder="Select a unit" />
                    </SelectTrigger>
                    <SelectContent>
                      {units.data?.map((u) => (
                        <SelectItem key={u.id} value={String(u.id)}>
                          {u.name} ({u.abbreviation})
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                )}
              />
              {form.formState.errors.unitId && (
                <p className="text-destructive text-sm">{form.formState.errors.unitId.message}</p>
              )}
            </div>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="flex flex-col gap-2">
              <Label htmlFor="purchasePrice">Purchase price</Label>
              <Input id="purchasePrice" inputMode="decimal" {...form.register('purchasePrice')} />
              {form.formState.errors.purchasePrice && (
                <p className="text-destructive text-sm">
                  {form.formState.errors.purchasePrice.message}
                </p>
              )}
            </div>
            <div className="flex flex-col gap-2">
              <Label htmlFor="sellingPrice">Selling price</Label>
              <Input id="sellingPrice" inputMode="decimal" {...form.register('sellingPrice')} />
              {form.formState.errors.sellingPrice && (
                <p className="text-destructive text-sm">
                  {form.formState.errors.sellingPrice.message}
                </p>
              )}
            </div>
          </div>

          <div className="grid grid-cols-3 gap-4">
            <div className="flex flex-col gap-2">
              <Label>Tax</Label>
              <Controller
                control={form.control}
                name="taxId"
                render={({ field }) => (
                  <Select value={field.value} onValueChange={field.onChange}>
                    <SelectTrigger>
                      <SelectValue placeholder="None" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value={NONE}>None</SelectItem>
                      {taxes.data
                        ?.filter((t) => t.is_active)
                        .map((t) => (
                          <SelectItem key={t.id} value={String(t.id)}>
                            {t.name} ({Math.round(t.rate * 100)}%)
                          </SelectItem>
                        ))}
                    </SelectContent>
                  </Select>
                )}
              />
            </div>
            <div className="flex flex-col gap-2">
              <Label>Discount</Label>
              <Controller
                control={form.control}
                name="discountId"
                render={({ field }) => (
                  <Select value={field.value} onValueChange={field.onChange}>
                    <SelectTrigger>
                      <SelectValue placeholder="None" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value={NONE}>None</SelectItem>
                      {discounts.data
                        ?.filter((d) => d.is_active)
                        .map((d) => (
                          <SelectItem key={d.id} value={String(d.id)}>
                            {d.name}
                          </SelectItem>
                        ))}
                    </SelectContent>
                  </Select>
                )}
              />
            </div>
            <div className="flex flex-col gap-2">
              <Label htmlFor="minStock">Minimum stock</Label>
              <Input id="minStock" inputMode="numeric" {...form.register('minStock')} />
              {form.formState.errors.minStock && (
                <p className="text-destructive text-sm">{form.formState.errors.minStock.message}</p>
              )}
            </div>
          </div>

          <div className="flex flex-col gap-2">
            <Label>Barcodes</Label>
            {barcodeFields.fields.map((field, index) => (
              <div key={field.id} className="flex items-center gap-2">
                <Input
                  {...form.register(`barcodes.${index}.value` as const)}
                  placeholder="Scan or type a barcode"
                />
                <Button
                  type="button"
                  variant="ghost"
                  size="icon"
                  onClick={() => barcodeFields.remove(index)}
                  disabled={barcodeFields.fields.length === 1}
                >
                  <TrashIcon />
                </Button>
              </div>
            ))}
            <Button
              type="button"
              variant="outline"
              size="sm"
              className="w-fit"
              onClick={() => barcodeFields.append({ value: '' })}
            >
              <PlusIcon /> Add barcode
            </Button>
          </div>

          {form.formState.errors.root && (
            <p className="text-destructive text-sm">{form.formState.errors.root.message}</p>
          )}

          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              Cancel
            </Button>
            <Button type="submit" disabled={isSaving}>
              {isSaving ? 'Saving…' : product ? 'Save changes' : 'Create product'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
