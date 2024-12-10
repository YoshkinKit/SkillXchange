import { useNavigate } from 'react-router-dom'
import { Button } from '../Button/Button'
import { useAuthContext } from '../../contexts/AuthContext'
import './SkillCard.css'

export const SkillCard = ({ skill, onAuthModalOpen }) => {
    const navigate = useNavigate()
    const { isAuth } = useAuthContext()

    const handleUsersClick = () => {
        if (isAuth) {
            navigate(`/skills/${skill.skill_id}/users`)
        } else {
            onAuthModalOpen()
        }
    }

    return (
        <div className="skill-card">
            <h3 className="skill-card__title">{skill.title}</h3>
            <p className="skill-card__description">{skill.description}</p>
            <Button 
                variant="outline" 
                onClick={handleUsersClick}
                className="skill-card__button"
            >
                Найти учителей
            </Button>
        </div>
    )
}