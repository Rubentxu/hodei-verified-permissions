import { Link, useLocation } from 'react-router-dom'
import { Store, FileJson, FileText, Zap } from 'lucide-react'
import clsx from 'clsx'

const menuItems = [
  { label: 'Policy Stores', href: '/policy-stores', icon: Store },
  { label: 'Schema', href: '/schema', icon: FileJson },
  { label: 'Policies', href: '/policies', icon: FileText },
  { label: 'Playground', href: '/playground', icon: Zap },
]

export default function Sidebar() {
  const location = useLocation()

  return (
    <aside className="w-64 bg-gray-900 text-white flex flex-col">
      <div className="p-6 border-b border-gray-800">
        <h2 className="text-lg font-semibold">Navigation</h2>
      </div>
      <nav className="flex-1 px-4 py-6 space-y-2">
        {menuItems.map((item) => {
          const Icon = item.icon
          const isActive = location.pathname.startsWith(item.href)
          return (
            <Link
              key={item.href}
              to={item.href}
              className={clsx(
                'flex items-center gap-3 px-4 py-3 rounded-lg transition-colors',
                isActive
                  ? 'bg-blue-600 text-white'
                  : 'text-gray-300 hover:bg-gray-800'
              )}
            >
              <Icon className="w-5 h-5" />
              <span>{item.label}</span>
            </Link>
          )
        })}
      </nav>
      <div className="p-4 border-t border-gray-800 text-xs text-gray-400">
        <p>Hodei v1.0.0</p>
      </div>
    </aside>
  )
}
