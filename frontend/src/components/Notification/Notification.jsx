import { useEffect } from 'react'
import './Notification.css'

export const Notification = ({ message, type = 'success', onClose }) => {
  useEffect(() => {
    const timer = setTimeout(onClose, 3000)
    return () => clearTimeout(timer)
  }, [onClose])

  return (
    <div className={`notification notification--${type}`}>
      {message}
    </div>
  )
}