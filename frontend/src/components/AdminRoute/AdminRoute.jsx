import { Navigate } from 'react-router-dom'
import { useAuthContext } from '../../contexts/AuthContext'

export const AdminRoute = ({ children }) => {
    const { isAuth, role, loading } = useAuthContext()

    if (loading) {
        return <div>Загрузка...</div>
    }

    if (!isAuth || role !== 'admin') {
        return <Navigate to="/" replace />
    }

    return children
}