import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom'
import Layout from '@components/common/Layout'
import PolicyStoresPage from '@features/policy-stores/pages/PolicyStoresPage'
import PolicyStoreDetailPage from '@features/policy-stores/pages/PolicyStoreDetailPage'
import CreatePolicyStorePage from '@features/policy-stores/pages/CreatePolicyStorePage'
import SchemaEditorPage from '@features/schema-editor/pages/SchemaEditorPage'
import PoliciesPage from '@features/policy-editor/pages/PoliciesPage'
import CreatePolicyPage from '@features/policy-editor/pages/CreatePolicyPage'
import EditPolicyPage from '@features/policy-editor/pages/EditPolicyPage'
import PlaygroundPage from '@features/playground/pages/PlaygroundPage'

function App() {
  return (
    <Router>
      <Layout>
        <Routes>
          {/* Policy Stores - Épica 14 */}
          <Route path="/" element={<PolicyStoresPage />} />
          <Route path="/policy-stores" element={<PolicyStoresPage />} />
          <Route path="/policy-stores/create" element={<CreatePolicyStorePage />} />
          <Route path="/policy-stores/:storeId" element={<PolicyStoreDetailPage />} />

          {/* Schema Editor - Épica 15 */}
          <Route path="/policy-stores/:storeId/schema" element={<SchemaEditorPage />} />

          {/* Policy Editor - Épica 16 */}
          <Route path="/policy-stores/:storeId/policies" element={<PoliciesPage />} />
          <Route path="/policy-stores/:storeId/policies/create" element={<CreatePolicyPage />} />
          <Route path="/policy-stores/:storeId/policies/:policyId/edit" element={<EditPolicyPage />} />

          {/* Playground - Épica 17 */}
          <Route path="/policy-stores/:storeId/playground" element={<PlaygroundPage />} />

          {/* Catch all */}
          <Route path="*" element={<Navigate to="/" replace />} />
        </Routes>
      </Layout>
    </Router>
  )
}

export default App
