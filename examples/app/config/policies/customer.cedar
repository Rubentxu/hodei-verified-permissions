// =============================================================================
// POLÍTICA: Customer Read-Only Access
// Descripción: Los clientes solo pueden ver información, no modificar
// =============================================================================

permit(
    principal in Group::"pet-customer",
    action in [
        Action::"viewDashboard",
        Action::"listPets",
        Action::"viewPet",
        Action::"listAppointments"
    ],
    resource
)
when {
    principal.role == "pet-customer"
};

// Denegar todas las acciones de escritura
forbid(
    principal in Group::"pet-customer",
    action in [
        Action::"createPet",
        Action::"updatePet",
        Action::"deletePet",
        Action::"createAppointment"
    ],
    resource
)
when {
    principal.role == "pet-customer"
};