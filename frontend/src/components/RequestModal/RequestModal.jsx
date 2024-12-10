import { useState } from 'react'
import { Modal } from '../Modal/Modal'
import { Button } from '../Button/Button'
import './RequestModal.css'

export const RequestModal = ({ isOpen, onClose, onSubmit }) => {
    const [coverLetter, setCoverLetter] = useState('')

    const handleSubmit = (e) => {
        e.preventDefault()
        onSubmit(coverLetter)
        setCoverLetter('')
    }

    return (
        <Modal
            isOpen={isOpen}
            onClose={onClose}
            title="Отправка запроса"
        >
            <form onSubmit={handleSubmit} className="request-form">
                <textarea
                    className="request-form__input"
                    value={coverLetter}
                    onChange={(e) => setCoverLetter(e.target.value)}
                    placeholder="Введите сопроводительное письмо..."
                    required
                />
                <div className="request-form__buttons">
                    <Button type="submit">
                        Подтвердить
                    </Button>
                    <Button 
                        type="button" 
                        variant="outline" 
                        onClick={onClose}
                    >
                        Отменить
                    </Button>
                </div>
            </form>
        </Modal>
    )
}