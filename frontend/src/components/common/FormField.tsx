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
    <Field.Label fontSize={{ base: 'sm', md: 'md' }} fontWeight="medium" mb={2}>
      {label}
    </Field.Label>
    {children || (
      <Input
        type={type}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        min={min}
        max={max}
        placeholder={placeholder}
        disabled={disabled}
        size={{ base: 'md', md: 'lg' }}
        borderRadius="button"
        borderWidth="2px"
        _focus={{ borderColor: 'brand.400', boxShadow: '0 0 0 1px var(--chakra-colors-brand-400)' }}
      />
    )}
  </Field.Root>
)
