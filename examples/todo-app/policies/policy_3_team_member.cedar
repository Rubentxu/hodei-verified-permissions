//  Team members can view and create tasks, and update/complete their own tasks
permit(
    principal in TodoApp::UserGroup::"team_members",
    action in [
        TodoApp::Action::"listTasks",
        TodoApp::Action::"getTask",
        TodoApp::Action::"createTask",
        TodoApp::Action::"listProjects",
        TodoApp::Action::"getProject"
    ],
    resource
);

// Team members can update tasks assigned to them
permit(
    principal,
    action in [
        TodoApp::Action::"updateTask",
        TodoApp::Action::"completeTask"
    ],
    resource
)
when {
    resource has assignee && resource.assignee == principal
};
