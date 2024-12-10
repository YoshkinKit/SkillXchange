import { useState, useEffect } from 'react'
import { Modal } from '../Modal/Modal'
import { Button } from '../Button/Button'
import './UserForm.css'

export const UserForm = ({ isOpen, onClose, onSubmit, initialData = null }) => {
    const [form, setForm] = useState({
        username: '',
        bio: ''
    })

    useEffect(() => {
        if (initialData) {
            setForm({
                username: initialData.username,
                bio: initialData.bio || ''
            })
        }
    }, [initialData])

    const handleSubmit = (e) => {
        e.preventDefault()
        onSubmit(form)
        setForm({ username: '', bio: '' })
    }

    return (
        <Modal
            isOpen={isOpen}
            onClose={onClose}
            title="Редактировать пользователя"
        >
            <form onSubmit={handleSubmit} className="user-form">
                <div className="user-form__group">
                    <label>Имя пользователя:</label>
                    <input
                        type="text"
                        value={form.username}
                        onChange={(e) => setForm({ ...form, username: e.target.value })}
                        required
                    />
                </div>

                <div className="user-form__group">
                    <label>Биография:</label>
                    <textarea
                        value={form.bio}
                        onChange={(e) => setForm({ ...form, bio: e.target.value })}
                    />
                </div>

                <div className="user-form__buttons">
                    <Button type="submit">Сохранить</Button>
                    <Button type="button" variant="outline" onClick={onClose}>
                        Отменить
                    </Button>
                </div>
            </form>
        </Modal>
    )
}