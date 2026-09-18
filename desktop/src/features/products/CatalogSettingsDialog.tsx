import { TrashIcon } from 'lucide-react'
import { type FormEvent, useState } from 'react'

import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { RequirePermission } from '@/features/auth/RequirePermission'
import { commandErrorMessage } from '@/features/auth/api'
import {
  useBrands,
  useCategories,
  useCreateBrand,
  useCreateCategory,
  useCreateDiscount,
  useCreateTax,
  useCreateUnit,
  useDeleteBrand,
  useDeleteCategory,
  useDeleteUnit,
  useDiscounts,
  useSetDiscountActive,
  useSetTaxActive,
  useTaxes,
  useUnits,
} from '@/features/products/queries'
import { PERMISSIONS } from '@/lib/permissions'

const SECTIONS = ['categories', 'brands', 'units', 'taxes', 'discounts'] as const
type Section = (typeof SECTIONS)[number]

export function CatalogSettingsDialog({
  open,
  onOpenChange,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
}) {
  const [section, setSection] = useState<Section>('categories')

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-xl">
        <DialogHeader>
          <DialogTitle>Categories, brands &amp; more</DialogTitle>
          <DialogDescription>
            Manage the categories, brands, units, taxes, and discounts products can reference.
          </DialogDescription>
        </DialogHeader>
        <div className="flex flex-wrap gap-1 border-b pb-3">
          {SECTIONS.map((s) => (
            <Button
              key={s}
              type="button"
              variant={section === s ? 'secondary' : 'ghost'}
              size="sm"
              onClick={() => setSection(s)}
            >
              {s[0].toUpperCase() + s.slice(1)}
            </Button>
          ))}
        </div>
        <div className="max-h-96 overflow-y-auto">
          {section === 'categories' && <CategoriesSection />}
          {section === 'brands' && <BrandsSection />}
          {section === 'units' && <UnitsSection />}
          {section === 'taxes' && <TaxesSection />}
          {section === 'discounts' && <DiscountsSection />}
        </div>
      </DialogContent>
    </Dialog>
  )
}

function SectionError({ message }: { message: string | null }) {
  if (!message) return null
  return <p className="text-destructive text-sm">{message}</p>
}

function EntityRow({
  label,
  onDelete,
  canDelete,
}: {
  label: string
  onDelete?: () => void
  canDelete: boolean
}) {
  return (
    <li className="flex items-center justify-between gap-2 rounded-md border px-3 py-2 text-sm">
      <span>{label}</span>
      {canDelete && onDelete && (
        <Button type="button" variant="ghost" size="icon" onClick={onDelete}>
          <TrashIcon className="size-4" />
        </Button>
      )}
    </li>
  )
}

function CategoriesSection() {
  const categories = useCategories()
  const createCategory = useCreateCategory()
  const deleteCategory = useDeleteCategory()
  const [name, setName] = useState('')
  const [error, setError] = useState<string | null>(null)

  const onSubmit = async (e: FormEvent) => {
    e.preventDefault()
    setError(null)
    try {
      await createCategory.mutateAsync({ name, parentId: null })
      setName('')
    } catch (err) {
      setError(commandErrorMessage(err))
    }
  }

  return (
    <div className="flex flex-col gap-3">
      <RequirePermission permission={PERMISSIONS.ProductsCreate}>
        <form className="flex gap-2" onSubmit={onSubmit}>
          <Input
            placeholder="New category name"
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
          <Button type="submit" disabled={!name.trim() || createCategory.isPending}>
            Add
          </Button>
        </form>
      </RequirePermission>
      <SectionError message={error} />
      <ul className="flex flex-col gap-1">
        {categories.data?.map((c) => (
          <EntityRow
            key={c.id}
            label={c.name}
            canDelete
            onDelete={() => deleteCategory.mutate(c.id)}
          />
        ))}
      </ul>
    </div>
  )
}

function BrandsSection() {
  const brands = useBrands()
  const createBrand = useCreateBrand()
  const deleteBrand = useDeleteBrand()
  const [name, setName] = useState('')
  const [error, setError] = useState<string | null>(null)

  const onSubmit = async (e: FormEvent) => {
    e.preventDefault()
    setError(null)
    try {
      await createBrand.mutateAsync({ name })
      setName('')
    } catch (err) {
      setError(commandErrorMessage(err))
    }
  }

  return (
    <div className="flex flex-col gap-3">
      <RequirePermission permission={PERMISSIONS.ProductsCreate}>
        <form className="flex gap-2" onSubmit={onSubmit}>
          <Input
            placeholder="New brand name"
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
          <Button type="submit" disabled={!name.trim() || createBrand.isPending}>
            Add
          </Button>
        </form>
      </RequirePermission>
      <SectionError message={error} />
      <ul className="flex flex-col gap-1">
        {brands.data?.map((b) => (
          <EntityRow
            key={b.id}
            label={b.name}
            canDelete
            onDelete={() => deleteBrand.mutate(b.id)}
          />
        ))}
      </ul>
    </div>
  )
}

