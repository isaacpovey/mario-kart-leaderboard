import { Field, Input } from '@chakra-ui/react'
import type { ReactNode } from 'react'

type FormFieldProps = {
  label: string
  value: string | number
  onChange: (value: string) => void
  type?: string
  min?: number
  max?: number
  placeholder?: string
  disabled?: boolean
  children?: ReactNode
}

export const FormField = ({ label, value, onChange, type = 'text', min, max, placeholder, disabled, children }: FormFieldProps) => (
  <Field.Root>
    <Field.Label>{label}</Field.Label>
    {children || <Input type={type} value={value} onChange={(e) => onChange(e.target.value)} min={min} max={max} placeholder={placeholder} disabled={disabled} />}
  </Field.Root>
)
