import { useState, useEffect } from 'react'
import { useParams } from 'react-router-dom'
import { ReviewForm } from '../../components/ReviewForm/ReviewForm'
import { Tooltip } from '../../components/Tooltip/Tooltip'
import { useAuthContext } from '../../contexts/AuthContext'
import { Pagination } from '../../components/Pagination/Pagination'
import { Review } from '../../components/Review/Review'
import { Button } from '../../components/Button/Button'
import './UserPage.css'

export const UserPage = () => {
    const { userId } = useParams()
    const { userId: currentUserId } = useAuthContext()
    const [showReviewForm, setShowReviewForm] = useState(false)
    const [canReview, setCanReview] = useState(false)
    const [learnedSkills, setLearnedSkills] = useState([])
    const [user, setUser] = useState(null)
    const [skills, setSkills] = useState([])
    const [reviews, setReviews] = useState([])
    const [loading, setLoading] = useState(true)
    const [error, setError] = useState(null)
    const [currentPage, setCurrentPage] = useState(1)
    const [reviewsPerPage] = useState(5)
    const [averageRating, setAverageRating] = useState(null)

    useEffect(() => {
        const fetchData = async () => {
            try {
                const [userResponse, skillsResponse, reviewsResponse] = await Promise.all([
                    fetch(`/api/users/${userId}`, {
                        headers: {
                            'Authorization': `Bearer ${localStorage.getItem('access_token')}`
                        }
                    }),
                    fetch(`/api/users/${userId}/skills`, {
                        headers: {
                            'Authorization': `Bearer ${localStorage.getItem('access_token')}`
                        }
                    }),
                    fetch(`/api/reviews/user/${userId}`, {
                        headers: {
                            'Authorization': `Bearer ${localStorage.getItem('access_token')}`
                        }
                    })
                ])
        
                const [userData, skillsData, reviewsData] = await Promise.all([
                    userResponse.json(),
                    skillsResponse.json(),
                    reviewsResponse.json()
                ])
        
                // Получаем информацию об авторах отзывов и навыках
                const reviewsWithDetails = await Promise.all(
                    reviewsData.map(async (review) => {
                        const [userResponse, skillResponse] = await Promise.all([
                            fetch(`/api/users/${review.sender_id}`, {
                                headers: {
                                    'Authorization': `Bearer ${localStorage.getItem('access_token')}`
                                }
                            }),
                            fetch(`/api/skills/${review.skill_id}`)
                        ])
                        
                        const [userData, skillData] = await Promise.all([
                            userResponse.json(),
                            skillResponse.json()
                        ])
        
                        return {
                            ...review,
                            username: userData.username,
                            skillTitle: skillData.title
                        }
                    })
                )
        
                setUser(userData)
                setSkills(skillsData)
                setReviews(reviewsWithDetails)
        
                // Вычисляем средний рейтинг
                if (reviewsWithDetails.length > 0) {
                    const avgRating = (
                        reviewsWithDetails.reduce((sum, review) => sum + review.rating, 0) /
                        reviewsWithDetails.length
                    ).toFixed(2)
                    setAverageRating(avgRating)
                }
            } catch (error) {
                setError(error.message)
            } finally {
                setLoading(false)
            }
        }

        fetchData()

        const checkCanReview = async () => {
            try {
                const response = await fetch(`/api/requests/sent`, {
                    headers: {
                        'Authorization': `Bearer ${localStorage.getItem('access_token')}`
                    }
                });

                if (response.ok) {
                    const requests = await response.json();

                    // Фильтруем принятые заявки для текущего преподавателя
                    const acceptedRequests = requests.filter(request =>
                        request.receiver_id === Number(userId) &&
                        request.status === 'accept'
                    );

                    if (acceptedRequests.length > 0) {
                        // Получаем информацию о навыках
                        const skillsResponse = await Promise.all(
                            acceptedRequests.map(request =>
                                fetch(`/api/skills/${request.skill_id}`).then(res => res.json())
                            )
                        );

                        const acceptedSkills = skillsResponse.map(skill => ({
                            skill_id: skill.skill_id,
                            title: skill.title
                        }));

                        setCanReview(true);
                        setLearnedSkills(acceptedSkills);
                    } else {
                        setCanReview(false);
                        setLearnedSkills([]);
                    }
                }
            } catch (error) {
                console.error('Ошибка при проверке возможности оставить отзыв:', error);
            }
        };

        if (currentUserId && currentUserId !== Number(userId)) {
            checkCanReview()
        }

    }, [userId, currentUserId])

    const handleReviewSubmit = async (formData) => {
        try {
            const response = await fetch('/api/reviews', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${localStorage.getItem('access_token')}`
                },
                body: JSON.stringify({
                    receiver_id: Number(userId),
                    skill_id: Number(formData.skill_id),
                    rating: formData.rating,
                    comment: formData.comment
                })
            })
    
            if (response.ok) {
                // Обновляем список отзывов через основную функцию загрузки
                await fetchData()
                setShowReviewForm(false)
            }
        } catch (error) {
            console.error('Ошибка при отправке отзыва:', error)
        }
    }

    if (loading) return <div className="user-page__loading">Загрузка...</div>
    if (error) return <div className="user-page__error">Ошибка: {error}</div>
    if (!user) return <div className="user-page__error">Пользователь не найден</div>

    // Получаем текущие отзывы для страницы
    const indexOfLastReview = currentPage * reviewsPerPage
    const indexOfFirstReview = indexOfLastReview - reviewsPerPage
    const currentReviews = reviews.slice(indexOfFirstReview, indexOfLastReview)

    return (
        <div className="user-page">
            <div className="user-page__header">
                <h1>{user.username}</h1>
                <p className="user-page__bio">{user.bio || 'Биография отсутствует'}</p>
            </div>

            <div className="user-page__skills">
                <h2>Навыки преподавания</h2>
                {skills.length > 0 ? (
                    <ul className="skills-list">
                        {skills.map(skill => (
                            <li key={skill.skill_id} className="skill-item">
                                {skill.title}
                            </li>
                        ))}
                    </ul>
                ) : (
                    <p className="no-skills">Пользователь пока не преподает ни одного навыка</p>
                )}
            </div>

            <div className="user-page__reviews">
                <div className="reviews-header">
                    <h2>Отзывы</h2>
                    <div className="reviews-actions">
                        {averageRating && (
                            <div className="average-rating">
                                Средний рейтинг: <span>{averageRating} ★</span>
                            </div>
                        )}
                        {currentUserId && currentUserId !== Number(userId) && (
                            !canReview ? (
                                <Tooltip text="Вы еще не проходили обучение у этого пользователя">
                                    <span>
                                        <Button disabled>
                                            Написать отзыв
                                        </Button>
                                    </span>
                                </Tooltip>
                            ) : (
                                <Button onClick={() => setShowReviewForm(true)}>
                                    Написать отзыв
                                </Button>
                            )
                        )}
                    </div>
                </div>

                {/* Добавляем форму отзыва */}
                <ReviewForm
                    isOpen={showReviewForm}
                    onClose={() => setShowReviewForm(false)}
                    onSubmit={handleReviewSubmit}
                    skills={learnedSkills}
                />

                {reviews.length > 0 ? (
                    <>
                        <div className="reviews-list">
                            {currentReviews.map(review => (
                                <Review key={review.review_id} review={review} />
                            ))}
                        </div>
                        <Pagination
                            currentPage={currentPage}
                            totalPages={Math.ceil(reviews.length / reviewsPerPage)}
                            onPageChange={setCurrentPage}
                        />
                    </>
                ) : (
                    <p className="no-reviews">Отзывов пока нет</p>
                )}
            </div>
        </div>
    )
}