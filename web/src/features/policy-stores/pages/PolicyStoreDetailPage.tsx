import { useParams, Link } from 'react-router-dom'
import { ArrowLeft } from 'lucide-react'

/**
 * HU 14.3: Ver detalles de un Policy Store
 * Muestra los detalles y permite navegar a secciones (Schema, Políticas, etc.)
 */
export default function PolicyStoreDetailPage() {
  const { storeId } = useParams<{ storeId: string }>()

  return (
    <div className="space-y-6">
      <Link to="/policy-stores" className="flex items-center gap-2 text-blue-600 hover:text-blue-700">
        <ArrowLeft className="w-4 h-4" />
        Volver a Policy Stores
      </Link>

      <div>
        <h1 className="text-3xl font-bold text-gray-900">Policy Store: {storeId}</h1>
        <p className="text-gray-600 mt-1">Gestiona todos los recursos de este Policy Store</p>
      </div>

      {/* Tabs de navegación */}
      <div className="border-b border-gray-200">
        <div className="flex gap-8">
          <Link
            to={`/policy-stores/${storeId}/schema`}
            className="px-4 py-3 border-b-2 border-blue-600 text-blue-600 font-medium"
          >
            Schema
          </Link>
          <Link
            to={`/policy-stores/${storeId}/policies`}
            className="px-4 py-3 border-b-2 border-transparent text-gray-600 hover:text-gray-900"
          >
            Políticas
          </Link>
          <Link
            to={`/policy-stores/${storeId}/playground`}
            className="px-4 py-3 border-b-2 border-transparent text-gray-600 hover:text-gray-900"
          >
            Playground
          </Link>
        </div>
      </div>

      {/* TODO: Implementar contenido de tabs */}
      <div className="bg-white rounded-lg shadow p-6">
        <p className="text-gray-600">Cargando detalles del Policy Store...</p>
      </div>
    </div>
  )
}
