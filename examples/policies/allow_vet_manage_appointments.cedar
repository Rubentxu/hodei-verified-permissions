permit(
    principal in RealmRole::"pet-vet",
    action in [Action::"create", Action::"update"],
    resource like Resource::"pets/*/appointments"
);
