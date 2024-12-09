import { useState, useEffect } from 'react'
import { useNavigate } from 'react-router-dom'
import { useAuthContext } from '../../contexts/AuthContext'
import { useProfile } from '../../hooks/useProfile'
import { Modal } from '../../components/Modal/Modal'
import { Button } from '../../components/Button/Button'
import { Notification } from '../../components/Notification/Notification'
import './Profile.css'

export const Profile = () => {
    const navigate = useNavigate()
    const { userId } = useAuthContext()
    const {
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
    } = useProfile(userId)

    // Все useState хуки вместе
    const [activeTab, setActiveTab] = useState('profile')
    const [editForm, setEditForm] = useState({
        username: '',
        email: '',
        bio: ''
    })
    const [selectedSkill, setSelectedSkill] = useState('')
    const [showDeleteModal, setShowDeleteModal] = useState(false)
    const [notification, setNotification] = useState(null)

    // Все useEffect хуки вместе
    useEffect(() => {
        if (profile) {
            setEditForm({
                username: profile.username,
                email: profile.email,
                bio: profile.bio || ''
            })
        }
    }, [profile])

    useEffect(() => {
        if (activeTab === 'skills') {
            fetchSkills()
        }
    }, [activeTab])

    // Определяем все функции после хуков
    const showNotification = (message, type = 'success') => {
        setNotification({ message, type })
    }

    const handleProfileUpdate = async (e) => {
        e.preventDefault()
        const success = await updateProfile(editForm)
        if (success) {
            showNotification('Профиль успешно обновлен')
            setEditForm({
                username: profile.username,
                email: profile.email,
                bio: profile.bio || ''
            })
        } else {
            showNotification('Ошибка при обновлении профиля', 'error')
        }
    }

    const handleAddSkill = async () => {
        if (!selectedSkill) return
        try {
            const success = await addSkill(Number(selectedSkill))
            if (success) {
                showNotification('Навык успешно добавлен')
                setSelectedSkill('')
            }
        } catch (error) {
            showNotification('Ошибка при добавлении навыка', 'error')
        }
    }

    const handleDeleteSkill = async (skillId) => {
        try {
            const success = await deleteSkill(skillId)
            if (success) {
                showNotification('Навык успешно удален')
            }
        } catch (error) {
            showNotification('Ошибка при удалении навыка', 'error')
        }
    }

    const handleDeleteAccount = async () => {
        try {
            const success = await deleteAccount()
            if (success) {
                showNotification('Аккаунт успешно удален')
                setTimeout(() => navigate('/'), 3000)
            }
        } catch (error) {
            showNotification('Ошибка при удалении аккаунта', 'error')
        }
    }

    // Ранний возврат для состояний загрузки и ошибки
    if (loading) return <div>Загрузка...</div>
    if (error) return <div>Ошибка: {error}</div>

    return (
        <div className="profile-page">
            <div className="profile-sidebar">
                <Button
                    variant={activeTab === 'profile' ? 'default' : 'outline'}
                    onClick={() => setActiveTab('profile')}
                >
                    Профиль
                </Button>
                <Button
                    variant={activeTab === 'skills' ? 'default' : 'outline'}
                    onClick={() => setActiveTab('skills')}
                >
                    Мои навыки
                </Button>
                <Button
                    variant={activeTab === 'danger' ? 'default' : 'outline'}
                    onClick={() => setActiveTab('danger')}
                >
                    Опасная зона
                </Button>

                {notification && (
                    <Notification
                        message={notification.message}
                        type={notification.type}
                        onClose={() => setNotification(null)}
                    />
                )}
            </div>

            <div className="profile-content">
                {activeTab === 'profile' && (
                    <form onSubmit={handleProfileUpdate} className="profile-form">
                        <h2>Профиль</h2>

                        <div className="form-group">
                            <label>Имя пользователя</label>
                            <input
                                type="text"
                                value={editForm.username || profile?.username || ''}
                                onChange={e => setEditForm({ ...editForm, username: e.target.value })}
                            />
                        </div>

                        <div className="form-group">
                            <label>Email</label>
                            <input
                                type="email"
                                value={editForm.email || profile?.email || ''}
                                onChange={e => setEditForm({ ...editForm, email: e.target.value })}
                            />
                        </div>

                        <div className="form-group">
                            <label>О себе</label>
                            <textarea
                                value={editForm.bio || profile?.bio || ''}
                                onChange={e => setEditForm({ ...editForm, bio: e.target.value })}
                            />
                        </div>

                        <Button type="submit">Обновить профиль</Button>
                    </form>
                )}

                {activeTab === 'skills' && (
                    <div className="skills-section">
                        <h2>Мои навыки</h2>

                        <div className="skills-list">
                            {skills.map(skill => (
                                <div key={skill.skill_id} className="skill-item">
                                    <span>{skill.title}</span>
                                    <Button
                                        variant="outline"
                                        onClick={() => handleDeleteSkill(skill.skill_id)}
                                    >
                                        Удалить
                                    </Button>
                                </div>
                            ))}
                        </div>

                        <div className="add-skill">
                            <select
                                value={selectedSkill}
                                onChange={e => setSelectedSkill(e.target.value)}
                            >
                                <option key="default" value="">Выберите навык</option>
                                {allSkills
                                    .filter(skill => !skills.some(s => s.skill_id === skill.skill_id))
                                    .map(skill => (
                                        <option key={skill.skill_id} value={skill.skill_id}>
                                            {skill.title}
                                        </option>
                                    ))
                                }
                            </select>
                            <Button
                                onClick={() => handleAddSkill()}
                            >
                                Добавить навык
                            </Button>
                        </div>
                    </div>
                )}

                {activeTab === 'danger' && (
                    <div className="danger-zone">
                        <h2>Опасная зона</h2>
                        <p>Внимание! Эти действия необратимы!</p>

                        <Button
                            variant="danger"
                            onClick={() => setShowDeleteModal(true)}
                        >
                            Удалить аккаунт
                        </Button>
                    </div>
                )}
            </div>

            <Modal
                isOpen={showDeleteModal}
                onClose={() => setShowDeleteModal(false)}
                title="Подтвердите действие"
            >
                <div className="delete-confirmation">
                    <p>Вы уверены, что хотите удалить свой аккаунт? Это действие необратимо!</p>
                    <div className="modal-actions">
                        <Button variant="outline" onClick={() => setShowDeleteModal(false)}>
                            Отмена
                        </Button>
                        <Button variant="danger" onClick={handleDeleteAccount}>
                            Удалить аккаунт
                        </Button>
                    </div>
                </div>
            </Modal>
        </div>
    )
}