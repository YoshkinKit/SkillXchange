import { useState } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import { Input } from '../../components/Input/Input'
import { Button } from '../../components/Button/Button'
import './Auth.css'

export const Register = () => {
  const navigate = useNavigate()
  const [form, setForm] = useState({
    username: '',
    email: '',
    password: '',
    confirmPassword: ''
  })
  const [errors, setErrors] = useState({})
  
  const validate = () => {
    const newErrors = {}
    
    if (!form.username) {
      newErrors.username = 'Введите имя пользователя'
    }
    
    if (!form.email) {
      newErrors.email = 'Введите email'
    } else if (!/^[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}$/i.test(form.email)) {
      newErrors.email = 'Неверный формат email'
    }
    
    if (!form.password) {
      newErrors.password = 'Введите пароль'
    } else if (form.password.length < 6) {
      newErrors.password = 'Пароль должен быть не менее 6 символов'
    }
    
    if (form.password !== form.confirmPassword) {
      newErrors.confirmPassword = 'Пароли не совпадают'
    }
    
    setErrors(newErrors)
    return Object.keys(newErrors).length === 0
  }
  
  const handleSubmit = async (e) => {
    e.preventDefault()
    
    if (!validate()) return
    
    try {
      const response = await fetch('/api/auth/register', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          username: form.username,
          email: form.email,
          password: form.password,
          role: 'user'
        }),
      })
      
      const data = await response.json()
      
      if (response.ok) {
        navigate('/login')
      } else {
        // Устанавливаем ошибку в соответствующее поле
        if (data.message.includes('имен')) {
          setErrors({ username: data.message })
        } else if (data.message.includes('email')) {
          setErrors({ email: data.message })
        } else {
          setErrors({ submit: data.message || 'Ошибка регистрации' })
        }
      }
    } catch (error) {
      setErrors({ submit: 'Ошибка сервера' })
    }
  }

  return (
    <div className="auth-container">
      <form className="auth-form" onSubmit={handleSubmit}>
        <h2>Регистрация</h2>
        
        <Input
          label="Имя пользователя"
          value={form.username}
          onChange={(e) => setForm({ ...form, username: e.target.value })}
          error={errors.username}
        />
        
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
        
        <Input
          label="Подтвердите пароль"
          type="password"
          value={form.confirmPassword}
          onChange={(e) => setForm({ ...form, confirmPassword: e.target.value })}
          error={errors.confirmPassword}
        />
        
        {errors.submit && <div className="auth-error">{errors.submit}</div>}
        
        <Button type="submit">Зарегистрироваться</Button>
        
        <p className="auth-link">
          Уже есть аккаунт? <Link to="/login">Войти</Link>
        </p>
      </form>
    </div>
  )
}