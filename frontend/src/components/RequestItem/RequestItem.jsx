import { Button } from '../Button/Button'
import './RequestItem.css'

export const RequestItem = ({ request, type, onAccept, onDecline }) => {
    const getStatusClass = (status) => {
        switch (status) {
            case 'pending': return 'status--pending'
            case 'accept': return 'status--accepted'
            case 'decline': return 'status--declined'
            default: return ''
        }
    }

    const getStatusText = (status, type) => {
        if (type === 'incoming') {
            switch (status) {
                case 'pending': return 'Пользователь ожидает вашего ответа'
                case 'accept': return 'Вы приняли этот запрос'
                case 'decline': return 'Вы отклонили этот запрос'
                default: return status
            }
        } else { // outgoing
            switch (status) {
                case 'pending': return 'В ожидании'
                case 'accept': return 'Запрос принят'
                case 'decline': return 'Запрос отклонен'
                default: return status
            }
        }
    }

    return (
        <div className="request">
            <div className="request__info">
                <div className="request__header">
                    <span className="request__user">
                        {type === 'incoming' ? 'От: ' : 'Кому: '}
                        {request.username}
                    </span>
                    <span className="request__skill">Навык: {request.skillTitle}</span>
                </div>
                <p className="request__letter">{request.cover_letter}</p>
                <span className={`request__status ${getStatusClass(request.status)}`}>
                    {getStatusText(request.status, type)}
                </span>
            </div>
            {type === 'incoming' && request.status === 'pending' && (
                <div className="request__actions">
                    <Button onClick={() => onAccept(request.request_id)}>
                        Принять
                    </Button>
                    <Button 
                        variant="outline"
                        onClick={() => onDecline(request.request_id)}
                    >
                        Отклонить
                    </Button>
                </div>
            )}
        </div>
    )
}