import { Link } from 'react-router-dom'
import { Button } from '../Button/Button'
import { Tooltip } from '../Tooltip/Tooltip'
import './TeacherCard.css'

export const TeacherCard = ({ teacher, skillId, onRequestClick }) => {
    return (
        <div className="teacher-card">
            <Link to={`/users/${teacher.user_id}`} className="teacher-card__name">
                {teacher.username}
            </Link>
            <p className="teacher-card__bio">
                {teacher.bio || 'Биография отсутствует'}
            </p>
            <div className="teacher-card__ratings">
                <div className="teacher-card__rating">
                    Общий рейтинг: {teacher.average_rating ? `${teacher.average_rating} ★` : '—'}
                </div>
                <Tooltip text="Рейтинг преподавания по выбранному навыку">
                    <div className="teacher-card__skill-rating">
                        Рейтинг навыка: {teacher.skill_rating ? `${teacher.skill_rating} ★` : '—'}
                    </div>
                </Tooltip>
            </div>
            <Button onClick={onRequestClick} className="teacher-card__button">
                Отправить запрос
            </Button>
        </div>
    )
}