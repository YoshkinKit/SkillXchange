import { useState, useEffect } from 'react'
import { Modal } from '../Modal/Modal'
import { Button } from '../Button/Button'
import './AdminReviewForm.css'

export const AdminReviewForm = ({ isOpen, onClose, onSubmit, initialData = null }) => {
    const [form, setForm] = useState({
        rating: 5,
        comment: ''
    })

    useEffect(() => {
        if (initialData) {
            setForm({
                rating: initialData.rating,
                comment: initialData.comment
            })
        }
    }, [initialData])

    const handleSubmit = (e) => {
        e.preventDefault()
        onSubmit(form)
    }

    return (
        <Modal
            isOpen={isOpen}
            onClose={onClose}
            title="Редактировать отзыв"
        >
            <form onSubmit={handleSubmit} className="admin-review-form">
                <div className="admin-review-form__group">
                    <label>Рейтинг:</label>
                    <select
                        value={form.rating}
                        onChange={(e) => setForm({ ...form, rating: Number(e.target.value) })}
                        required
                    >
                        {[5, 4, 3, 2, 1].map(num => (
                            <option key={num} value={num}>{num} ★</option>
                        ))}
                    </select>
                </div>

                <div className="admin-review-form__group">
                    <label>Комментарий:</label>
                    <textarea
                        value={form.comment}
                        onChange={(e) => setForm({ ...form, comment: e.target.value })}
                        required
                    />
                </div>

                <div className="admin-review-form__buttons">
                    <Button type="submit">Сохранить</Button>
                    <Button type="button" variant="outline" onClick={onClose}>
                        Отменить
                    </Button>
                </div>
            </form>
        </Modal>
    )
}