function UnitsSection() {
  const units = useUnits()
  const createUnit = useCreateUnit()
  const deleteUnit = useDeleteUnit()
  const [name, setName] = useState('')
  const [abbreviation, setAbbreviation] = useState('')
  const [error, setError] = useState<string | null>(null)

  const onSubmit = async (e: FormEvent) => {
    e.preventDefault()
    setError(null)
    try {
      await createUnit.mutateAsync({ name, abbreviation })
      setName('')
      setAbbreviation('')
    } catch (err) {
      setError(commandErrorMessage(err))
    }
  }

  return (
    <div className="flex flex-col gap-3">
      <RequirePermission permission={PERMISSIONS.ProductsCreate}>
        <form className="flex gap-2" onSubmit={onSubmit}>
          <Input
            placeholder="Name (e.g. Piece)"
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
          <Input
            placeholder="Abbr. (e.g. pc)"
            className="max-w-28"
            value={abbreviation}
            onChange={(e) => setAbbreviation(e.target.value)}
          />
          <Button
            type="submit"
            disabled={!name.trim() || !abbreviation.trim() || createUnit.isPending}
          >
            Add
          </Button>
        </form>
      </RequirePermission>
      <SectionError message={error} />
      <ul className="flex flex-col gap-1">
        {units.data?.map((u) => (
          <EntityRow
            key={u.id}
            label={`${u.name} (${u.abbreviation})`}
            canDelete
            onDelete={() => deleteUnit.mutate(u.id)}
          />
        ))}
      </ul>
    </div>
  )
}

function TaxesSection() {
  const taxes = useTaxes()
  const createTax = useCreateTax()
  const setTaxActive = useSetTaxActive()
  const [name, setName] = useState('')
  const [ratePercent, setRatePercent] = useState('')
  const [error, setError] = useState<string | null>(null)

  const onSubmit = async (e: FormEvent) => {
    e.preventDefault()
    setError(null)
    const rate = Number.parseFloat(ratePercent) / 100
    if (Number.isNaN(rate) || rate < 0 || rate > 1) {
      setError('Enter a rate between 0 and 100')
      return
    }
    try {
      await createTax.mutateAsync({ name, rate })
      setName('')
      setRatePercent('')
    } catch (err) {
      setError(commandErrorMessage(err))
    }
  }

  return (
    <div className="flex flex-col gap-3">
      <RequirePermission permission={PERMISSIONS.ProductsCreate}>
        <form className="flex gap-2" onSubmit={onSubmit}>
          <Input
            placeholder="Name (e.g. VAT)"
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
          <Input
            placeholder="Rate %"
            className="max-w-24"
            inputMode="decimal"
            value={ratePercent}
            onChange={(e) => setRatePercent(e.target.value)}
          />
          <Button
            type="submit"
            disabled={!name.trim() || !ratePercent.trim() || createTax.isPending}
          >
            Add
          </Button>
        </form>
      </RequirePermission>
      <SectionError message={error} />
      <ul className="flex flex-col gap-1">
        {taxes.data?.map((t) => (
          <li
            key={t.id}
            className="flex items-center justify-between gap-2 rounded-md border px-3 py-2 text-sm"
          >
            <span>
              {t.name} ({Math.round(t.rate * 100)}%)
            </span>
            <RequirePermission permission={PERMISSIONS.ProductsUpdate}>
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={() => setTaxActive.mutate({ id: t.id, isActive: !t.is_active })}
              >
                {t.is_active ? 'Deactivate' : 'Activate'}
              </Button>
            </RequirePermission>
          </li>
        ))}
      </ul>
    </div>
  )
}

function DiscountsSection() {
  const discounts = useDiscounts()
  const createDiscount = useCreateDiscount()
  const setDiscountActive = useSetDiscountActive()
  const [name, setName] = useState('')
  const [value, setValue] = useState('')
  const [error, setError] = useState<string | null>(null)

  const onSubmit = async (e: FormEvent) => {
    e.preventDefault()
    setError(null)
    const parsedValue = Number.parseInt(value, 10)
    if (Number.isNaN(parsedValue) || parsedValue < 0) {
      setError('Enter a non-negative whole number')
      return
    }
    try {
      await createDiscount.mutateAsync({ name, kind: 'percent', value: parsedValue })
      setName('')
      setValue('')
    } catch (err) {
      setError(commandErrorMessage(err))
    }
  }

  return (
    <div className="flex flex-col gap-3">
      <RequirePermission permission={PERMISSIONS.ProductsCreate}>
        <form className="flex gap-2" onSubmit={onSubmit}>
          <Input
            placeholder="Name (e.g. Clearance)"
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
          <Input
            placeholder="Percent off"
            className="max-w-28"
            inputMode="numeric"
            value={value}
            onChange={(e) => setValue(e.target.value)}
          />
          <Button
            type="submit"
            disabled={!name.trim() || !value.trim() || createDiscount.isPending}
          >
            Add
          </Button>
        </form>
      </RequirePermission>
      <SectionError message={error} />
      <ul className="flex flex-col gap-1">
        {discounts.data?.map((d) => (
          <li
            key={d.id}
            className="flex items-center justify-between gap-2 rounded-md border px-3 py-2 text-sm"
          >
            <span>
              {d.name} ({d.value}
              {d.kind === 'percent' ? '%' : ' minor units'} off)
            </span>
            <RequirePermission permission={PERMISSIONS.ProductsUpdate}>
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={() => setDiscountActive.mutate({ id: d.id, isActive: !d.is_active })}
              >
                {d.is_active ? 'Deactivate' : 'Activate'}
              </Button>
            </RequirePermission>
          </li>
        ))}
      </ul>
    </div>
  )
}
