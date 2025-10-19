permit(
    principal in RealmRole::"pet-customer",
    action == Action::"read",
    resource like Resource::"pets/*"
);
