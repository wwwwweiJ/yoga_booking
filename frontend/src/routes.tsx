import { createBrowserRouter } from 'react-router'
import { App } from './App'
import { Login } from './auth/Login'
import { Register } from './auth/Register'
import { RequireAuth } from './auth/RequireAuth'
import { Home } from './pages/Home'
import { OrganizationsList } from './pages/organizations/List'
import { ClassesList } from './pages/classes/List'
import { ClassForm } from './pages/classes/Form'
import { BookingsList } from './pages/bookings/List'
import { Admin } from './pages/admin/Admin'
import { StudioEditor } from './pages/studio/StudioEditor'
import { StudioPublicPage } from './pages/studio/StudioPublicPage'
// scaffold:imports

export const router = createBrowserRouter([
  {
    path: '/',
    element: <App />,
    children: [
      { index: true, element: <Home /> },
      { path: 'login', element: <Login /> },
      { path: 'register/:token', element: <Register /> },
      { path: 'studio/:token', element: <StudioPublicPage /> },
      {
        element: <RequireAuth />,
        children: [
          { path: 'organizations', element: <OrganizationsList /> },
          { path: 'classes', element: <ClassesList /> },
          { path: 'classes/new', element: <ClassForm /> },
          { path: 'classes/:id/edit', element: <ClassForm /> },
          { path: 'bookings', element: <BookingsList /> },
          { path: 'admin', element: <Admin /> },
          { path: 'studio/edit', element: <StudioEditor /> },
          // scaffold:routes
        ],
      },
    ],
  },
])
