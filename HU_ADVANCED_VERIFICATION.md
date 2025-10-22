# 🔍 VERIFICACIÓN DE HISTORIAS DE USUARIO AVANZADAS
## Hodei Verified Permissions - Épicas 4-9

**Fecha:** 22 de Octubre de 2025, 19:30  
**Documento Base:** `docs/historias-usuario-2.md`  
**Método:** Verificación de código + Investigación AWS AVP + Tests

---

## 📚 CONTEXTO: AWS VERIFIED PERMISSIONS

### Funcionalidades Clave de AWS AVP

Basado en la documentación oficial de AWS:

1. **Cedar Policy Language v2.4**
   - Lenguaje de políticas de código abierto
   - Separación de lógica de negocio y autorización
   - Decisiones: ALLOW o DENY

2. **Identity Sources**
   - Integración con Amazon Cognito
   - Soporte para OIDC genérico
   - Mapeo de claims JWT a entidades Cedar
   - Validación de tokens (firma, issuer, audience, expiración)

3. **Batch Operations**
   - Múltiples decisiones de autorización en una llamada
   - Optimizado para renderizado de UI
   - Reduce latencia de red

4. **Policy Templates**
   - Plantillas con placeholders (?principal, ?resource)
   - Políticas dinámicas para compartir recursos
   - Template-linked policies

5. **Multi-Tenancy**
   - Policy Store por tenant (aislamiento fuerte)
   - Policy Store compartido con atributos (aislamiento lógico)

---

## ✅ ÉPICA 4: IDENTITY + JWT VALIDATION

### Estado Documentado: 100% COMPLETADA
### Estado Real a Verificar: ?

---

### HU 4.1: Configurar Identity Source

**Requisitos AWS AVP:**
- Configuración de issuer URI
- Configuración de audience
- JWKS URI para obtener claves públicas
- Asociación a PolicyStore

**Verificación de Código:**
