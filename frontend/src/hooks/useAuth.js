import { useState, useEffect, useCallback } from 'react'
import { jwtDecode } from 'jwt-decode'

export const useAuth = () => {
  const [isAuth, setIsAuth] = useState(false)
  const [role, setRole] = useState(null)
  const [userId, setUserId] = useState(null)
  const [loading, setLoading] = useState(true)

  const login = useCallback((accessToken, refreshToken) => {
    localStorage.setItem('access_token', accessToken)
    localStorage.setItem('refresh_token', refreshToken)
    const decoded = jwtDecode(accessToken)
    setRole(decoded.role)
    setUserId(decoded.sub)
    setIsAuth(true)
  }, [])

  const logout = useCallback(() => {
    localStorage.removeItem('access_token')
    localStorage.removeItem('refresh_token')
    setIsAuth(false)
    setRole(null)
    setUserId(null)
  }, [])

  const refreshToken = useCallback(async () => {
    try {
      const refresh_token = localStorage.getItem('refresh_token')
      if (!refresh_token) {
        logout()
        return
      }

      const response = await fetch('/api/auth/refresh', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ refresh_token }),
      })

      if (response.ok) {
        const data = await response.json()
        localStorage.setItem('access_token', data.access_token)
        const decoded = jwtDecode(data.access_token)
        setRole(decoded.role)
        setUserId(decoded.sub)
        setIsAuth(true)
      } else {
        logout()
      }
    } catch (error) {
      console.error('Ошибка обновления токена:', error)
      logout()
    }
  }, [logout])

  const checkAuth = async () => {
    const token = localStorage.getItem('access_token')
    if (token) {
      try {
        const decoded = jwtDecode(token)
        // Добавляем небольшой запас времени для проверки
        if (decoded.exp * 1000 <= Date.now() + 1000) {
          // Токен истёк или близок к истечению - пробуем обновить
          await refreshToken()
        } else {
          // Токен валиден - устанавливаем состояние
          setRole(decoded.role)
          setUserId(decoded.sub)
          setIsAuth(true)
          // Устанавливаем таймер на обновление токена
          const timeLeft = decoded.exp * 1000 - Date.now() - 10000
          setTimeout(refreshToken, Math.max(0, timeLeft))
        }
      } catch (error) {
        console.error('Ошибка проверки токена:', error)
        try {
          await refreshToken()
        } catch (refreshError) {
          console.error('Ошибка обновления токена:', refreshError)
          logout()
        }
      }
    } else {
      logout()
    }
    
    setLoading(false)
  }

  useEffect(() => {
    checkAuth()
  
    const interval = setInterval(checkAuth, 300000)
    return () => clearInterval(interval)
  }, [refreshToken])

  return { isAuth, role, userId, login, logout, refreshToken, loading }
}