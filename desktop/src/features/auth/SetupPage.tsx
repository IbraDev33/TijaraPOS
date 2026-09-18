import { zodResolver } from '@hookform/resolvers/zod'
import { useState } from 'react'
import { useForm } from 'react-hook-form'
import { z } from 'zod'

import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { commandErrorMessage } from '@/features/auth/api'
import { useAuth } from '@/features/auth/AuthProvider'

const setupSchema = z
  .object({
    fullName: z.string().min(1, 'Full name is required').max(100),
    username: z
      .string()
      .min(3, 'Username must be at least 3 characters')
      .max(50)
      .regex(/^[a-zA-Z0-9._-]+$/, "Only letters, numbers, '.', '_', and '-' are allowed"),
    password: z.string().min(8, 'Password must be at least 8 characters'),
    confirmPassword: z.string(),
  })
  .refine((data) => data.password === data.confirmPassword, {
    message: "Passwords don't match",
    path: ['confirmPassword'],
  })

type SetupValues = z.infer<typeof setupSchema>

/**
 * Shown once, before any user account exists. There is no seeded default
 * admin credential anywhere in this app — this form is how the first
 * account gets created.
 */
export function SetupPage() {
  const { bootstrapAdmin } = useAuth()
  const [formError, setFormError] = useState<string | null>(null)

  const form = useForm<SetupValues>({
    resolver: zodResolver(setupSchema),
    defaultValues: { fullName: '', username: '', password: '', confirmPassword: '' },
  })

  const onSubmit = form.handleSubmit(async (values) => {
    setFormError(null)
    try {
      await bootstrapAdmin(values)
    } catch (error) {
      setFormError(commandErrorMessage(error))
    }
  })

  return (
    <div className="bg-background flex min-h-svh items-center justify-center p-4">
      <Card className="w-full max-w-sm">
        <CardHeader>
          <CardTitle>Welcome to TijaraPOS</CardTitle>
          <CardDescription>Create the first administrator account to get started.</CardDescription>
        </CardHeader>
        <CardContent>
          <form className="flex flex-col gap-4" onSubmit={onSubmit} noValidate>
            <div className="flex flex-col gap-2">
              <Label htmlFor="fullName">Full name</Label>
              <Input id="fullName" autoFocus autoComplete="name" {...form.register('fullName')} />
              {form.formState.errors.fullName && (
                <p className="text-destructive text-sm">{form.formState.errors.fullName.message}</p>
              )}
            </div>
            <div className="flex flex-col gap-2">
              <Label htmlFor="username">Username</Label>
              <Input id="username" autoComplete="username" {...form.register('username')} />
              {form.formState.errors.username && (
                <p className="text-destructive text-sm">{form.formState.errors.username.message}</p>
              )}
            </div>
            <div className="flex flex-col gap-2">
              <Label htmlFor="password">Password</Label>
              <Input
                id="password"
                type="password"
                autoComplete="new-password"
                {...form.register('password')}
              />
              {form.formState.errors.password && (
                <p className="text-destructive text-sm">{form.formState.errors.password.message}</p>
              )}
            </div>
            <div className="flex flex-col gap-2">
              <Label htmlFor="confirmPassword">Confirm password</Label>
              <Input
                id="confirmPassword"
                type="password"
                autoComplete="new-password"
                {...form.register('confirmPassword')}
              />
              {form.formState.errors.confirmPassword && (
                <p className="text-destructive text-sm">
                  {form.formState.errors.confirmPassword.message}
                </p>
              )}
            </div>
            {formError && <p className="text-destructive text-sm">{formError}</p>}
            <Button type="submit" disabled={form.formState.isSubmitting}>
              {form.formState.isSubmitting ? 'Creating account…' : 'Create administrator account'}
            </Button>
          </form>
        </CardContent>
      </Card>
    </div>
  )
}
