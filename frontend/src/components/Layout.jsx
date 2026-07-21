import { Outlet, useNavigate } from 'react-router-dom';
import { useAuth } from '../AuthContext';

export function Layout() {
  const { logout } = useAuth();
  const navigate = useNavigate();

  async function handleLogout() {
    await logout();
    navigate('/login', { replace: true });
  }

  return (
    <div className="app-shell">
      <div className="topbar">
        <div className="topbar-brand">
          <span className="topbar-brand-dot" />
          Push Platform
        </div>
        <button className="btn btn-secondary btn-sm" onClick={handleLogout}>
          Log out
        </button>
      </div>
      <Outlet />
    </div>
  );
}
