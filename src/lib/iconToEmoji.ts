function iconToEmoji(icon: string): string {
  const map: Record<string, string> = {
    Layers: '📚', FolderOpen: '📁', Star: '⭐', Heart: '❤️', Zap: '⚡',
    Compass: '🧭', Target: '🎯', Award: '🏆',
    GraduationCap: '🎓', BookOpen: '📖', PenTool: '✒️', Calculator: '🔢',
    FlaskConical: '🧪', Languages: '🗣️', Microscope: '🔬', Library: '📚',
    Briefcase: '💼', Building2: '🏢', Presentation: '📊', LineChart: '📈',
    Calendar: '📅', Mail: '📧', Users: '👥', MessageSquare: '💬',
    Home: '🏠', Dumbbell: '🏋️', Utensils: '🍽️', Plane: '✈️',
    Music: '🎵', Camera: '📷', Gamepad2: '🎮', ShoppingBag: '🛍️',
    Palette: '🎨', Pencil: '✏️', Film: '🎬', Code2: '💻',
    Globe: '🌐', Sparkles: '✨', Gem: '💎', Crown: '👑',
  }
  return map[icon] || '📦'
}

export { iconToEmoji }
