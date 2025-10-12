import { atom, useAtom } from 'jotai'
import { cn } from '@/lib/utils'

// Example atom for demonstration
const greetingAtom = atom('Mario Kart Leaderboard')

const App = () => {
  const [greeting] = useAtom(greetingAtom)

  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-slate-50 to-slate-100 dark:from-slate-950 dark:to-slate-900">
      <div className="max-w-2xl w-full p-8">
        <div
          className={cn(
            'bg-white dark:bg-slate-800 rounded-lg shadow-xl p-12',
            'border border-slate-200 dark:border-slate-700'
          )}
        >
          <h1 className="text-5xl font-bold text-center mb-4 bg-gradient-to-r from-blue-600 to-purple-600 bg-clip-text text-transparent">
            Hello World!
          </h1>
          <p className="text-2xl text-center text-slate-700 dark:text-slate-300 mb-8">{greeting}</p>

          <div className="space-y-4 text-sm text-slate-600 dark:text-slate-400">
            <div className="bg-slate-50 dark:bg-slate-900 rounded-md p-4">
              <h2 className="font-semibold text-slate-900 dark:text-slate-100 mb-2">Tech Stack:</h2>
              <ul className="space-y-1 list-disc list-inside">
                <li>React 19 with TypeScript 5.9</li>
                <li>Vite 7 (ESM)</li>
                <li>urql - GraphQL Client</li>
                <li>gql.tada - Type-safe GraphQL</li>
                <li>Jotai - Atomic State Management</li>
                <li>Tailwind CSS 4 + shadcn/ui</li>
                <li>Biome - Linting & Formatting</li>
              </ul>
            </div>

            <div className="bg-blue-50 dark:bg-blue-950 rounded-md p-4 border border-blue-200 dark:border-blue-800">
              <h3 className="font-semibold text-blue-900 dark:text-blue-100 mb-2">Next Steps:</h3>
              <ol className="space-y-1 list-decimal list-inside text-blue-800 dark:text-blue-200">
                <li>Start the backend: cd backend && cargo run</li>
                <li>Generate GraphQL schema: pnpm graphql:generate</li>
                <li>Start dev server: pnpm dev</li>
              </ol>
            </div>

            <div className="text-center text-xs text-slate-500 dark:text-slate-500 pt-4">
              Built with functional principles • Strictly typed • Modern ESM
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

export default App
