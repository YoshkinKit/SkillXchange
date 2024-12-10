import { useState, useEffect } from 'react'
import { useAuthContext } from '../contexts/AuthContext'

export const useProfile = (userId) => {
  const { logout } = useAuthContext()
  const [profile, setProfile] = useState(null)
  const [skills, setSkills] = useState([])
  const [allSkills, setAllSkills] = useState([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState(null)
  const [reviews, setReviews] = useState([])
  const [currentPage, setCurrentPage] = useState(1)
  const [reviewsPerPage] = useState(5)
  const [averageRating, setAverageRating] = useState(null)
  const [incomingRequests, setIncomingRequests] = useState([])
  const [outgoingRequests, setOutgoingRequests] = useState([])
  const [incomingPage, setIncomingPage] = useState(1)
  const [outgoingPage, setOutgoingPage] = useState(1)
  const [requestsPerPage] = useState(5)

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

  const fetchReviews = async () => {
    try {
      // Получаем отзывы
      const reviewsResponse = await fetch(`/api/reviews/user/${userId}`, {
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('access_token')}`
        }
      });

      if (!reviewsResponse.ok) throw new Error('Ошибка получения отзывов');
      const reviewsData = await reviewsResponse.json();

      // Получаем уникальные ID отправителей и навыков
      const senderIds = [...new Set(reviewsData.map(review => review.sender_id))];
      const skillIds = [...new Set(reviewsData.map(review => review.skill_id))];

      // Получаем данные всех отправителей и навыков
      const [usersData, skillsData] = await Promise.all([
        Promise.all(
          senderIds.map(senderId =>
            fetch(`/api/users/${senderId}`, {
              headers: {
                'Authorization': `Bearer ${localStorage.getItem('access_token')}`
              }
            }).then(res => res.json())
          )
        ),
        Promise.all(
          skillIds.map(skillId =>
            fetch(`/api/skills/${skillId}`).then(res => res.json())
          )
        )
      ]);

      // Создаем мапы пользователей и навыков
      const usersMap = new Map(
        usersData.map(user => [user.user_id, user.username])
      );
      const skillsMap = new Map(
        skillsData.map(skill => [skill.skill_id, skill.title])
      );

      // Объединяем все данные
      const reviewsWithDetails = reviewsData.map(review => ({
        ...review,
        username: usersMap.get(review.sender_id),
        skillTitle: skillsMap.get(review.skill_id)
      }));

      setReviews(reviewsWithDetails);
      setAverageRating(
        reviewsData.length > 0
          ? (reviewsData.reduce((sum, review) => sum + review.rating, 0) / reviewsData.length).toFixed(2)
          : null
      );
    } catch (error) {
      setError(error.message);
    }
  };

  const getCurrentReviews = () => {
    const indexOfLastReview = currentPage * reviewsPerPage
    const indexOfFirstReview = indexOfLastReview - reviewsPerPage
    return reviews.slice(indexOfFirstReview, indexOfLastReview)
  }

  const fetchRequests = async () => {
    try {
      // Получаем входящие и исходящие запросы
      const [incomingResponse, outgoingResponse] = await Promise.all([
        fetch('/api/requests/received', {
          headers: {
            'Authorization': `Bearer ${localStorage.getItem('access_token')}`
          }
        }),
        fetch('/api/requests/sent', {
          headers: {
            'Authorization': `Bearer ${localStorage.getItem('access_token')}`
          }
        })
      ]);

      console.log(incomingResponse)
      console.log(outgoingResponse)

      if (!incomingResponse.ok || !outgoingResponse.ok) {
        throw new Error('Ошибка получения запросов');
      }

      const [incomingData, outgoingData] = await Promise.all([
        incomingResponse.json(),
        outgoingResponse.json()
      ]);

      // Получаем уникальные ID пользователей и навыков
      const userIds = [...new Set([
        ...incomingData.map(r => r.sender_id),
        ...outgoingData.map(r => r.receiver_id)
      ])];
      const skillIds = [...new Set([
        ...incomingData.map(r => r.skill_id),
        ...outgoingData.map(r => r.skill_id)
      ])];

      // Получаем информацию о пользователях и навыках
      const [usersData, skillsData] = await Promise.all([
        Promise.all(
          userIds.map(id =>
            fetch(`/api/users/${id}`, {
              headers: {
                'Authorization': `Bearer ${localStorage.getItem('access_token')}`
              }
            }).then(res => res.json())
          )
        ),
        Promise.all(
          skillIds.map(id =>
            fetch(`/api/skills/${id}`).then(res => res.json())
          )
        )
      ]);

      // Создаем мапы для быстрого доступа
      const usersMap = new Map(usersData.map(user => [user.user_id, user.username]));
      const skillsMap = new Map(skillsData.map(skill => [skill.skill_id, skill.title]));

      // Добавляем информацию о пользователях и навыках
      const processedIncoming = incomingData.map(request => ({
        ...request,
        username: usersMap.get(request.sender_id),
        skillTitle: skillsMap.get(request.skill_id)
      }));

      const processedOutgoing = outgoingData.map(request => ({
        ...request,
        username: usersMap.get(request.receiver_id),
        skillTitle: skillsMap.get(request.skill_id)
      }));

      setIncomingRequests(processedIncoming);
      setOutgoingRequests(processedOutgoing);
    } catch (error) {
      console.log(error)
      setError(error.message);
    }
  };

  const handleAcceptRequest = async (requestId) => {
    try {
      const response = await fetch(`/api/requests/${requestId}/status`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${localStorage.getItem('access_token')}`
        },
        body: JSON.stringify({ status: 'accept' })
      });

      if (!response.ok) throw new Error('Ошибка при принятии запроса');

      await fetchRequests(); // Обновляем список запросов
      return true;
    } catch (error) {
      setError(error.message);
      return false;
    }
  };

  const handleDeclineRequest = async (requestId) => {
    try {
      const response = await fetch(`/api/requests/${requestId}/status`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${localStorage.getItem('access_token')}`
        },
        body: JSON.stringify({ status: 'decline' })
      });

      if (!response.ok) throw new Error('Ошибка при отклонении запроса');

      await fetchRequests(); // Обновляем список запросов
      return true;
    } catch (error) {
      setError(error.message);
      return false;
    }
  };

  const getCurrentPageRequests = (requests, currentPage) => {
    const indexOfLast = currentPage * requestsPerPage;
    const indexOfFirst = indexOfLast - requestsPerPage;
    return requests.slice(indexOfFirst, indexOfLast);
};

  useEffect(() => {
    Promise.all([fetchProfile(), fetchSkills(), fetchReviews()])
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
    fetchSkills,
    reviews: getCurrentReviews(),
    totalReviews: reviews.length,
    totalPages: Math.ceil(reviews.length / reviewsPerPage),
    currentPage,
    setCurrentPage,
    averageRating,
    fetchReviews,
    incomingRequests,
    outgoingRequests,
    handleAcceptRequest,
    handleDeclineRequest,
    fetchRequests,
    getCurrentPageRequests,
    requestsPerPage,
    incomingPage,
    setIncomingPage,
    outgoingPage,
    setOutgoingPage
  }
}