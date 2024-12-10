import "./Review.css"

export const Review = ({ review }) => {
    return (
        <div className="review">
            <div className="review__header">
                <div className="review__info">
                    <span className="review__author">{review.username}</span>
                    <span className="review__skill">Навык: {review.skillTitle}</span>
                </div>
                <span className="review__rating">{review.rating} ★</span>
            </div>
            <p className="review__text">{review.comment}</p>
        </div>
    )
}