// Allows admin usergroup access to everything
permit(
    principal in TodoApp::UserGroup::"admin",
    action,
    resource
);