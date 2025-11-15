import { createSystem, defaultConfig, defineConfig } from '@chakra-ui/react'

const customConfig = defineConfig({
  theme: {
    tokens: {
      colors: {
        brand: {
          50: { value: '#fffbeb' },
          100: { value: '#fef3c7' },
          200: { value: '#fde68a' },
          300: { value: '#fcd34d' },
          400: { value: '#fbbf24' },
          500: { value: '#f59e0b' },
          600: { value: '#d97706' },
          700: { value: '#b45309' },
          800: { value: '#92400e' },
          900: { value: '#78350f' },
        },
      },
      fonts: {
        heading: { value: 'system-ui, sans-serif' },
        body: { value: 'system-ui, sans-serif' },
      },
      radii: {
        card: { value: '12px' },
        button: { value: '8px' },
      },
      shadows: {
        card: { value: '0 2px 8px rgba(0, 0, 0, 0.1)' },
        'card-hover': { value: '0 4px 12px rgba(0, 0, 0, 0.15)' },
      },
    },
    semanticTokens: {
      colors: {
        primary: {
          DEFAULT: { value: '{colors.brand.500}' },
          emphasized: { value: '{colors.brand.600}' },
          fg: { value: 'white' },
        },
        'bg.canvas': {
          DEFAULT: { value: '#f7f8fa' },
        },
        'bg.panel': {
          DEFAULT: { value: 'white' },
        },
      },
    },
  },
})

export const system = createSystem(defaultConfig, customConfig)
