import { Navigate } from 'react-router-dom'
import { useAuthContext } from '../../contexts/AuthContext'

export const ProtectedRoute = ({ children }) => {
  const { isAuth, loading } = useAuthContext()
  
  if (loading) {
    // Можно заменить на компонент загрузки или скелетон
    return <div>Загрузка...</div>
  }

  if (!isAuth) {
    return <Navigate to="/login" replace />
  }
  
  return children
}