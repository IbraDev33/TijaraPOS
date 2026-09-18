/**
 * Money is stored as integer minor units (cents) end to end — see
 * `docs/database.md`. These helpers are the only place that converts
 * to/from the decimal string a human types into a form field, so
 * rounding behavior lives in one spot.
 */

export function centsToDisplay(cents: number): string {
  return (cents / 100).toFixed(2)
}

/** Returns null if the string isn't a valid non-negative amount. */
export function displayToCents(value: string): number | null {
  const trimmed = value.trim()
  if (!/^\d+(\.\d{1,2})?$/.test(trimmed)) {
    return null
  }
  return Math.round(Number.parseFloat(trimmed) * 100)
}
