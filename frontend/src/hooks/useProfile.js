import { useState, useEffect } from 'react'
import { useAuthContext } from '../contexts/AuthContext'

export const useProfile = (userId) => {
  const { logout } = useAuthContext()
  const [profile, setProfile] = useState(null)
  const [skills, setSkills] = useState([])
  const [allSkills, setAllSkills] = useState([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState(null)

  const fetchProfile = async () => {
    try {
      const response = await fetch(`/api/users/${userId}`, {
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('access_token')}`
        }
      })
      if (!response.ok) throw new Error('Ошибка загрузки профиля')
      const data = await response.json()
      setProfile(data)
    } catch (error) {
      setError(error.message)
      if (error.message.includes('unauthorized')) {
        logout()
      }
    }
  }

  const fetchSkills = async () => {
    try {
      // Получаем все навыки
      const skillsResponse = await fetch('/api/skills')
      if (!skillsResponse.ok) throw new Error('Ошибка получения навыков')
      const allSkillsData = await skillsResponse.json()
      setAllSkills(allSkillsData)

      // Получаем навыки пользователя
      const userSkillsResponse = await fetch(`/api/users/${userId}/skills`, {
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('access_token')}`
        }
      })
      if (!userSkillsResponse.ok) {
        const errorText = await userSkillsResponse.text()
        throw new Error(`Ошибка получения навыков пользователя: ${errorText}`)
      }
      const userSkillsData = await userSkillsResponse.json()
      setSkills(userSkillsData)
      console.log(allSkillsData)
      console.log(userSkillsData)
    } catch (error) {
      setError(error.message)
    }
  }

  useEffect(() => {
    fetchProfile()
    fetchSkills()
      .finally(() => setLoading(false))
  }, [userId, logout])

  const updateProfile = async (data) => {
    try {
      const response = await fetch(`/api/users/${userId}`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${localStorage.getItem('access_token')}`
        },
        body: JSON.stringify(data)
      })

      if (!response.ok) {
        const errorText = await response.text()
        throw new Error(errorText)
      }

      // Обновляем локальное состояние без парсинга ответа
      setProfile({ ...profile, ...data })
      return true
    } catch (error) {
      setError(error.message)
      return false
    }
  }

  const addSkill = async (skillId) => {
    try {
      const response = await fetch(`/api/users/${userId}/skills`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${localStorage.getItem('access_token')}`
        },
        body: JSON.stringify({ skill_id: Number(skillId) })
      })

      if (!response.ok) {
        throw new Error('Ошибка при добавлении навыка')
      }

      // Получаем новую информацию о навыках
      const newSkillResponse = await fetch(`/api/skills/${skillId}`, {
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('access_token')}`
        }
      })

      if (newSkillResponse.ok) {
        const newSkill = await newSkillResponse.json()
        setSkills(prevSkills => [...prevSkills, newSkill])
      } else {
        throw new Error('Не удалось получить данные нового навыка')
      }

      return true
    } catch (error) {
      throw error
    }
  }

  const deleteSkill = async (skillId) => {
    try {
      const response = await fetch(`/api/users/${userId}/skills/${skillId}`, {
        method: 'DELETE',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('access_token')}`
        }
      })

      if (!response.ok) throw new Error('Ошибка удаления навыка')

      // Обновляем локальный список навыков
      setSkills(prevSkills => prevSkills.filter(skill => skill.skill_id !== skillId))
      return true
    } catch (error) {
      throw error
    }
  }

  const deleteAccount = async () => {
    try {
      const response = await fetch(`/api/users/${userId}`, {
        method: 'DELETE',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('access_token')}`
        }
      })

      if (!response.ok) throw new Error('Ошибка удаления аккаунта')

      logout()
      return true
    } catch (error) {
      setError(error.message)
      return false
    }
  }

  return {
    profile,
    skills,
    allSkills,
    loading,
    error,
    updateProfile,
    addSkill,
    deleteSkill,
    deleteAccount,
    fetchSkills
  }
}