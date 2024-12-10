import { useState, useEffect } from 'react'
import { Modal } from '../Modal/Modal'
import { Button } from '../Button/Button'
import './CategoryForm.css'

export const CategoryForm = ({ isOpen, onClose, onSubmit, initialData = null }) => {
    const [form, setForm] = useState({
        title: '',
        description: ''
    })

    useEffect(() => {
        if (initialData) {
            setForm({
                title: initialData.title,
                description: initialData.description
            })
        }
    }, [initialData])

    const handleSubmit = (e) => {
        e.preventDefault()
        onSubmit(form)
        setForm({ title: '', description: '' })
    }

    return (
        <Modal
            isOpen={isOpen}
            onClose={onClose}
            title={initialData ? "Изменить категорию" : "Добавить категорию"}
        >
            <form onSubmit={handleSubmit} className="category-form">
                <div className="category-form__group">
                    <label>Название:</label>
                    <input
                        type="text"
                        value={form.title}
                        onChange={(e) => setForm({ ...form, title: e.target.value })}
                        required
                    />
                </div>

                <div className="category-form__group">
                    <label>Описание:</label>
                    <textarea
                        value={form.description}
                        onChange={(e) => setForm({ ...form, description: e.target.value })}
                        required
                    />
                </div>

                <div className="category-form__buttons">
                    <Button type="submit">
                        {initialData ? "Сохранить" : "Добавить"}
                    </Button>
                    <Button type="button" variant="outline" onClick={onClose}>
                        Отменить
                    </Button>
                </div>
            </form>
        </Modal>
    )
}