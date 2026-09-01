import { Link, Outlet, useNavigate } from 'react-router'
import { clearToken, getToken } from './auth/token'
import { LOCALES, useI18n } from './i18n'

export function App() {
  const navigate = useNavigate()
  const isAuthenticated = getToken() !== null
  const { t, locale, setLocale } = useI18n()

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
                {t('nav.classes')}
              </Link>
              <Link to="/bookings" className="nav-link">
                {t('nav.bookings')}
              </Link>
              <Link to="/organizations" className="nav-link">
                {t('nav.studio')}
              </Link>
              <button type="button" className="btn-ghost" onClick={handleLogout}>
                {t('nav.logout')}
              </button>
            </>
          )}
          {!isAuthenticated && (
            <Link to="/login" className="nav-link">
              {t('nav.login')}
            </Link>
          )}
          <div className="lang-switch">
            {LOCALES.map(({ code, label }) => (
              <button
                key={code}
                type="button"
                className={code === locale ? 'is-active' : ''}
                onClick={() => setLocale(code)}
              >
                {label}
              </button>
            ))}
          </div>
        </div>
      </nav>
      <main className="container">
        <Outlet />
      </main>
    </div>
  )
}
