import { Link } from 'react-router-dom'
import { Plus } from 'lucide-react'

/**
 * HU 14.1: Ver lista de todos los Policy Stores
 * Muestra una tabla con todos los Policy Stores existentes
 */
export default function PolicyStoresPage() {
  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-gray-900">Policy Stores</h1>
          <p className="text-gray-600 mt-1">Gestiona los almacenes de políticas de tu aplicación</p>
        </div>
        <Link
          to="/policy-stores/create"
          className="flex items-center gap-2 bg-blue-600 text-white px-4 py-2 rounded-lg hover:bg-blue-700 transition-colors"
        >
          <Plus className="w-5 h-5" />
          Crear Policy Store
        </Link>
      </div>

      {/* TODO: Implementar tabla de Policy Stores */}
      <div className="bg-white rounded-lg shadow p-6">
        <p className="text-gray-600">Cargando Policy Stores...</p>
      </div>
    </div>
  )
}
