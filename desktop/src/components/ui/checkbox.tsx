import type * as React from 'react'

import { cn } from '@/lib/utils'

/**
 * A plain native checkbox rather than a Radix primitive — native
 * semantics are sufficient here and it avoids another dependency for a
 * single boolean input.
 */
function Checkbox({ className, ...props }: React.ComponentProps<'input'>) {
  return (
    <input
      type="checkbox"
      data-slot="checkbox"
      className={cn(
        'border-input text-primary size-4 shrink-0 rounded-sm border shadow-xs outline-none',
        'focus-visible:outline-ring focus-visible:outline-2 focus-visible:outline-offset-2',
        'disabled:cursor-not-allowed disabled:opacity-50',
        className,
      )}
      {...props}
    />
  )
}

export { Checkbox }
