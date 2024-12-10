import './Button.css'

export const Button = ({ 
  children, 
  variant = 'default', 
  onClick,
  className = '',
  type = 'button',
  disabled = false
}) => {
  return (
    <button
      type={type}
      onClick={onClick}
      className={`button button--${variant} ${className}`}
      disabled={disabled}
    >
      {children}
    </button>
  )
}