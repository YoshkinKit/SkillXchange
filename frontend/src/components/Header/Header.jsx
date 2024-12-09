import { Link, useNavigate } from 'react-router-dom'
import { useAuthContext } from '../../contexts/AuthContext'
import { Button } from '../Button/Button'
import './Header.css'


export const Header = () => {
  const { isAuth, role, logout } = useAuthContext()
  const navigate = useNavigate()

  const handleLogout = () => {
    logout()
    navigate('/')
  }

  return (
    <header className="header">
      <Link to="/" className="header__logo">
        SkillXChange
      </Link>
      
      <div className="header__buttons">
        {isAuth ? (
          <>
            <Button variant="outline" onClick={() => navigate('/chat')}>
              Чат
            </Button>
            <Button variant="outline" onClick={() => navigate('/profile')}>
              Личный кабинет
            </Button>
            {role === 'admin' && (
              <Button variant="outline" onClick={() => navigate('/admin')}>
                Админ-панель
              </Button>
            )}
            <Button onClick={handleLogout}>
              Выйти
            </Button>
          </>
        ) : (
          <>
            <Button variant="outline" onClick={() => navigate('/login')}>
              Войти
            </Button>
            <Button onClick={() => navigate('/register')}>
              Зарегистрироваться
            </Button>
          </>
        )}
      </div>
    </header>
  )
}