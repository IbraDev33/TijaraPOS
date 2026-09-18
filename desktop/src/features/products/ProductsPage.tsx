import { PencilIcon, PlusIcon, Settings2Icon, TrashIcon } from 'lucide-react'
import { useState } from 'react'

import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { RequirePermission } from '@/features/auth/RequirePermission'
import type { Product } from '@/features/products/api'
import { CatalogSettingsDialog } from '@/features/products/CatalogSettingsDialog'
import { ProductFormDialog } from '@/features/products/ProductFormDialog'
import {
  useCategories,
  useDeleteProduct,
  useProducts,
  useSetProductActive,
} from '@/features/products/queries'
import { centsToDisplay } from '@/lib/money'
import { PERMISSIONS } from '@/lib/permissions'

const ALL_CATEGORIES = 'all'

export function ProductsPage() {
  const [search, setSearch] = useState('')
  const [categoryId, setCategoryId] = useState(ALL_CATEGORIES)
  const [includeInactive, setIncludeInactive] = useState(false)
  const [formOpen, setFormOpen] = useState(false)
  const [settingsOpen, setSettingsOpen] = useState(false)
  const [editingProduct, setEditingProduct] = useState<Product | null>(null)

  const categories = useCategories()
  const products = useProducts({
    search: search.trim() || undefined,
    categoryId: categoryId === ALL_CATEGORIES ? undefined : Number(categoryId),
    includeInactive,
  })
  const setProductActive = useSetProductActive()
  const deleteProduct = useDeleteProduct()

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center justify-between gap-4">
        <h1 className="text-2xl font-semibold">Products</h1>
        <div className="flex items-center gap-2">
          <RequirePermission permission={PERMISSIONS.ProductsView}>
            <Button variant="outline" onClick={() => setSettingsOpen(true)}>
              <Settings2Icon /> Categories, brands &amp; more
            </Button>
          </RequirePermission>
          <RequirePermission permission={PERMISSIONS.ProductsCreate}>
            <Button
              onClick={() => {
                setEditingProduct(null)
                setFormOpen(true)
              }}
            >
              <PlusIcon /> New product
            </Button>
          </RequirePermission>
        </div>
      </div>

      <div className="flex flex-wrap items-end gap-4">
        <div className="flex min-w-48 flex-col gap-2">
          <Label htmlFor="search">Search</Label>
          <Input
            id="search"
            placeholder="SKU or name…"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
        </div>
        <div className="flex min-w-48 flex-col gap-2">
          <Label>Category</Label>
          <Select value={categoryId} onValueChange={setCategoryId}>
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value={ALL_CATEGORIES}>All categories</SelectItem>
              {categories.data?.map((c) => (
                <SelectItem key={c.id} value={String(c.id)}>
                  {c.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        <div className="flex items-center gap-2 pb-2">
          <Checkbox
            id="includeInactive"
            checked={includeInactive}
            onChange={(e) => setIncludeInactive(e.target.checked)}
          />
          <Label htmlFor="includeInactive">Show inactive</Label>
        </div>
      </div>

      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>SKU</TableHead>
            <TableHead>Name</TableHead>
            <TableHead>Category</TableHead>
            <TableHead>Brand</TableHead>
            <TableHead>Price</TableHead>
            <TableHead>Stock</TableHead>
            <TableHead>Active</TableHead>
            <TableHead />
          </TableRow>
        </TableHeader>
        <TableBody>
          {products.isLoading && (
            <TableRow>
              <TableCell colSpan={8} className="text-muted-foreground text-center">
                Loading…
              </TableCell>
            </TableRow>
          )}
          {products.data?.length === 0 && (
            <TableRow>
              <TableCell colSpan={8} className="text-muted-foreground text-center">
                No products found.
              </TableCell>
            </TableRow>
          )}
          {products.data?.map((product) => (
            <TableRow key={product.id}>
              <TableCell className="font-mono text-xs">{product.sku}</TableCell>
              <TableCell>{product.name}</TableCell>
              <TableCell>{product.category_name ?? '—'}</TableCell>
              <TableCell>{product.brand_name ?? '—'}</TableCell>
              <TableCell>{centsToDisplay(product.selling_price)}</TableCell>
              <TableCell>
                {product.current_stock}
                {product.current_stock <= product.min_stock && (
                  <span className="text-destructive ml-1 text-xs">low</span>
                )}
              </TableCell>
              <TableCell>
                <RequirePermission
                  permission={PERMISSIONS.ProductsUpdate}
                  fallback={product.is_active ? 'Yes' : 'No'}
                >
                  <Checkbox
                    checked={product.is_active}
                    onChange={(e) =>
                      setProductActive.mutate({ id: product.id, isActive: e.target.checked })
                    }
                  />
                </RequirePermission>
              </TableCell>
              <TableCell>
                <div className="flex justify-end gap-1">
                  <RequirePermission permission={PERMISSIONS.ProductsUpdate}>
                    <Button
                      variant="ghost"
                      size="icon"
                      onClick={() => {
                        setEditingProduct(product)
                        setFormOpen(true)
                      }}
                    >
                      <PencilIcon />
                    </Button>
                  </RequirePermission>
                  <RequirePermission permission={PERMISSIONS.ProductsDelete}>
                    <Button
                      variant="ghost"
                      size="icon"
                      onClick={() => {
                        if (confirm(`Delete ${product.name}? This cannot be undone.`)) {
                          deleteProduct.mutate(product.id)
                        }
                      }}
                    >
                      <TrashIcon />
                    </Button>
                  </RequirePermission>
                </div>
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>

      <ProductFormDialog open={formOpen} onOpenChange={setFormOpen} product={editingProduct} />
      <CatalogSettingsDialog open={settingsOpen} onOpenChange={setSettingsOpen} />
    </div>
  )
}
