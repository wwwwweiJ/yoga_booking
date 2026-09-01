import React from 'react'
import ReactDOM from 'react-dom/client'
import './index.css'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { RouterProvider } from 'react-router'
import { I18nProvider } from './i18n'
import { router } from './routes'

const root = document.getElementById('root')

if (!root) {
  throw new Error('No root element found')
}

const queryClient = new QueryClient()

ReactDOM.createRoot(root).render(
  <React.StrictMode>
    <QueryClientProvider client={queryClient}>
      <I18nProvider>
        <RouterProvider router={router} />
      </I18nProvider>
    </QueryClientProvider>
  </React.StrictMode>,
)
