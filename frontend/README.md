# Mario Kart Leaderboard - Frontend

Modern React frontend for the Mario Kart Leaderboard application, built with the latest technologies and best practices.

## Tech Stack

- **React 19** - Latest React with modern hooks and concurrent features
- **TypeScript 5.9** - Strictest configuration for maximum type safety
- **Vite 7** - Lightning-fast build tool with ESM support
- **urql** - Lightweight GraphQL client with caching
- **gql.tada** - Type-safe GraphQL queries with full IDE support
- **Jotai** - Atomic state management
- **Tailwind CSS 4** - Utility-first CSS with the latest features
- **shadcn/ui** - High-quality, accessible components
- **Biome** - Fast linting and formatting with all recommended rules

## Project Structure

```
frontend/
├── src/
│   ├── components/
│   │   └── ui/          # shadcn/ui components
│   ├── lib/
│   │   ├── graphql.ts   # GraphQL-tada configuration
│   │   ├── urql.ts      # urql client setup
│   │   └── utils.ts     # Utility functions (cn, etc.)
│   ├── store/           # Jotai atoms for state management
│   ├── App.tsx          # Main application component
│   ├── root.tsx         # Application entry point
│   └── index.css        # Global styles with Tailwind
├── biome.json           # Biome linting configuration
├── components.json      # shadcn/ui configuration
├── tailwind.config.js   # Tailwind CSS configuration
├── tsconfig.json        # TypeScript configuration
└── vite.config.ts       # Vite configuration
```

## Getting Started

### Prerequisites

- Node.js 22.x (LTS)
- pnpm 10.x
- Backend server running at `http://localhost:8080`

### Installation

```bash
# Install dependencies
pnpm install
```

### Generate GraphQL Schema

Before running the app, generate the GraphQL schema types:

```bash
# Make sure the backend is running first
pnpm graphql:generate
```

### Development

```bash
# Start the development server
pnpm dev
```

The app will be available at `http://localhost:5173`

### Building

```bash
# Build for production
pnpm build

# Preview production build
pnpm preview
```

### Linting & Formatting

```bash
# Check linting and formatting
pnpm lint

# Auto-fix linting and formatting issues
pnpm lint:fix

# Format code
pnpm format
```

## Code Principles

This project follows functional programming principles:

- **No mutations** - All data transformations use immutable patterns
- **Pure functions** - Components and utilities are side-effect free where possible
- **Array methods over loops** - Use `map`, `filter`, `reduce` instead of `for` loops
- **Const over let** - Variables are immutable by default
- **Type safety** - Strictest TypeScript settings enabled

## TypeScript Configuration

The project uses the strictest possible TypeScript configuration:

- `strict: true` - All strict mode checks
- `noUncheckedIndexedAccess` - Safer array/object access
- `exactOptionalPropertyTypes` - Stricter optional properties
- `noImplicitOverride` - Explicit override keyword
- `noPropertyAccessFromIndexSignature` - Consistent property access
- `noImplicitReturns` - All code paths must return
- `forceConsistentCasingInFileNames` - Consistent file naming

## Adding Components

To add shadcn/ui components:

```bash
# Example: add a button component
npx shadcn@latest add button
```

Components will be added to `src/components/ui/` and can be imported as:

```typescript
import { Button } from '@/components/ui/button'
```

## State Management with Jotai

Create atoms in `src/store/`:

```typescript
import { atom } from 'jotai'

export const userAtom = atom({ name: '', email: '' })
```

Use in components:

```typescript
import { useAtom } from 'jotai'
import { userAtom } from '@/store'

const MyComponent = () => {
  const [user, setUser] = useAtom(userAtom)
  // ...
}
```

## GraphQL with gql.tada

Define type-safe queries:

```typescript
import { graphql } from '@/lib/graphql'

const GetPlayersQuery = graphql(`
  query GetPlayers {
    players {
      id
      name
    }
  }
`)

// Full TypeScript inference!
const [result] = useQuery({ query: GetPlayersQuery })
```

## License

MIT
