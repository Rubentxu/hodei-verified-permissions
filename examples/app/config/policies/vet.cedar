// =============================================================================
// POLÍTICA: Vet Access
// Descripción: Los veterinarios pueden ver todo y crear citas
// =============================================================================

permit(
    principal in Group::"pet-vet",
    action in [
        Action::"viewDashboard",
        Action::"listPets",
        Action::"viewPet",
        Action::"listAppointments",
        Action::"createAppointment"
    ],
    resource
)
when {
    principal.role == "pet-vet"
};

// Denegar explícitamente acciones de admin
forbid(
    principal in Group::"pet-vet",
    action in [
        Action::"createPet",
        Action::"updatePet",
        Action::"deletePet"
    ],
    resource
)
when {
    principal.role == "pet-vet"
};