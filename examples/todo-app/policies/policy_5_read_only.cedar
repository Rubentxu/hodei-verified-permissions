// Read-only users can only view tasks and projects
permit(
    principal in TodoApp::UserGroup::"viewers",
    action in [
        TodoApp::Action::"listTasks",
        TodoApp::Action::"getTask",
        TodoApp::Action::"listProjects",
        TodoApp::Action::"getProject"
    ],
    resource
);
