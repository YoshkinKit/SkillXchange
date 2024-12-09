import './Input.css'

export const Input = ({ 
  label,
  type = 'text',
  value,
  onChange,
  error,
  ...props
}) => {
  return (
    <div className="input-wrapper">
      <label className="input-label">{label}</label>
      <input
        type={type}
        value={value}
        onChange={onChange}
        className={`input ${error ? 'input--error' : ''}`}
        {...props}
      />
      {error && <span className="input-error">{error}</span>}
    </div>
  )
}