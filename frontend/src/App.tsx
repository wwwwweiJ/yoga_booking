import { Link, Outlet, useNavigate } from 'react-router'
import { clearToken, getToken } from './auth/token'

export function App() {
  const navigate = useNavigate()
  const isAuthenticated = getToken() !== null

  function handleLogout() {
    clearToken()
    navigate('/login')
  }

  return (
    <div>
      <nav className="app-nav">
        <Link to="/" className="brand">
          <span aria-hidden="true">🧘</span> Yoga Booking
        </Link>
        <div className="nav-links">
          {isAuthenticated && (
            <>
              <Link to="/classes" className="nav-link">
                Classes
              </Link>
              <Link to="/bookings" className="nav-link">
                My Bookings
              </Link>
              <Link to="/organizations" className="nav-link">
                My Studio
              </Link>
              <button type="button" className="btn-ghost" onClick={handleLogout}>
                Log out
              </button>
            </>
          )}
          {!isAuthenticated && (
            <Link to="/login" className="nav-link">
              Log in
            </Link>
          )}
        </div>
      </nav>
      <main className="container">
        <Outlet />
      </main>
    </div>
  )
}
