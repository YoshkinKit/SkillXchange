import { useState, useEffect } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { useAuthContext } from '../../contexts/AuthContext'
import { Button } from '../../components/Button/Button'
import { TeacherCard } from '../../components/TeacherCard/TeacherCard'
import { RequestModal } from '../../components/RequestModal/RequestModal'
import { Notification } from '../../components/Notification/Notification'
import './Teachers.css'

export const Teachers = () => {
    const { skillId } = useParams()
    const navigate = useNavigate()
    const { userId } = useAuthContext()
    const [teachers, setTeachers] = useState([])
    const [loading, setLoading] = useState(true)
    const [error, setError] = useState(null)
    const [selectedTeacher, setSelectedTeacher] = useState(null)
    const [notification, setNotification] = useState(null)

    useEffect(() => {
        const fetchTeachers = async () => {
            try {
                // Получаем учителей
                const teachersResponse = await fetch(`/api/users/skill/${skillId}`, {
                    headers: {
                        'Authorization': `Bearer ${localStorage.getItem('access_token')}`
                    }
                })
                if (!teachersResponse.ok) throw new Error('Ошибка загрузки данных')
                let teachersData = await teachersResponse.json()

                // Фильтруем текущего пользователя
                teachersData = teachersData.filter(teacher => teacher.user_id !== userId)

                // Получаем отзывы для каждого учителя
                const teachersWithRatings = await Promise.all(
                    teachersData.map(async (teacher) => {
                        // Получаем все отзывы
                        const reviewsResponse = await fetch(`/api/reviews/user/${teacher.user_id}`, {
                            headers: {
                                'Authorization': `Bearer ${localStorage.getItem('access_token')}`
                            }
                        })
                        const reviews = await reviewsResponse.json()

                        // Получаем отзывы по конкретному навыку
                        const skillReviewsResponse = await fetch(
                            `/api/reviews/user/${teacher.user_id}/skill/${skillId}`,
                            {
                                headers: {
                                    'Authorization': `Bearer ${localStorage.getItem('access_token')}`
                                }
                            }
                        )
                        const skillReviews = await skillReviewsResponse.json()

                        // Считаем средние рейтинги
                        const average_rating = reviews.length > 0
                            ? (reviews.reduce((sum, r) => sum + r.rating, 0) / reviews.length).toFixed(2)
                            : null

                        const skill_rating = skillReviews.length > 0
                            ? (skillReviews.reduce((sum, r) => sum + r.rating, 0) / skillReviews.length).toFixed(2)
                            : null

                        return {
                            ...teacher,
                            average_rating,
                            skill_rating
                        }
                    })
                )

                setTeachers(teachersWithRatings)
            } catch (error) {
                setError(error.message)
            } finally {
                setLoading(false)
            }
        }

        fetchTeachers()
    }, [skillId, userId])

    if (loading) return <div className="teachers__loading">Загрузка...</div>
    if (error) return <div className="teachers__error">Ошибка: {error}</div>

    return (
        <div className="teachers">
            {teachers.length > 0 ? (
                <div className="teachers__grid">
                    {teachers.map(teacher => (
                        <TeacherCard
                            key={teacher.user_id}
                            teacher={teacher}
                            skillId={skillId}
                            onRequestClick={() => setSelectedTeacher(teacher)}
                        />
                    ))}
                </div>
            ) : (
                <div className="teachers__empty">
                    <p>К сожалению, пока никто не обучает данному навыку</p>
                    <Button 
                        onClick={() => navigate('/skills')}
                        className="teachers__back-button"
                    >
                        Найти другие навыки
                    </Button>
                </div>
            )}

            <RequestModal
                isOpen={!!selectedTeacher}
                onClose={() => setSelectedTeacher(null)}
                onSubmit={async (coverLetter) => {
                    try {
                        const response = await fetch('/api/requests', {
                            method: 'POST',
                            headers: {
                                'Content-Type': 'application/json',
                                'Authorization': `Bearer ${localStorage.getItem('access_token')}`
                            },
                            body: JSON.stringify({
                                receiver_id: selectedTeacher.user_id,
                                skill_id: Number(skillId),
                                cover_letter: coverLetter
                            })
                        })

                        if (!response.ok) throw new Error('Ошибка создания запроса')
                        
                        setNotification({
                            message: 'Запрос успешно отправлен',
                            type: 'success'
                        })
                        setSelectedTeacher(null)
                    } catch (error) {
                        setNotification({
                            message: 'Ошибка при отправке запроса',
                            type: 'error'
                        })
                    }
                }}
            />

            {notification && (
                <Notification
                    message={notification.message}
                    type={notification.type}
                    onClose={() => setNotification(null)}
                />
            )}
        </div>
    )
}