import { useState } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import { useAuthContext } from '../../contexts/AuthContext'
import { Input } from '../../components/Input/Input'
import { Button } from '../../components/Button/Button'
import './Auth.css'

export const Login = () => {
  const { login } = useAuthContext()
  const navigate = useNavigate()
  const [form, setForm] = useState({
    email: '',
    password: ''
  })
  const [errors, setErrors] = useState({})
  
  const validate = () => {
    const newErrors = {}
    
    if (!form.email) {
      newErrors.email = 'Введите email'
    }
    
    if (!form.password) {
      newErrors.password = 'Введите пароль'
    }
    
    setErrors(newErrors)
    return Object.keys(newErrors).length === 0
  }
  
  const handleSubmit = async (e) => {
    e.preventDefault()
    
    if (!validate()) return
    
    try {
      const response = await fetch('/api/auth/login', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(form),
      })
      
      if (response.ok) {
        const data = await response.json()
        login(data.access_token, data.refresh_token)
        navigate('/')
      } else {
        setErrors({ submit: 'Неверный email или пароль' })
      }
    } catch (error) {
      setErrors({ submit: 'Ошибка сервера' })
    }
  }

  return (
    <div className="auth-container">
      <form className="auth-form" onSubmit={handleSubmit}>
        <h2>Вход</h2>
        
        <Input
          label="Email"
          type="email"
          value={form.email}
          onChange={(e) => setForm({ ...form, email: e.target.value })}
          error={errors.email}
        />
        
        <Input
          label="Пароль"
          type="password"
          value={form.password}
          onChange={(e) => setForm({ ...form, password: e.target.value })}
          error={errors.password}
        />
        
        {errors.submit && <div className="auth-error">{errors.submit}</div>}
        
        <Button type="submit">Войти</Button>
        
        <p className="auth-link">
          Нет аккаунта? <Link to="/register">Зарегистрироваться</Link>
        </p>
      </form>
    </div>
  )
}