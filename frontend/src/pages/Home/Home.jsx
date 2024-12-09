import { useNavigate } from 'react-router-dom'
import { Button } from '../../components/Button/Button'
import './Home.css'

export const Home = () => {
  const navigate = useNavigate()

  return (
    <div className="home">
      <div className="home__content">
        <h1>Делитесь знаниями, получайте навыки</h1>
        <p>
          SkillXChange - это платформа, где люди могут обмениваться своими навыками.
          Научите кого-то тому, что умеете сами, и научитесь чему-то новому взамен!
        </p>
        <Button onClick={() => navigate('/skills')}>
          Обучиться новому
        </Button>
      </div>
    </div>
  )
}