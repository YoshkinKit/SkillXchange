import catImage from '../../assets/Работяга.jpg'
import './Chat.css'

export const Chat = () => {
    return (
        <div className="chat-placeholder">
            <img 
                src={catImage} 
                alt="Working cat" 
                className="chat-placeholder__image"
            />
            <p className="chat-placeholder__text">
                Этот раздел находится в разработке
            </p>
        </div>
    )
}