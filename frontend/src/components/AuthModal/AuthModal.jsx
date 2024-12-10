import { useNavigate } from 'react-router-dom'
import { Modal } from '../Modal/Modal'
import { Button } from '../Button/Button'
import './AuthModal.css'

export const AuthModal = ({ isOpen, onClose }) => {
    const navigate = useNavigate()

    return (
        <Modal 
            isOpen={isOpen} 
            onClose={onClose}
            title="Требуется авторизация"
        >
            <div className="auth-modal">
                <p className="auth-modal__text">
                    Для доступа к этой функции необходимо войти в систему или зарегистрироваться
                </p>
                <div className="auth-modal__buttons">
                    <Button onClick={() => navigate('/login')}>
                        Войти
                    </Button>
                    <Button 
                        variant="outline" 
                        onClick={() => navigate('/register')}
                    >
                        Зарегистрироваться
                    </Button>
                </div>
            </div>
        </Modal>
    )
}