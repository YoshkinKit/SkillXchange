import { useState } from 'react'
import { Modal } from '../Modal/Modal'
import { Button } from '../Button/Button'
import './ReviewForm.css'

export const ReviewForm = ({ isOpen, onClose, onSubmit, skills }) => {
    const [form, setForm] = useState({
        rating: 5,
        skill_id: '',
        comment: ''
    })

    const handleSubmit = (e) => {
        e.preventDefault()
        onSubmit(form)
        setForm({ rating: 5, skill_id: '', comment: '' })
    }

    return (
        <Modal
            isOpen={isOpen}
            onClose={onClose}
            title="Написать отзыв"
        >
            <form onSubmit={handleSubmit} className="review-form">
                <div className="review-form__rating">
                    <label>Рейтинг:</label>
                    <select
                        value={form.rating}
                        onChange={(e) => setForm({ ...form, rating: Number(e.target.value) })}
                    >
                        {[5, 4, 3, 2, 1].map(num => (
                            <option key={num} value={num}>{num} ★</option>
                        ))}
                    </select>
                </div>

                <div className="review-form__skill">
                    <label>Навык:</label>
                    <select
                        value={form.skill_id}
                        onChange={(e) => setForm({ ...form, skill_id: e.target.value })}
                        required
                    >
                        <option value="">Выберите навык</option>
                        {skills.map(skill => (
                            <option key={skill.skill_id} value={skill.skill_id}>
                                {skill.title}
                            </option>
                        ))}
                    </select>
                </div>

                <div className="review-form__comment">
                    <label>Комментарий:</label>
                    <textarea
                        value={form.comment}
                        onChange={(e) => setForm({ ...form, comment: e.target.value })}
                        required
                    />
                </div>

                <div className="review-form__buttons">
                    <Button type="submit">Подтвердить</Button>
                    <Button type="button" variant="outline" onClick={onClose}>
                        Отменить
                    </Button>
                </div>
            </form>
        </Modal>
    )
}