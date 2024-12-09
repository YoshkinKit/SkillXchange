import './Button.css'

export const Button = ({ 
  children, 
  variant = 'default', 
  onClick,
  className = '',
  type = 'button'
}) => {
  return (
    <button
      type={type}
      onClick={onClick}
      className={`button button--${variant} ${className}`}
    >
      {children}
    </button>
  )
}