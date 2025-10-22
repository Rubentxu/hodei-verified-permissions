import { Shield } from 'lucide-react'

export default function Header() {
  return (
    <header className="bg-white border-b border-gray-200 px-6 py-4 shadow-sm">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <Shield className="w-6 h-6 text-blue-600" />
          <h1 className="text-2xl font-bold text-gray-900">Hodei Verified Permissions</h1>
        </div>
        <div className="text-sm text-gray-600">
          v1.0.0
        </div>
      </div>
    </header>
  )
}
