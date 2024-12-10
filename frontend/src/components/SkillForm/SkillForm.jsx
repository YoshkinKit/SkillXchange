import { useState, useEffect } from 'react'
import { Modal } from '../Modal/Modal'
import { Button } from '../Button/Button'
import './SkillForm.css'

export const SkillForm = ({ 
    isOpen, 
    onClose, 
    onSubmit, 
    categories,
    initialData = null
}) => {
    const [form, setForm] = useState({
        title: '',
        description: '',
        category_id: ''
    })

    useEffect(() => {
        if (initialData) {
            setForm({
                title: initialData.title,
                description: initialData.description,
                category_id: initialData.category_id.toString()
            })
        }
    }, [initialData])

    const handleSubmit = (e) => {
        e.preventDefault()
        const formData = {
            ...form,
            category_id: Number(form.category_id)
        }
        onSubmit(formData)
        setForm({ title: '', description: '', category_id: '' })
    }

    return (
        <Modal
            isOpen={isOpen}
            onClose={onClose}
            title={initialData ? "Изменить навык" : "Добавить навык"}
        >
            <form onSubmit={handleSubmit} className="skill-form">
                <div className="skill-form__group">
                    <label>Название:</label>
                    <input
                        type="text"
                        value={form.title}
                        onChange={(e) => setForm({ ...form, title: e.target.value })}
                        required
                    />
                </div>

                <div className="skill-form__group">
                    <label>Описание:</label>
                    <textarea
                        value={form.description}
                        onChange={(e) => setForm({ ...form, description: e.target.value })}
                        required
                    />
                </div>

                <div className="skill-form__group">
                    <label>Категория:</label>
                    <select
                        value={form.category_id}
                        onChange={(e) => setForm({ ...form, category_id: e.target.value })}
                        required
                    >
                        <option value="">Выберите категорию</option>
                        {categories.map(category => (
                            <option key={category.category_id} value={category.category_id}>
                                {category.title}
                            </option>
                        ))}
                    </select>
                </div>

                <div className="skill-form__buttons">
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