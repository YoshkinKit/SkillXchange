import { useState, useEffect } from 'react'
import { SkillCard } from '../../components/SkillCard/SkillCard'
import { AuthModal } from '../../components/AuthModal/AuthModal'
import './Skills.css'

export const Skills = () => {
    const [skills, setSkills] = useState([])
    const [categories, setCategories] = useState([])
    const [selectedCategory, setSelectedCategory] = useState('all')
    const [isAuthModalOpen, setIsAuthModalOpen] = useState(false)
    const [loading, setLoading] = useState(true)
    const [error, setError] = useState(null)

    useEffect(() => {
        const fetchData = async () => {
            try {
                const [skillsResponse, categoriesResponse] = await Promise.all([
                    fetch('/api/skills'),
                    fetch('/api/categories')
                ])

                if (!skillsResponse.ok || !categoriesResponse.ok) {
                    throw new Error('Ошибка загрузки данных')
                }

                const [skillsData, categoriesData] = await Promise.all([
                    skillsResponse.json(),
                    categoriesResponse.json()
                ])

                setSkills(skillsData)
                setCategories(categoriesData)
            } catch (error) {
                setError(error.message)
            } finally {
                setLoading(false)
            }
        }

        fetchData()
    }, [])

    const filteredSkills = selectedCategory === 'all'
        ? skills
        : skills.filter(skill => skill.category_id === parseInt(selectedCategory))

    // Группируем навыки по категориям
    const groupedSkills = filteredSkills.reduce((acc, skill) => {
        const category = categories.find(c => c.category_id === skill.category_id)
        if (!category) return acc

        if (!acc[category.title]) {
            acc[category.title] = []
        }
        acc[category.title].push(skill)
        return acc
    }, {})

    if (loading) return <div className="skills__loading">Загрузка...</div>
    if (error) return <div className="skills__error">Ошибка: {error}</div>

    return (
        <div className="skills">
            <div className="skills__filter">
                <select
                    value={selectedCategory}
                    onChange={(e) => setSelectedCategory(e.target.value)}
                    className="skills__select"
                >
                    <option value="all">Все категории</option>
                    {categories.map(category => (
                        <option 
                            key={category.category_id} 
                            value={category.category_id}
                        >
                            {category.title}
                        </option>
                    ))}
                </select>
            </div>

            <div className="skills__list">
                {Object.entries(groupedSkills).map(([category, skills]) => (
                    <div key={category} className="skills__category">
                        <h2 className="skills__category-title">{category}</h2>
                        <div className="skills__grid">
                            {skills.map(skill => (
                                <SkillCard
                                    key={skill.skill_id}
                                    skill={skill}
                                    onAuthModalOpen={() => setIsAuthModalOpen(true)}
                                />
                            ))}
                        </div>
                    </div>
                ))}
            </div>

            <AuthModal
                isOpen={isAuthModalOpen}
                onClose={() => setIsAuthModalOpen(false)}
            />
        </div>
    )
}