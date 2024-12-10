import { useState } from 'react'
import './Tooltip.css'

export const Tooltip = ({ children, text }) => {
    const [isVisible, setIsVisible] = useState(false)

    return (
        <div 
            className="tooltip-container"
            onMouseEnter={() => setIsVisible(true)}
            onMouseLeave={() => setIsVisible(false)}
        >
            {children}
            {isVisible && (
                <div className="tooltip-text">
                    {text}
                </div>
            )}
        </div>
    )
}