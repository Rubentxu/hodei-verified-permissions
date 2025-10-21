// Project managers can manage tasks and assign them to team members
permit(
    principal in TodoApp::UserGroup::"project_managers",
    action in [
        TodoApp::Action::"listTasks",
        TodoApp::Action::"getTask",
        TodoApp::Action::"createTask",
        TodoApp::Action::"updateTask",
        TodoApp::Action::"deleteTask",
        TodoApp::Action::"assignTask",
        TodoApp::Action::"completeTask",
        TodoApp::Action::"listProjects",
        TodoApp::Action::"getProject",
        TodoApp::Action::"createProject"
    ],
    resource
);
