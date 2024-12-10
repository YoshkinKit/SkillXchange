import { useState, useEffect } from 'react'
import { Link } from 'react-router-dom'
import { Button } from '../../components/Button/Button'
import { SkillForm } from '../../components/SkillForm/SkillForm'
import { Modal } from '../../components/Modal/Modal'
import { Notification } from '../../components/Notification/Notification'
import { CategoryForm } from '../../components/CategoryForm/CategoryForm'
import { UserForm } from '../../components/UserForm/UserForm'
import { AdminReviewForm } from '../../components/ReviewForm/AdminReviewForm'
import { Pagination } from '../../components/Pagination/Pagination'

import './Admin.css'

export const Admin = () => {
    const userId = localStorage.getItem('user_id')
    const [activeTab, setActiveTab] = useState('skills')
    const [skills, setSkills] = useState([])
    const [categories, setCategories] = useState([])
    const [showSkillForm, setShowSkillForm] = useState(false)
    const [editingSkill, setEditingSkill] = useState(null)
    const [deletingSkill, setDeletingSkill] = useState(null)
    const [notification, setNotification] = useState(null)
    const [showCategoryForm, setShowCategoryForm] = useState(false)
    const [editingCategory, setEditingCategory] = useState(null)
    const [deletingCategory, setDeletingCategory] = useState(null)
    const [users, setUsers] = useState([])
    const [editingUser, setEditingUser] = useState(null)
    const [deletingUser, setDeletingUser] = useState(null)
    const [reviews, setReviews] = useState([])
    const [editingReview, setEditingReview] = useState(null)
    const [deletingReview, setDeletingReview] = useState(null)
    const [skillsPage, setSkillsPage] = useState(1)
    const [categoriesPage, setCategoriesPage] = useState(1)
    const [usersPage, setUsersPage] = useState(1)
    const [reviewsPage, setReviewsPage] = useState(1)
    const [itemsPerPage] = useState(5)


    useEffect(() => {
        fetchData()
    }, [])

    const fetchData = async () => {
        try {
            const [skillsRes, categoriesRes, usersRes, reviewsResponse] = await Promise.all([
                fetch('/api/skills'),
                fetch('/api/categories'),
                fetch('/api/users', {
                    headers: {
                        'Authorization': `Bearer ${localStorage.getItem('access_token')}`
                    }
                }),
                fetch('/api/reviews', {
                    headers: {
                        'Authorization': `Bearer ${localStorage.getItem('access_token')}`
                    }
                })
            ])

            const [skillsData, categoriesData, usersData] = await Promise.all([
                skillsRes.json(),
                categoriesRes.json(),
                usersRes.json()
            ])

            setSkills(skillsData)
            setCategories(categoriesData)
            setUsers(usersData)

            if (reviewsResponse.ok) {
                const reviewsData = await reviewsResponse.json()

                // Получаем информацию о пользователях и навыках для каждого отзыва
                const reviewsWithDetails = await Promise.all(
                    reviewsData.map(async (review) => {
                        const [senderRes, receiverRes, skillRes] = await Promise.all([
                            fetch(`/api/users/${review.sender_id}`, {
                                headers: {
                                    'Authorization': `Bearer ${localStorage.getItem('access_token')}`
                                }
                            }),
                            fetch(`/api/users/${review.receiver_id}`, {
                                headers: {
                                    'Authorization': `Bearer ${localStorage.getItem('access_token')}`
                                }
                            }),
                            fetch(`/api/skills/${review.skill_id}`)
                        ])

                        const [sender, receiver, skill] = await Promise.all([
                            senderRes.json(),
                            receiverRes.json(),
                            skillRes.json()
                        ])

                        return {
                            ...review,
                            senderName: sender.username,
                            receiverName: receiver.username,
                            skillTitle: skill.title
                        }
                    })
                )

                setReviews(reviewsWithDetails)
            }
        } catch (error) {
            console.error('Ошибка загрузки данных:', error)
            setNotification({
                message: 'Ошибка при загрузке данных',
                type: 'error'
            })
        }
    }

    const handleAddSkill = async (formData) => {
        try {
            const response = await fetch('/api/admin/skills', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${localStorage.getItem('access_token')}`
                },
                body: JSON.stringify(formData)
            })

            if (response.ok) {
                await fetchData()
                setShowSkillForm(false)
                setNotification({ message: 'Навык успешно добавлен', type: 'success' })
            } else {
                const error = await response.text()
                setNotification({ message: error, type: 'error' })
            }
        } catch (error) {
            setNotification({ message: 'Ошибка при добавлении навыка', type: 'error' })
        }
    }

    const handleEditSkill = async (formData) => {
        try {
            const response = await fetch(`/api/admin/skills/${editingSkill.skill_id}`, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${localStorage.getItem('access_token')}`
                },
                body: JSON.stringify({
                    title: formData.title || undefined,
                    description: formData.description || undefined,
                    category_id: formData.category_id || undefined
                })
            })

            if (response.ok) {
                await fetchData()
                setEditingSkill(null)
                setNotification({ message: 'Навык успешно обновлен', type: 'success' })
            } else {
                const error = await response.text()
                setNotification({ message: error, type: 'error' })
            }
        } catch (error) {
            setNotification({ message: 'Ошибка при обновлении навыка', type: 'error' })
        }
    }

    const handleDeleteSkill = async () => {
        try {
            const response = await fetch(`/api/admin/skills/${deletingSkill.skill_id}`, {
                method: 'DELETE',
                headers: {
                    'Authorization': `Bearer ${localStorage.getItem('access_token')}`
                }
            })

            if (response.ok) {
                await fetchData()
                setDeletingSkill(null)
                setNotification({ message: 'Навык успешно удален', type: 'success' })
            } else {
                const error = await response.text()
                setNotification({ message: error, type: 'error' })
            }
        } catch (error) {
            setNotification({ message: 'Ошибка при удалении навыка', type: 'error' })
        }
    }

    const handleAddCategory = async (formData) => {
        try {
            const response = await fetch('/api/admin/categories', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${localStorage.getItem('access_token')}`
                },
                body: JSON.stringify(formData)
            })

            if (response.ok) {
                await fetchData()
                setShowCategoryForm(false)
                setNotification({ message: 'Категория успешно добавлена', type: 'success' })
            } else {
                const error = await response.text()
                setNotification({ message: error, type: 'error' })
            }
        } catch (error) {
            setNotification({ message: 'Ошибка при добавлении категории', type: 'error' })
        }
    }

    const handleEditCategory = async (formData) => {
        try {
            const response = await fetch(`/api/admin/categories/${editingCategory.category_id}`, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${localStorage.getItem('access_token')}`
                },
                body: JSON.stringify(formData)
            })

            if (response.ok) {
                await fetchData()
                setEditingCategory(null)
                setNotification({
                    message: 'Категория успешно обновлена',
                    type: 'success'
                })
            } else {
                const error = await response.text()
                setNotification({
                    message: error || 'Ошибка при обновлении категории',
                    type: 'error'
                })
            }
        } catch (error) {
            setNotification({
                message: 'Ошибка при обновлении категории',
                type: 'error'
            })
        }
    }

    const handleDeleteCategory = async () => {
        try {
            const response = await fetch(`/api/admin/categories/${deletingCategory.category_id}`, {
                method: 'DELETE',
                headers: {
                    'Authorization': `Bearer ${localStorage.getItem('access_token')}`
                }
            })

            if (response.ok) {
                await fetchData()
                setDeletingCategory(null)
                setNotification({
                    message: 'Категория успешно удалена',
                    type: 'success'
                })
            } else {
                const error = await response.text()
                setNotification({
                    message: error || 'Ошибка при удалении категории',
                    type: 'error'
                })
            }
        } catch (error) {
            setNotification({
                message: 'Ошибка при удалении категории',
                type: 'error'
            })
        }
    }

    const handleEditUser = async (formData) => {
        try {
            const response = await fetch(`/api/users/${editingUser.user_id}`, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${localStorage.getItem('access_token')}`
                },
                body: JSON.stringify({
                    username: formData.username,
                    bio: formData.bio
                })
            })

            if (response.ok) {
                await fetchData()
                setEditingUser(null)
                setNotification({
                    message: 'Пользователь успешно обновлен',
                    type: 'success'
                })
            } else {
                const error = await response.text()
                setNotification({
                    message: error || 'Ошибка при обновлении пользователя',
                    type: 'error'
                })
            }
        } catch (error) {
            setNotification({
                message: 'Ошибка при обновлении пользователя',
                type: 'error'
            })
        }
    }

    const handleDeleteUser = async () => {
        try {
            const response = await fetch(`/api/users/${deletingUser.user_id}`, {
                method: 'DELETE',
                headers: {
                    'Authorization': `Bearer ${localStorage.getItem('access_token')}`
                }
            })

            if (response.ok) {
                await fetchData()
                setDeletingUser(null)
                setNotification({
                    message: 'Пользователь успешно удален',
                    type: 'success'
                })
            } else {
                const error = await response.text()
                setNotification({
                    message: error || 'Ошибка при удалении пользователя',
                    type: 'error'
                })
            }
        } catch (error) {
            setNotification({
                message: 'Ошибка при удалении пользователя',
                type: 'error'
            })
        }
    }

    const handleUpdateReview = async (formData) => {
        try {
            const response = await fetch(`/api/reviews/${editingReview.review_id}`, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${localStorage.getItem('access_token')}`
                },
                body: JSON.stringify(formData)
            })

            if (response.ok) {
                await fetchData()
                setEditingReview(null)
                setNotification({
                    message: 'Отзыв успешно обновлен',
                    type: 'success'
                })
            } else {
                const error = await response.text()
                setNotification({
                    message: error || 'Ошибка при обновлении отзыва',
                    type: 'error'
                })
            }
        } catch (error) {
            setNotification({
                message: 'Ошибка при обновлении отзыва',
                type: 'error'
            })
        }
    }

    const handleDeleteReview = async () => {
        try {
            const response = await fetch(`/api/reviews/${deletingReview.review_id}`, {
                method: 'DELETE',
                headers: {
                    'Authorization': `Bearer ${localStorage.getItem('access_token')}`
                }
            })

            if (response.ok) {
                await fetchData()
                setDeletingReview(null)
                setNotification({
                    message: 'Отзыв успешно удален',
                    type: 'success'
                })
            } else {
                const error = await response.text()
                setNotification({
                    message: error || 'Ошибка при удалении отзыва',
                    type: 'error'
                })
            }
        } catch (error) {
            setNotification({
                message: 'Ошибка при удалении отзыва',
                type: 'error'
            })
        }
    }

    const getCurrentPageItems = (items, currentPage) => {
        const indexOfLast = currentPage * itemsPerPage
        const indexOfFirst = indexOfLast - itemsPerPage
        return items.slice(indexOfFirst, indexOfLast)
    }

    return (
        <div className="admin-page">
            <div className="admin-sidebar">
                <Button
                    variant={activeTab === 'skills' ? 'default' : 'outline'}
                    onClick={() => setActiveTab('skills')}
                >
                    Навыки
                </Button>
                <Button
                    variant={activeTab === 'categories' ? 'default' : 'outline'}
                    onClick={() => setActiveTab('categories')}
                >
                    Категории
                </Button>
                <Button
                    variant={activeTab === 'users' ? 'default' : 'outline'}
                    onClick={() => setActiveTab('users')}
                >
                    Пользователи
                </Button>
                <Button
                    variant={activeTab === 'reviews' ? 'default' : 'outline'}
                    onClick={() => setActiveTab('reviews')}
                >
                    Отзывы
                </Button>
            </div>

            <div className="admin-content">
                {activeTab === 'skills' && (
                    <div>
                        <div className="admin-header">
                            <h2>Управление навыками</h2>
                            <Button onClick={() => setShowSkillForm(true)}>
                                Добавить навык
                            </Button>
                        </div>

                        <div className="skills-list">
                            {getCurrentPageItems(skills, skillsPage).map(skill => (
                                <div key={skill.skill_id} className="skill-item">
                                    <div className="skill-item__info">
                                        <h3>{skill.title}</h3>
                                        <p>{skill.description}</p>
                                    </div>
                                    <div className="skill-item__actions">
                                        <Button
                                            variant="outline"
                                            onClick={() => setEditingSkill(skill)}
                                        >
                                            Изменить
                                        </Button>
                                        <Button
                                            variant="outline"
                                            onClick={() => setDeletingSkill(skill)}
                                        >
                                            Удалить
                                        </Button>
                                    </div>
                                </div>
                            ))}
                        </div>
                        {skills.length > itemsPerPage && (
                            <Pagination
                                currentPage={skillsPage}
                                totalPages={Math.ceil(skills.length / itemsPerPage)}
                                onPageChange={setSkillsPage}
                            />
                        )}

                        <SkillForm
                            isOpen={showSkillForm}
                            onClose={() => setShowSkillForm(false)}
                            onSubmit={handleAddSkill}
                            categories={categories}
                        />

                        <SkillForm
                            isOpen={!!editingSkill}
                            onClose={() => setEditingSkill(null)}
                            onSubmit={handleEditSkill}
                            categories={categories}
                            initialData={editingSkill}
                        />

                        <Modal
                            isOpen={!!deletingSkill}
                            onClose={() => setDeletingSkill(null)}
                            title="Подтверждение удаления"
                        >
                            <div className="delete-confirmation">
                                <p>Вы уверены, что хотите удалить навык "{deletingSkill?.title}"?</p>
                                <div className="modal-actions">
                                    <Button onClick={handleDeleteSkill}>
                                        Удалить
                                    </Button>
                                    <Button
                                        variant="outline"
                                        onClick={() => setDeletingSkill(null)}
                                    >
                                        Отмена
                                    </Button>
                                </div>
                            </div>
                        </Modal>

                    </div>
                )}

                {activeTab === 'categories' && (
                    <div>
                        <div className="admin-header">
                            <h2>Управление категориями</h2>
                            <Button onClick={() => setShowCategoryForm(true)}>
                                Добавить категорию
                            </Button>
                        </div>

                        <div className="categories-list">
                            {getCurrentPageItems(categories, categoriesPage).map(category => (
                                <div key={category.category_id} className="category-item">
                                    <div className="category-item__info">
                                        <h3>{category.title}</h3>
                                        <p>{category.description}</p>
                                    </div>
                                    <div className="category-item__actions">
                                        <Button
                                            variant="outline"
                                            onClick={() => setEditingCategory(category)}
                                        >
                                            Изменить
                                        </Button>
                                        <Button
                                            variant="outline"
                                            onClick={() => setDeletingCategory(category)}
                                        >
                                            Удалить
                                        </Button>
                                    </div>
                                </div>
                            ))}
                        </div>
                        {categories.length > itemsPerPage && (
                            <Pagination
                                currentPage={categoriesPage}
                                totalPages={Math.ceil(categories.length / itemsPerPage)}
                                onPageChange={setCategoriesPage}
                            />
                        )}

                        <CategoryForm
                            isOpen={showCategoryForm}
                            onClose={() => setShowCategoryForm(false)}
                            onSubmit={handleAddCategory}
                        />

                        <CategoryForm
                            isOpen={!!editingCategory}
                            onClose={() => setEditingCategory(null)}
                            onSubmit={handleEditCategory}
                            initialData={editingCategory}
                        />

                        <Modal
                            isOpen={!!deletingCategory}
                            onClose={() => setDeletingCategory(null)}
                            title="Подтверждение удаления"
                        >
                            <div className="delete-confirmation">
                                <p>
                                    Вы уверены, что хотите удалить категорию "{deletingCategory?.title}"?
                                </p>
                                <div className="modal-actions">
                                    <Button onClick={handleDeleteCategory}>
                                        Удалить
                                    </Button>
                                    <Button
                                        variant="outline"
                                        onClick={() => setDeletingCategory(null)}
                                    >
                                        Отмена
                                    </Button>
                                </div>
                            </div>
                        </Modal>
                    </div>
                )}

                {activeTab === 'users' && (
                    <div>
                        <h2>Управление пользователями</h2>
                        <div className="users-list">
                            {getCurrentPageItems(
                                users.filter(user => user.user_id !== userId), // Фильтруем текущего пользователя
                                usersPage
                            ).map(user => (
                                <div key={user.user_id} className="user-item">
                                    <Link to={`/users/${user.user_id}`} className="user-item__name">
                                        {user.username}
                                    </Link>
                                    <div className="user-item__actions">
                                        <Button
                                            variant="outline"
                                            onClick={() => setEditingUser(user)}
                                        >
                                            Обновить
                                        </Button>
                                        <Button
                                            variant="outline"
                                            onClick={() => setDeletingUser(user)}
                                        >
                                            Удалить
                                        </Button>
                                    </div>
                                </div>
                            ))}
                        </div>
                        {users.length > itemsPerPage && (
                            <Pagination
                                currentPage={usersPage}
                                totalPages={Math.ceil((users.length - 1) / itemsPerPage)} // Учитываем удаление текущего пользователя
                                onPageChange={setUsersPage}
                            />
                        )}

                        <UserForm
                            isOpen={!!editingUser}
                            onClose={() => setEditingUser(null)}
                            onSubmit={handleEditUser}
                            initialData={editingUser}
                        />

                        <Modal
                            isOpen={!!deletingUser}
                            onClose={() => setDeletingUser(null)}
                            title="Подтверждение удаления"
                        >
                            <div className="delete-confirmation">
                                <p>Вы уверены, что хотите удалить пользователя "{deletingUser?.username}"?</p>
                                <div className="modal-actions">
                                    <Button onClick={handleDeleteUser}>
                                        Удалить
                                    </Button>
                                    <Button
                                        variant="outline"
                                        onClick={() => setDeletingUser(null)}
                                    >
                                        Отмена
                                    </Button>
                                </div>
                            </div>
                        </Modal>
                    </div>
                )}

                {activeTab === 'reviews' && (
                    <div>
                        <h2>Управление отзывами</h2>
                        <div className="reviews-list">
                            {getCurrentPageItems(reviews, reviewsPage).map(review => (
                                <div key={review.review_id} className="review-item">
                                    <div className="review-item__info">
                                        <div className="review-item__users">
                                            <span>От: <Link to={`/users/${review.sender_id}`}>{review.senderName}</Link></span>
                                            <span>Кому: <Link to={`/users/${review.receiver_id}`}>{review.receiverName}</Link></span>
                                        </div>
                                        <div className="review-item__details">
                                            <span className="review-item__skill">Навык: {review.skillTitle}</span>
                                            <span className="review-item__rating">Рейтинг: {review.rating} ★</span>
                                        </div>
                                        <p className="review-item__comment">{review.comment}</p>
                                    </div>
                                    <div className="review-item__actions">
                                        <Button
                                            variant="outline"
                                            onClick={() => setEditingReview(review)}
                                        >
                                            Изменить
                                        </Button>
                                        <Button
                                            variant="outline"
                                            onClick={() => setDeletingReview(review)}
                                        >
                                            Удалить
                                        </Button>
                                    </div>
                                </div>
                            ))}
                        </div>
                        {reviews.length > itemsPerPage && (
                            <Pagination
                                currentPage={reviewsPage}
                                totalPages={Math.ceil(reviews.length / itemsPerPage)}
                                onPageChange={setReviewsPage}
                            />
                        )}

                        <AdminReviewForm
                            isOpen={!!editingReview}
                            onClose={() => setEditingReview(null)}
                            onSubmit={handleUpdateReview}
                            initialData={editingReview}
                        />

                        <Modal
                            isOpen={!!deletingReview}
                            onClose={() => setDeletingReview(null)}
                            title="Подтверждение удаления"
                        >
                            <div className="delete-confirmation">
                                <p>Вы уверены, что хотите удалить этот отзыв?</p>
                                <div className="modal-actions">
                                    <Button onClick={handleDeleteReview}>
                                        Удалить
                                    </Button>
                                    <Button
                                        variant="outline"
                                        onClick={() => setDeletingReview(null)}
                                    >
                                        Отмена
                                    </Button>
                                </div>
                            </div>
                        </Modal>
                    </div>
                )}


            </div>

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