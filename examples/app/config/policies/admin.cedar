// =============================================================================
// POLÍTICA: Admin Full Access
// Descripción: Los administradores tienen acceso completo a todas las acciones
// =============================================================================

permit(
    principal in Group::"pet-admin",
    action in [
        Action::"viewDashboard",
        Action::"listPets",
        Action::"createPet",
        Action::"updatePet",
        Action::"deletePet",
        Action::"viewPet",
        Action::"listAppointments",
        Action::"createAppointment"
    ],
    resource
)
when {
    principal.role == "pet-admin"
};